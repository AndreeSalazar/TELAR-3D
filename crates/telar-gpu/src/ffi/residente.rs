//! Recursos residentes por fotograma. La geometría nunca se genera en el host.
//! Los kernels llegan como bytes; este módulo no conoce NUDO ni el compilador.

use super::{
    buffer::{Buffer, BufferMapeado, como_bytes, una_vez},
    contexto::{Contexto, ok},
    malla_gpu::MallaGpu,
    recursos::Recursos,
    tipos::Comandos,
    tipos_calculo::{BUFFER_ALMACEN, BUFFER_INDIRECTO, BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC},
    tipos_grafico::{BUFFER_INDICES, BUFFER_VERTICES},
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE};
use crate::procedural::TAMANO_CONTROL;
use crate::{ActividadProcedural, DominioProcedural, ParametrosProcedurales, PlanProcedural};

pub(super) struct EtapaResidente {
    pub(super) recursos: Recursos,
    pub(super) nombre: String,
    pub(super) dominio: DominioProcedural,
}

pub(crate) struct Ranura {
    // Antes de los buffers: Drop de Recursos espera a que acabe su uso en GPU.
    pub(super) etapas: Vec<EtapaResidente>,
    pub(crate) parametros: BufferMapeado,
    pub(crate) malla: MallaGpu,
    pub(crate) control: Buffer,
    pub(super) arranque: Option<Buffer>,
    pub(crate) comando: Comandos,
    pub(crate) capacidad: u32,
}

pub(crate) struct Residente {
    pub(crate) ranuras: Vec<Ranura>,
    pub(super) actividad: ActividadProcedural,
    pub(super) pendiente: bool,
}

impl Residente {
    pub(crate) fn nuevo(ctx: &Contexto, capacidad: u32, plan: PlanProcedural, cantidad: usize) -> Result<Self, String> {
        if !(2..=1024).contains(&capacidad) {
            return Err("capacidad procedural fuera de 2..=1024 puntos por lado".to_owned());
        }
        if (capacidad * capacidad).div_ceil(64) > ctx.grupos_max_x {
            return Err("la capacidad excede los grupos de cálculo del dispositivo".to_owned());
        }
        if plan.arranque().is_some() && ctx.perfil.almacenes < 5 {
            return Err("el arranque requiere cinco buffers de almacenamiento por etapa".to_owned());
        }
        let mayor = u64::from((capacidad - 1).pow(2) * 6 * 4)
            .max(u64::from(capacidad * capacidad) * 24)
            .max(TAMANO_CONTROL)
            .max(plan.arranque().map_or(0, |a| core::mem::size_of_val(a) as u64));
        if mayor > u64::from(ctx.perfil.rango_almacen) || ctx.perfil.invocaciones < 64 || ctx.perfil.hilos_x < 64 {
            return Err("los límites físicos no admiten los buffers o grupos de la ruta".to_owned());
        }
        let mut residente = Self { ranuras: Vec::new(), actividad: ActividadProcedural::default(), pendiente: false };
        for _ in 0..cantidad {
            residente.ranuras.push(Ranura::nueva(ctx, capacidad, plan, &mut residente.actividad)?);
        }
        Ok(residente)
    }

    pub(crate) fn actividad(&self) -> ActividadProcedural {
        self.actividad
    }

    pub(crate) fn contar_paso(&mut self) {
        self.actividad.pasos += 1;
        self.actividad.parametros += 16;
    }

    pub(crate) fn paso(&mut self, ctx: &Contexto, parametros: ParametrosProcedurales) -> Result<(), String> {
        use super::tipos::{InfoEnvio, ST_SUBMIT_INFO};
        if self.pendiente {
            return Err("el paso anterior no confirmó su finalización".to_owned());
        }
        let ranura = &self.ranuras[0];
        ranura.actualizar(parametros)?;
        let recursos = &ranura.etapas[0].recursos;
        let fd = recursos.fd;
        let nulo = core::ptr::null();
        let envio = InfoEnvio {
            s_type: ST_SUBMIT_INFO,
            p_next: nulo,
            n_esperas: 0,
            esperas: nulo.cast(),
            etapas_espera: nulo.cast(),
            n_comandos: 1,
            comandos: &raw const ranura.comando,
            n_senales: 0,
            senales: nulo.cast(),
        };
        // SAFETY: este ejecutor espera su valla antes de devolver; el comando es
        // reutilizable, sus recursos viven en self y ningún paso sigue pendiente.
        unsafe {
            ok((fd.reiniciar_vallas)(ctx.dispositivo, 1, &raw const recursos.valla), "reiniciar valla residente")?;
            ok((fd.enviar)(ctx.cola, 1, &raw const envio, recursos.valla), "enviar cálculo residente")?;
            self.pendiente = true;
            ok(
                (fd.esperar_vallas)(ctx.dispositivo, 1, &raw const recursos.valla, 1, 10_000_000_000),
                "esperar cálculo de prueba",
            )?;
        }
        self.pendiente = false;
        self.contar_paso();
        Ok(())
    }
}

