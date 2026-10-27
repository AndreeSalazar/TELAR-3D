//! Lote de cálculo pregrabado: productor → indirecto → generación → deformación.

use super::{
    contexto::{Contexto, ok},
    residente::Ranura,
    tipos::{ETAPA_TRANSFER, InfoInicioComandos, ST_COMMAND_BUFFER_BEGIN_INFO},
    tipos_calculo::{
        ACCESO_INDIRECTO, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BarreraMemoria, ETAPA_CALCULO,
        ETAPA_INDIRECTO, ETAPA_TOP_OF_PIPE, PUNTO_CALCULO, ST_MEMORY_BARRIER,
    },
    tipos_grafico::{ACCESO_LEER_ATRIBUTOS, ACCESO_LEER_INDICES, ETAPA_ENTRADA_VERTICES},
};

impl Ranura {
    pub(super) fn grabar(&self, ctx: &Contexto) -> Result<(), String> {
        let recursos = &self.recursos[0];
        let (fd, fc, cmd, nulo) = (recursos.fd, recursos.fd.calculo, self.comando, core::ptr::null());
        let inicio =
            InfoInicioComandos { s_type: ST_COMMAND_BUFFER_BEGIN_INFO, p_next: nulo, flags: 0, herencia: nulo };
        let barrera = |origen, destino, lectura, escritura| {
            let memoria = BarreraMemoria {
                s_type: ST_MEMORY_BARRIER,
                p_next: nulo,
                acceso_origen: lectura,
                acceso_destino: escritura,
            };
            // GPU: publica escrituras del productor para los consumidores de cada
            // pasada, incluidos los lectores de argumentos indirectos y de vértices.
            // SAFETY: cmd se está grabando y memoria vive durante la llamada.
            unsafe {
                (fd.barrera)(cmd, origen, destino, 0, 1, (&raw const memoria).cast(), 0, nulo, 0, nulo.cast());
            }
        };
        // SAFETY: comando nuevo, primario y propiedad de recursos; flags 0 permite
        // reenviarlo después de completar la valla, sin volver a grabarlo.
        unsafe {
            ok((fd.empezar)(cmd, &raw const inicio), "grabar el lote residente")?;
        }
        barrera(
            ETAPA_CALCULO | ETAPA_INDIRECTO | ETAPA_ENTRADA_VERTICES | ETAPA_TRANSFER,
            ETAPA_CALCULO,
            ACCESO_SHADER_WRITE
                | ACCESO_SHADER_READ
                | ACCESO_INDIRECTO
                | ACCESO_LEER_ATRIBUTOS
                | ACCESO_LEER_INDICES
                | ACCESO_TRANSFER_READ,
            ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
        );
        let medir = ctx.bits_tiempo > 0;
        // SAFETY: las cuatro consultas pertenecen a la ranura y su valla ya se
        // completó antes de cada reenvío; los pipelines y conjuntos son persistentes.
        unsafe {
            if medir {
                (fc.reiniciar_consultas)(cmd, recursos.consultas, 0, 4);
                (fc.marcar_tiempo)(cmd, ETAPA_TOP_OF_PIPE, recursos.consultas, 0);
            }
        }
        for (indice, recurso) in self.recursos.iter().enumerate() {
            // SAFETY: los conjuntos tienen los cuatro buffers vivos hasta que
            // termina la ranura; control contiene argumentos escritos por preparar.
            unsafe {
                (fc.usar_pipeline)(cmd, PUNTO_CALCULO, recurso.pipeline);
                (fc.usar_conjuntos)(
                    cmd,
                    PUNTO_CALCULO,
                    recurso.disposicion_pipeline,
                    0,
                    1,
                    &raw const recurso.conjunto,
                    0,
                    nulo.cast(),
                );
                if indice == 0 {
                    (fc.despachar)(cmd, 1, 1, 1);
                } else {
                    (fc.despachar_indirecto)(cmd, self.control.objeto, 0);
                }
                if medir {
                    (fc.marcar_tiempo)(
                        cmd,
                        ETAPA_CALCULO,
                        recursos.consultas,
                        u32::try_from(indice + 1).map_err(|_| "consulta fuera de rango")?,
                    );
                }
            }
            if indice < 2 {
                barrera(
                    ETAPA_CALCULO,
                    ETAPA_CALCULO | ETAPA_INDIRECTO,
                    ACCESO_SHADER_WRITE,
                    ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_INDIRECTO,
                );
            }
        }
        barrera(
            ETAPA_CALCULO,
            ETAPA_ENTRADA_VERTICES | ETAPA_INDIRECTO | ETAPA_TRANSFER,
            ACCESO_SHADER_WRITE,
            ACCESO_LEER_ATRIBUTOS | ACCESO_LEER_INDICES | ACCESO_INDIRECTO | ACCESO_TRANSFER_READ,
        );
        // SAFETY: las tres pasadas y sus barreras están completas.
        unsafe { ok((fd.terminar)(cmd), "cerrar el lote residente") }
    }
}
