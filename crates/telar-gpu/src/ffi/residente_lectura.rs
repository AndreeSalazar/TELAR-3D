//! Diagnóstico explícito. La ruta viva no llama aquí ni consulta resultados CPU.

use super::{
    buffer::{Buffer, una_vez},
    contexto::{Contexto, ok},
    pipeline_grafico::PipelineGrafico,
    residente::Residente,
    tipos::ETAPA_TRANSFER,
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_TRANSFER_DST, BarreraMemoria, Copia,
        ETAPA_CALCULO, ETAPA_HOST, RESULTADO_64, ST_MEMORY_BARRIER,
    },
    visor::capturar_buffers,
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_CON_CACHE, MEMORIA_VISIBLE, nanosegundos};
use crate::{Constantes, DiagnosticoProcedural, FORMATO_CAPTURA, Sombreadores};

impl Residente {
    pub(crate) fn capturar(
        &self,
        ctx: &Contexto,
        shaders: Sombreadores,
        tamano: (u32, u32),
        fondo: [f32; 4],
        constantes: &Constantes,
    ) -> Result<Vec<u8>, String> {
        if self.actividad.pasos == 0 || self.pendiente {
            return Err("primero hay que ejecutar un paso".to_owned());
        }
        let pipeline = PipelineGrafico::nuevo(ctx, shaders.vertices, shaders.fragmentos, FORMATO_CAPTURA)?;
        let ranura = &self.ranuras[0];
        capturar_buffers(
            ctx,
            &pipeline,
            &ranura.malla,
            Some(ranura.control.objeto),
            tamano,
            fondo,
            &constantes.plana(FORMATO_CAPTURA),
        )
    }

    pub(crate) fn inspeccionar(&mut self, ctx: &Contexto) -> Result<DiagnosticoProcedural, String> {
        if self.actividad.pasos == 0 || self.pendiente {
            return Err("primero hay que ejecutar un paso".to_owned());
        }
        let ranura = &self.ranuras[0];
        let nv = ranura.malla.vertices.tamano;
        let ni = ranura.malla.indices.tamano;
        let bytes = 64 + nv + ni;
        let lectura =
            Buffer::nuevo(ctx, bytes, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, MEMORIA_CON_CACHE))?;
        una_vez(ctx, |fd, cmd| {
            let nulo = core::ptr::null();
            let barrera = BarreraMemoria {
                s_type: ST_MEMORY_BARRIER,
                p_next: nulo,
                acceso_origen: ACCESO_SHADER_WRITE,
                acceso_destino: ACCESO_TRANSFER_READ,
            };
            // GPU: publicación del control y geometría para diagnóstico, fuera del bucle vivo.
            // SAFETY: los tres orígenes y lectura viven hasta completar la valla;
            // las regiones de destino no se solapan y caben en el buffer.
            unsafe {
                (fd.barrera)(
                    cmd,
                    ETAPA_CALCULO,
                    ETAPA_TRANSFER,
                    0,
                    1,
                    (&raw const barrera).cast(),
                    0,
                    nulo,
                    0,
                    nulo.cast(),
                );
                for (buffer, destino, tamano) in [
                    (ranura.control.objeto, 0, 64),
                    (ranura.malla.vertices.objeto, 64, nv),
                    (ranura.malla.indices.objeto, 64 + nv, ni),
                ] {
                    let copia = Copia { origen: 0, destino, tamano };
                    (fd.calculo.copiar_buffer)(cmd, buffer, lectura.objeto, 1, &raw const copia);
                }
                let host = BarreraMemoria {
                    s_type: ST_MEMORY_BARRIER,
                    p_next: nulo,
                    acceso_origen: super::tipos::ACCESO_TRANSFER_WRITE,
                    acceso_destino: ACCESO_HOST_READ,
                };
                (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, (&raw const host).cast(), 0, nulo, 0, nulo.cast());
            }
        })?;
        let datos = lectura.leer(usize::try_from(bytes).map_err(|_| "diagnóstico demasiado grande")?)?;
        self.actividad.descargados += bytes;
        let palabras: Vec<u32> = datos.as_chunks::<4>().0.iter().map(|b| u32::from_ne_bytes(*b)).collect();
        let control: [u32; 16] = palabras[..16].try_into().map_err(|_| "control incompleto")?;
        let puntos = control[9] as usize;
        let indices = control[3] as usize;
        if puntos > (nv / 24) as usize || indices > (ni / 4) as usize {
            return Err("los recuentos GPU exceden la capacidad".to_owned());
        }
        let vertices = palabras[16..16 + puntos * 6].as_chunks::<6>().0.iter().map(|v| v.map(f32::from_bits)).collect();
        let desde = 16 + (nv / 4) as usize;
        let indices = palabras[desde..desde + indices].to_vec();
        Ok(DiagnosticoProcedural { control, vertices, indices, nanosegundos: self.tiempos(ctx)? })
    }

    fn tiempos(&self, ctx: &Contexto) -> Result<Option<[f64; 3]>, String> {
        if ctx.bits_tiempo == 0 {
            return Ok(None);
        }
        let recurso = &self.ranuras[0].recursos[0];
        let mut tics = [0u64; 4];
        // SAFETY: paso esperó su valla; las cuatro consultas escritas caben en tics.
        unsafe {
            ok(
                (recurso.fd.calculo.leer_consultas)(
                    ctx.dispositivo,
                    recurso.consultas,
                    0,
                    4,
                    32,
                    tics.as_mut_ptr().cast(),
                    8,
                    RESULTADO_64,
                ),
                "leer tiempos residentes",
            )?;
        }
        Ok(Some(core::array::from_fn(|i| nanosegundos(tics[i], tics[i + 1], ctx.bits_tiempo, ctx.periodo_ns))))
    }
}
