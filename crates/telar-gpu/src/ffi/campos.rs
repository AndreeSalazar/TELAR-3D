//! Estado único de simulación: barreras entre pasadas y entre cuadros en la misma cola.
use super::{
    buffer::{Buffer, una_vez},
    contexto::Contexto,
    funciones::FnDispositivo,
    recursos::Recursos,
    tipos::{ACCESO_TRANSFER_WRITE, Comandos, ETAPA_TRANSFER},
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_INDIRECTO, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ,
        BUFFER_ALMACEN, BUFFER_INDIRECTO, BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia,
        ETAPA_CALCULO, ETAPA_HOST, ETAPA_INDIRECTO, ETAPA_SHADER_CALCULO, PUNTO_CALCULO, ST_MEMORY_BARRIER,
    },
    tipos_grafico::ETAPA_PIPELINE_FRAGMENTOS,
};
use crate::{
    PlanCampos,
    eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE},
};
pub(crate) struct Campos {
    cache: std::cell::RefCell<super::cache_campos::Cache>,
    pub(super) programas: Vec<Recursos>,
    secuencia: Vec<usize>,
    grupos: u32,
    bytes: u64,
    pub(super) activo: bool,
    version: u64,
    rutas: Vec<crate::RutaCampos>,
    usa_indirectos: bool,
}
impl Campos {
    pub(super) fn configurar_despachos(&mut self, rutas: &[(usize, crate::RutaCampos)]) -> Result<(), String> {
        let mut nuevo = vec![crate::RutaCampos::Directo(self.grupos); self.programas.len()];
        let mut vistos = vec![false; self.programas.len()];
        for &(programa, ruta) in rutas {
            let valida = match ruta {
                crate::RutaCampos::Directo(grupos) | crate::RutaCampos::Productor(grupos) => {
                    (1..=self.grupos).contains(&grupos)
                }
                crate::RutaCampos::Indirecto(offset) => offset.is_multiple_of(4) && offset <= 52,
            };
            if programa >= nuevo.len() || !valida || vistos[programa] {
                return Err("ruta de despacho de campos inválida".into());
            }
            vistos[programa] = true;
            nuevo[programa] = ruta;
        }
        if nuevo.iter().any(|r| matches!(r, crate::RutaCampos::Indirecto(_)))
            && !nuevo.iter().any(|r| matches!(r, crate::RutaCampos::Productor(_)))
        {
            return Err("indirectos requieren un productor declarado".into());
        }
        if nuevo != self.rutas {
            self.version = self.version.checked_add(1).ok_or("versiones del plan agotadas")?;
            self.usa_indirectos = nuevo.iter().any(|r| matches!(r, crate::RutaCampos::Indirecto(_)));
            self.rutas = nuevo;
        }
        Ok(())
    }
    pub(super) fn carga(&self) -> crate::CargaCampos {
        let cache = self.cache.borrow();
        crate::CargaCampos {
            bytes: self.bytes + 64,
            grupos: self.grupos,
            ranuras: self.secuencia.len(),
            programas: self.programas.len(),
            grabaciones: cache.grabaciones,
            reusos: cache.reusos,
            directas: cache.directas,
            indirectos: self.rutas.iter().filter(|r| matches!(r, crate::RutaCampos::Indirecto(_))).count(),
        }
    }
    pub(super) fn configurar_cache(&self, activa: bool, mascara: u32) {
        self.cache.borrow_mut().configurar(activa, mascara);
    }
    /// El juego está libre tras completar su valla; nunca se usa en capturas/offline.
    pub(super) fn preparar_cache(
        &self,
        fd: &FnDispositivo,
        juego: usize,
        bloque: &[f32; 32],
    ) -> Result<Option<Comandos>, String> {
        if !self.activo {
            return Ok(None);
        }
        self.cache.borrow_mut().preparar(juego, self.version, bloque, |cmd, constantes| {
            // SAFETY: secundario del juego libre grabándose; mantiene recursos propios del plan.
            unsafe {
                self.grabar(fd, cmd, constantes);
            }
        })
    }
    pub(super) fn control(&self, offset: u32) -> Option<(super::tipos::Objeto, u64)> {
        let desde = u64::from(offset) * 4;
        (desde + 64 <= self.bytes).then_some((self.programas[0].buffers[1], desde))
    }
    pub(super) fn secuencia(&mut self, nueva: &[usize]) -> Result<(), String> {
        if nueva.is_empty() || nueva.len() > 2048 || nueva.iter().any(|v| *v >= self.programas.len()) {
            return Err("secuencia de campos inválida".into());
        }
        if self.secuencia != nueva {
            self.version = self.version.checked_add(1).ok_or("versiones del plan agotadas")?;
            self.secuencia = nueva.to_vec();
        }
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
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER | ETAPA_INDIRECTO,
                    ETAPA_TRANSFER,
                    ACCESO_SHADER_READ
                        | ACCESO_SHADER_WRITE
                        | ACCESO_TRANSFER_READ
                        | ACCESO_TRANSFER_WRITE
                        | ACCESO_INDIRECTO,
                    ACCESO_TRANSFER_WRITE,
                );
                (fd.calculo.llenar_buffer)(cmd, self.programas[0].buffers[1], 0, self.bytes, 0);
                (fd.calculo.llenar_buffer)(cmd, self.programas[0].buffers[0], 0, 64, 0);
                // GPU: el cero queda visible incluso si la simulación permanece pausada.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_INDIRECTO,
                    ACCESO_TRANSFER_WRITE,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_INDIRECTO,
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
                recursos.buffer(
                    ctx,
                    0,
                    64,
                    BUFFER_ALMACEN | BUFFER_INDIRECTO | BUFFER_TRANSFER_SRC | BUFFER_TRANSFER_DST,
                    (MEMORIA_GPU, 0),
                )?;
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
            recursos.descriptores_buffers(&[(buffers[0], 64), (buffers[1], bytes)])?;
            programas.push(recursos);
        }
        let campos = Self {
            cache: std::cell::RefCell::new(super::cache_campos::Cache::nueva(ctx)?),
            programas,
            secuencia: plan.secuencia.to_vec(),
            grupos,
            bytes,
            activo: true,
            version: 0,
            rutas: vec![crate::RutaCampos::Directo(grupos); plan.programas.len()],
            usa_indirectos: false,
        };
        una_vez(ctx, |fd, cmd| {
            // SAFETY: estado recién creado con TRANSFER_DST, rango completo alineado.
            // GPU: inicializa todos los campos antes de la primera lectura.
            unsafe {
                (fd.calculo.llenar_buffer)(cmd, campos.programas[0].buffers[1], 0, bytes, 0);
                (fd.calculo.llenar_buffer)(cmd, campos.programas[0].buffers[0], 0, 64, 0);
                // GPU: también permite visualizar cero sin ejecutar un paso de simulación.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_INDIRECTO,
                    ACCESO_TRANSFER_WRITE,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_INDIRECTO,
                );
            }
        })?;
        Ok(campos)
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
        Self::transferir_rango(ctx, datos, 0, self.bytes, self.programas[0].buffers[1])
    }
    pub(super) fn leer_argumentos(&self, ctx: &Contexto) -> Result<Vec<u32>, String> {
        Self::transferir_rango(ctx, None, 0, 64, self.programas[0].buffers[0])
    }
    pub(super) fn leer_rango(&self, ctx: &Contexto, desde: u32, cantidad: u32) -> Result<Vec<u32>, String> {
        let desde = u64::from(desde) * 4;
        let bytes = u64::from(cantidad) * 4;
        if cantidad == 0 || desde + bytes > self.bytes {
            return Err("rango de campos inválido".into());
        }
        Self::transferir_rango(ctx, None, desde, bytes, self.programas[0].buffers[1])
    }
    pub(super) fn escribir_rango(&self, ctx: &Contexto, desde: u32, datos: &[u32]) -> Result<(), String> {
        let inicio = u64::from(desde) * 4;
        let bytes = u64::try_from(core::mem::size_of_val(datos)).map_err(|_| "rango excesivo")?;
        if bytes == 0 || inicio + bytes > self.bytes {
            return Err("rango de carga inválido".into());
        }
        Self::transferir_rango(ctx, Some(datos), inicio, bytes, self.programas[0].buffers[1]).map(|_| ())
    }
    fn transferir_rango(
        ctx: &Contexto,
        datos: Option<&[u32]>,
        desde: u64,
        bytes: u64,
        objeto: super::tipos::Objeto,
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
        una_vez(ctx, |fd, cmd| {
            let copia =
                Copia { origen: if carga { 0 } else { desde }, destino: if carga { desde } else { 0 }, tamano: bytes };
            // SAFETY: ambos buffers viven hasta completar una_vez y sus rangos coinciden.
            unsafe {
                // GPU: transferencias explícitas esperan escrituras y lecturas previas del estado.
                barrera(
                    fd,
                    cmd,
                    ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER | ETAPA_INDIRECTO,
                    ETAPA_TRANSFER,
                    ACCESO_SHADER_READ
                        | ACCESO_SHADER_WRITE
                        | ACCESO_TRANSFER_WRITE
                        | ACCESO_TRANSFER_READ
                        | ACCESO_INDIRECTO,
                    if carga { ACCESO_TRANSFER_WRITE } else { ACCESO_TRANSFER_READ },
                );
                let (origen, destino) = if carga { (staging.objeto, objeto) } else { (objeto, staging.objeto) };
                (fd.calculo.copiar_buffer)(cmd, origen, destino, 1, &raw const copia);
                // GPU: publica la carga para compute/fragmentos, o la descarga para el host.
                barrera(
                    fd,
                    cmd,
                    ETAPA_TRANSFER,
                    if carga { ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_INDIRECTO } else { ETAPA_HOST },
                    ACCESO_TRANSFER_WRITE,
                    if carga { ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_INDIRECTO } else { ACCESO_HOST_READ },
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
pub(super) unsafe fn barrera(
    fd: &FnDispositivo,
    cmd: Comandos,
    origen: u32,
    destino: u32,
    lectura: u32,
    escritura: u32,
) {
    let nulo = core::ptr::null();
    let memoria =
        BarreraMemoria { s_type: ST_MEMORY_BARRIER, p_next: nulo, acceso_origen: lectura, acceso_destino: escritura };
    // SAFETY: estructura válida, sin barreras de buffers/imágenes; contrato del llamador.
    // GPU: establece dependencia de memoria explícita entre las etapas indicadas.
    unsafe {
        (fd.barrera)(cmd, origen, destino, 0, 1, (&raw const memoria).cast(), 0, nulo, 0, nulo.cast());
    }
}

#[path = "campos_comandos.rs"]
mod comandos;
