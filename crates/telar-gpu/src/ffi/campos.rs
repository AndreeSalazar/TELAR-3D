//! Estado único de simulación: barreras entre pasadas y entre cuadros en la misma cola.
use super::{
    buffer::{Buffer, una_vez},
    contexto::Contexto,
    funciones::FnDispositivo,
    recursos::Recursos,
    tipos::{ACCESO_TRANSFER_WRITE, Comandos, ETAPA_TRANSFER},
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_ALMACEN,
        BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia, ETAPA_CALCULO, ETAPA_HOST,
        ETAPA_SHADER_CALCULO, PUNTO_CALCULO, ST_MEMORY_BARRIER,
    },
    tipos_grafico::ETAPA_PIPELINE_FRAGMENTOS,
};
use crate::{
    PlanCampos,
    eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE},
};
pub(crate) struct Campos {
    pub(super) programas: Vec<Recursos>,
    secuencia: Vec<usize>,
    grupos: u32,
    bytes: u64,
    pub(super) activo: bool,
}
impl Campos {
    pub(super) fn secuencia(&mut self, nueva: &[usize]) -> Result<(), String> {
        if nueva.is_empty() || nueva.len() > 2048 || nueva.iter().any(|v| *v >= self.programas.len()) {
            return Err("secuencia de campos inválida".into());
        }
        self.secuencia = nueva.to_vec();
        Ok(())
    }
    pub(super) fn limpiar(&self, ctx: &Contexto) -> Result<(), String> {
        una_vez(ctx, |fd, cmd| {
            // SAFETY: buffer vivo y rango completo alineado; envío sobre la misma cola.
            unsafe {
                // GPU: espera usos previos del estado antes de reinicializarlo.
                barrera(
                    fd,
                    cmd,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER,
                    ETAPA_TRANSFER,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_TRANSFER_READ | ACCESO_TRANSFER_WRITE,
                    ACCESO_TRANSFER_WRITE,
                );
                (fd.calculo.llenar_buffer)(cmd, self.programas[0].buffers[1], 0, self.bytes, 0);
                // GPU: el cero queda visible incluso si la simulación permanece pausada.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS,
                    ACCESO_TRANSFER_WRITE,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
                );
            }
        })
    }
    pub(super) fn nuevo(ctx: &Contexto, plan: PlanCampos) -> Result<Self, String> {
        let bytes = plan.validar()?;
        let grupos = plan.elementos.div_ceil(64);
        if grupos > ctx.grupos_max_x {
            return Err("campos exceden grupos físicos".into());
        }
        let fd = ctx.fd.ok_or("falta dispositivo")?;
        let mut programas = Vec::new();
        for codigo in plan.programas {
            let mut recursos = Recursos::nuevo(&fd, ctx.dispositivo);
            if programas.is_empty() {
                recursos.buffer(ctx, 0, 4, BUFFER_ALMACEN, (MEMORIA_GPU, 0))?;
                recursos.buffer(
                    ctx,
                    1,
                    bytes,
                    BUFFER_ALMACEN | BUFFER_TRANSFER_SRC | BUFFER_TRANSFER_DST,
                    (MEMORIA_GPU, 0),
                )?;
            }
            recursos.pipeline_campos(codigo)?;
            let buffers = programas.first().map_or(recursos.buffers, |r: &Recursos| r.buffers);
            recursos.descriptores_buffers(&[(buffers[0], 4), (buffers[1], bytes)])?;
            programas.push(recursos);
        }
        let campos = Self { programas, secuencia: plan.secuencia.to_vec(), grupos, bytes, activo: true };
        una_vez(ctx, |fd, cmd| {
            // SAFETY: estado recién creado con TRANSFER_DST, rango completo alineado.
            // GPU: inicializa todos los campos antes de la primera lectura.
            unsafe {
                (fd.calculo.llenar_buffer)(cmd, campos.programas[0].buffers[1], 0, bytes, 0);
                // GPU: también permite visualizar cero sin ejecutar un paso de simulación.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS,
                    ACCESO_TRANSFER_WRITE,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
                );
            }
        })?;
        Ok(campos)
    }
    /// # Safety
    /// Comando grabándose fuera de rendering; recursos viven hasta completar la valla.
    pub(super) unsafe fn grabar(&self, fd: &FnDispositivo, cmd: Comandos, bloque: &[f32; 32]) {
        if !self.activo {
            return;
        }
        // SAFETY: el llamador conserva todos los programas/buffers hasta terminar el envío.
        unsafe {
            for (numero, &indice) in self.secuencia.iter().enumerate() {
                // GPU: protege lectura anterior de fragmentos y escritura de la pasada anterior.
                barrera(
                    fd,
                    cmd,
                    if numero == 0 {
                        ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER
                    } else {
                        ETAPA_CALCULO
                    },
                    ETAPA_CALCULO,
                    if numero == 0 {
                        ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_TRANSFER_READ | ACCESO_TRANSFER_WRITE
                    } else {
                        ACCESO_SHADER_WRITE
                    },
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
                );
                let r = &self.programas[indice];
                (fd.calculo.usar_pipeline)(cmd, PUNTO_CALCULO, r.pipeline);
                (fd.calculo.usar_conjuntos)(
                    cmd,
                    PUNTO_CALCULO,
                    r.disposicion_pipeline,
                    0,
                    1,
                    &raw const r.conjunto,
                    0,
                    core::ptr::null(),
                );
                (fd.grafico.empujar_constantes)(
                    cmd,
                    r.disposicion_pipeline,
                    ETAPA_SHADER_CALCULO,
                    0,
                    128,
                    bloque.as_ptr().cast(),
                );
                (fd.calculo.despachar)(cmd, self.grupos, 1, 1);
            }
            // GPU: publica todos los campos para el visor posterior y sus lecturas de diagnóstico.
            barrera(fd, cmd, ETAPA_CALCULO, ETAPA_PIPELINE_FRAGMENTOS, ACCESO_SHADER_WRITE, ACCESO_SHADER_READ);
        }
    }
    pub(super) fn ejecutar(&self, ctx: &Contexto, bloque: &[f32; 32]) -> Result<(), String> {
        una_vez(ctx, |fd, cmd| {
            // SAFETY: una_vez conserva el estado/pipelines hasta completar la valla, fuera de rendering.
            unsafe {
                self.grabar(fd, cmd, bloque);
            }
        })
    }
    pub(super) fn transferir(&self, ctx: &Contexto, datos: Option<&[u32]>) -> Result<Vec<u32>, String> {
        if datos.is_some_and(|v| core::mem::size_of_val(v) as u64 != self.bytes) {
            return Err("tamaño de campos incompatible".into());
        }
        self.transferir_rango(ctx, datos, 0, self.bytes)
    }
    pub(super) fn leer_rango(&self, ctx: &Contexto, desde: u32, cantidad: u32) -> Result<Vec<u32>, String> {
        let desde = u64::from(desde) * 4;
        let bytes = u64::from(cantidad) * 4;
        if cantidad == 0 || desde + bytes > self.bytes {
            return Err("rango de campos inválido".into());
        }
        self.transferir_rango(ctx, None, desde, bytes)
    }
    pub(super) fn escribir_rango(&self, ctx: &Contexto, desde: u32, datos: &[u32]) -> Result<(), String> {
        let inicio = u64::from(desde) * 4;
        let bytes = u64::try_from(core::mem::size_of_val(datos)).map_err(|_| "rango excesivo")?;
        if bytes == 0 || inicio + bytes > self.bytes {
            return Err("rango de carga inválido".into());
        }
        self.transferir_rango(ctx, Some(datos), inicio, bytes).map(|_| ())
    }
    fn transferir_rango(
        &self,
        ctx: &Contexto,
        datos: Option<&[u32]>,
        desde: u64,
        bytes: u64,
    ) -> Result<Vec<u32>, String> {
        let carga = datos.is_some();
        let staging = Buffer::nuevo(
            ctx,
            bytes,
            if carga { BUFFER_TRANSFER_SRC } else { BUFFER_TRANSFER_DST },
            (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0),
        )?;
        if let Some(datos) = datos {
            staging.escribir(0, super::buffer::como_bytes(datos))?;
        }
        let objeto = self.programas[0].buffers[1];
        una_vez(ctx, |fd, cmd| {
            let copia =
                Copia { origen: if carga { 0 } else { desde }, destino: if carga { desde } else { 0 }, tamano: bytes };
            // SAFETY: ambos buffers viven hasta completar una_vez y sus rangos coinciden.
            unsafe {
                // GPU: transferencias explícitas esperan escrituras y lecturas previas del estado.
                barrera(
                    fd,
                    cmd,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER,
                    ETAPA_TRANSFER,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_TRANSFER_WRITE | ACCESO_TRANSFER_READ,
                    if carga { ACCESO_TRANSFER_WRITE } else { ACCESO_TRANSFER_READ },
                );
                let (origen, destino) = if carga { (staging.objeto, objeto) } else { (objeto, staging.objeto) };
                (fd.calculo.copiar_buffer)(cmd, origen, destino, 1, &raw const copia);
                // GPU: publica la carga para compute/fragmentos, o la descarga para el host.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    if carga { ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS } else { ETAPA_HOST },
                    ACCESO_TRANSFER_WRITE,
                    if carga { ACCESO_SHADER_READ | ACCESO_SHADER_WRITE } else { ACCESO_HOST_READ },
                );
            }
        })?;
        if carga {
            return Ok(Vec::new());
        }
        let bytes = staging.leer(usize::try_from(bytes).map_err(|_| "campos excesivos")?)?;
        Ok(bytes.as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect())
    }
}
/// # Safety
/// Mismos requisitos de vida y grabación que el comando que protege.
unsafe fn barrera(fd: &FnDispositivo, cmd: Comandos, origen: u32, destino: u32, lectura: u32, escritura: u32) {
    let nulo = core::ptr::null();
    let memoria =
        BarreraMemoria { s_type: ST_MEMORY_BARRIER, p_next: nulo, acceso_origen: lectura, acceso_destino: escritura };
    // SAFETY: estructura válida, sin barreras de buffers/imágenes; contrato del llamador.
    // GPU: establece dependencia de memoria explícita entre las etapas indicadas.
    unsafe {
        (fd.barrera)(cmd, origen, destino, 0, 1, (&raw const memoria).cast(), 0, nulo, 0, nulo.cast());
    }
}
