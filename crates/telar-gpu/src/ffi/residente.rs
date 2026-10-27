//! Recursos residentes por fotograma. La geometría nunca se genera en el host.
//! Los kernels llegan como bytes; este módulo no conoce NUDO ni el compilador.

use super::{
    buffer::{Buffer, como_bytes},
    contexto::{Contexto, ok},
    malla_gpu::MallaGpu,
    recursos::Recursos,
    tipos::Comandos,
    tipos_calculo::{BUFFER_ALMACEN, BUFFER_INDIRECTO, BUFFER_TRANSFER_SRC},
    tipos_grafico::{BUFFER_INDICES, BUFFER_VERTICES},
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE};
use crate::{ActividadProcedural, KernelsProcedurales, ParametrosProcedurales};

pub(crate) struct Ranura {
    // Antes de los buffers: Drop de Recursos espera a que acabe su uso en GPU.
    pub(super) recursos: [Recursos; 3],
    pub(crate) parametros: Buffer,
    pub(crate) malla: MallaGpu,
    pub(crate) control: Buffer,
    pub(crate) comando: Comandos,
    pub(crate) capacidad: u32,
}

pub(crate) struct Residente {
    pub(crate) ranuras: Vec<Ranura>,
    pub(super) actividad: ActividadProcedural,
    pub(super) pendiente: bool,
}

impl Residente {
    pub(crate) fn nuevo(
        ctx: &Contexto,
        capacidad: u32,
        kernels: KernelsProcedurales,
        cantidad: usize,
    ) -> Result<Self, String> {
        if !(2..=1024).contains(&capacidad) {
            return Err("capacidad procedural fuera de 2..=1024 puntos por lado".to_owned());
        }
        if (capacidad * capacidad).div_ceil(64) > ctx.grupos_max_x {
            return Err("la capacidad excede los grupos de cálculo del dispositivo".to_owned());
        }
        let mut residente = Self { ranuras: Vec::new(), actividad: ActividadProcedural::default(), pendiente: false };
        for _ in 0..cantidad {
            residente.ranuras.push(Ranura::nueva(ctx, capacidad, kernels, &mut residente.actividad)?);
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
        let recursos = &ranura.recursos[0];
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
        kernels: KernelsProcedurales,
        actividad: &mut ActividadProcedural,
    ) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("falta el dispositivo")?;
        let mut crear = |bytes, uso, memoria| {
            let buffer = Buffer::nuevo(ctx, bytes, uso, memoria)?;
            actividad.buffers += 1;
            Ok::<_, String>(buffer)
        };
        let parametros = crear(16, BUFFER_ALMACEN, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
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
        let control = crear(48, BUFFER_ALMACEN | BUFFER_INDIRECTO | BUFFER_TRANSFER_SRC, (MEMORIA_GPU, 0))?;
        let mut recursos = core::array::from_fn::<_, 3, _>(|_| Recursos::nuevo(&fd, ctx.dispositivo));
        let buffers = [&parametros, &vertices, &indices, &control].map(|b| (b.objeto, b.tamano));
        for (recurso, spirv) in recursos.iter_mut().zip([kernels.preparar, kernels.generar, kernels.deformar]) {
            recurso.pipeline_enlaces(spirv, 4)?;
            actividad.pipelines += 1;
            recurso.descriptores_buffers(&buffers)?;
        }
        let comando = recursos[0].preparar_consultas(ctx, if ctx.bits_tiempo > 0 { 4 } else { 0 })?;
        let ranura = Self {
            recursos,
            parametros,
            malla: MallaGpu { vertices, indices, n_indices: 0 },
            control,
            comando,
            capacidad,
        };
        ranura.grabar(ctx)?;
        actividad.grabaciones += 1;
        Ok(ranura)
    }

    /// Se llama después de esperar la valla del fotograma dueño de esta ranura.
    pub(crate) fn actualizar(&self, parametros: ParametrosProcedurales) -> Result<(), String> {
        self.parametros.escribir(0, como_bytes(&parametros.palabras(self.capacidad)?))
    }
}