impl Ranura {
    fn nueva(
        ctx: &Contexto,
        capacidad: u32,
        plan: PlanProcedural,
        actividad: &mut ActividadProcedural,
    ) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("falta el dispositivo")?;
        let mut crear = |bytes, uso, memoria| {
            let buffer = Buffer::nuevo(ctx, bytes, uso, memoria)?;
            actividad.buffers += 1;
            Ok::<_, String>(buffer)
        };
        let parametros = crear(16, BUFFER_ALMACEN, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?.mapear_persistente()?;
        let vertices = crear(
            u64::from(capacidad * capacidad) * 24,
            BUFFER_ALMACEN | BUFFER_VERTICES | BUFFER_TRANSFER_SRC,
            (MEMORIA_GPU, 0),
        )?;
        let indices = crear(
            u64::from((capacidad - 1).pow(2) * 6) * 4,
            BUFFER_ALMACEN | BUFFER_INDICES | BUFFER_TRANSFER_SRC,
            (MEMORIA_GPU, 0),
        )?;
        let control = crear(
            TAMANO_CONTROL,
            BUFFER_ALMACEN | BUFFER_INDIRECTO | BUFFER_TRANSFER_SRC | BUFFER_TRANSFER_DST,
            (MEMORIA_GPU, 0),
        )?;
        actividad.mapeos += 1;
        // GPU: inicializa la caché una sola vez; el lote publica transfer → compute.
        una_vez(ctx, |fd, comando| {
            // SAFETY: buffer completo, uso TRANSFER_DST; vive hasta completar
            // la valla de inicialización y durante todas las ejecuciones siguientes.
            unsafe {
                (fd.calculo.llenar_buffer)(comando, control.objeto, 0, TAMANO_CONTROL, 0);
            }
        })?;
        let arranque = if let Some(palabras) = plan.arranque() {
            let bytes = como_bytes(palabras);
            let buffer =
                Buffer::nuevo(ctx, bytes.len() as u64, BUFFER_ALMACEN | BUFFER_TRANSFER_DST, (MEMORIA_GPU, 0))?;
            let subida =
                Buffer::nuevo(ctx, bytes.len() as u64, BUFFER_TRANSFER_SRC, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
            subida.escribir(0, bytes)?;
            una_vez(ctx, |fd, cmd| {
                let copia = super::tipos_calculo::Copia { origen: 0, destino: 0, tamano: buffer.tamano };
                // GPU: la primera barrera del lote publica transfer → compute para el arranque.
                // SAFETY: ambos buffers miden bytes.len(); una_vez espera antes de liberar subida.
                unsafe {
                    (fd.calculo.copiar_buffer)(cmd, subida.objeto, buffer.objeto, 1, &raw const copia);
                }
            })?;
            actividad.buffers += 1;
            Some(buffer)
        } else {
            None
        };
        let mut etapas = Vec::with_capacity(plan.etapas().len());
        let mut buffers = [&parametros.buffer, &vertices, &indices, &control].map(|b| (b.objeto, b.tamano)).to_vec();
        if let Some(buffer) = &arranque {
            buffers.push((buffer.objeto, buffer.tamano));
        }
        for etapa in plan.etapas() {
            let mut recurso = Recursos::nuevo(&fd, ctx.dispositivo);
            recurso.pipeline_enlaces(etapa.codigo, u32::try_from(buffers.len()).map_err(|_| "demasiados buffers")?)?;
            actividad.pipelines += 1;
            recurso.descriptores_buffers(&buffers)?;
            etapas.push(EtapaResidente { recursos: recurso, nombre: etapa.nombre.to_owned(), dominio: etapa.dominio });
        }
        let consultas = u32::try_from(etapas.len() + 1).map_err(|_| "demasiadas consultas")?;
        let comando = etapas[0].recursos.preparar_consultas(ctx, if ctx.bits_tiempo > 0 { consultas } else { 0 })?;
        let ranura = Self {
            etapas,
            parametros,
            malla: MallaGpu { vertices, indices, n_indices: 0 },
            control,
            arranque,
            comando,
            capacidad,
        };
        ranura.grabar(ctx)?;
        actividad.grabaciones += 1;
        Ok(ranura)
    }

    /// Se llama después de esperar la valla del fotograma dueño de esta ranura.
    pub(crate) fn actualizar(&self, parametros: ParametrosProcedurales) -> Result<(), String> {
        self.parametros.escribir(como_bytes(&parametros.palabras(self.capacidad)?))
    }
}
