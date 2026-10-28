//! Historial único compartido por los fotogramas: barreras ordenan compute y lectura.
use super::{
    buffer::{Buffer, una_vez},
    contexto::Contexto,
    funciones::FnDispositivo,
    recursos::Recursos,
    tipos::{ACCESO_TRANSFER_WRITE, Comandos, ETAPA_TRANSFER},
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_ALMACEN,
        BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia, ETAPA_CALCULO, ETAPA_HOST, PUNTO_CALCULO,
        ST_MEMORY_BARRIER,
    },
    tipos_grafico::ETAPA_PIPELINE_FRAGMENTOS,
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE};
pub(crate) struct Historial {
    pub(super) recursos: Recursos,
    pub(super) tamano: [u32; 2],
    bytes: u64,
    grupos: u32,
}
impl Historial {
    pub(super) fn nuevo(ctx: &Contexto, codigo: &[u32], tamano: [u32; 2]) -> Result<Self, String> {
        if !(1..=1024).contains(&tamano[0])
            || !(1..=768).contains(&tamano[1])
            || codigo.len() < 5
            || codigo[0] != 0x0723_0203
        {
            return Err("historial fuera de contrato".into());
        }
        let fd = ctx.fd.ok_or("falta dispositivo")?;
        let pixeles = tamano[0] * tamano[1];
        let bytes = u64::from(pixeles) * 16;
        let grupos = pixeles.div_ceil(64);
        if grupos > ctx.grupos_max_x {
            return Err("historial excede límite de grupos".into());
        }
        let mut recursos = Recursos::nuevo(&fd, ctx.dispositivo);
        recursos.buffer(ctx, 0, 4, BUFFER_ALMACEN, (MEMORIA_GPU, 0))?;
        recursos.buffer(ctx, 1, bytes, BUFFER_ALMACEN | BUFFER_TRANSFER_SRC | BUFFER_TRANSFER_DST, (MEMORIA_GPU, 0))?;
        recursos.pipeline_enlaces(codigo, 2)?;
        recursos.descriptores_buffers(&[(recursos.buffers[0], 4), (recursos.buffers[1], bytes)])?;
        una_vez(ctx, |fd, cmd| {
            // SAFETY: buffer vivo, TRANSFER_DST y rango completo múltiplo de cuatro.
            // GPU: primera escritura de sumas y contadores; no se lee memoria sin inicializar.
            unsafe {
                (fd.calculo.llenar_buffer)(cmd, recursos.buffers[1], 0, bytes, 0);
            }
        })?;
        Ok(Self { recursos, tamano, bytes, grupos })
    }
    /// # Safety
    /// Comando grabándose fuera del renderizado; recursos viven hasta su valla.
    pub(super) unsafe fn grabar(&self, fd: &FnDispositivo, cmd: Comandos) {
        let nulo = core::ptr::null();
        let etapas = ETAPA_CALCULO | ETAPA_PIPELINE_FRAGMENTOS | ETAPA_TRANSFER;
        let antes = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: nulo,
            acceso_origen: ACCESO_SHADER_READ | ACCESO_SHADER_WRITE | ACCESO_TRANSFER_WRITE | ACCESO_TRANSFER_READ,
            acceso_destino: ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
        };
        let despues = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: nulo,
            acceso_origen: ACCESO_SHADER_WRITE,
            acceso_destino: ACCESO_SHADER_READ,
        };
        // SAFETY: el llamador garantiza grabación, vida útil y misma cola ordenada.
        unsafe {
            // GPU: impide que otra muestra sobrescriba mientras el fotograma anterior lee.
            (fd.barrera)(cmd, etapas, ETAPA_CALCULO, 0, 1, (&raw const antes).cast(), 0, nulo, 0, nulo.cast());
            (fd.calculo.usar_pipeline)(cmd, PUNTO_CALCULO, self.recursos.pipeline);
            (fd.calculo.usar_conjuntos)(
                cmd,
                PUNTO_CALCULO,
                self.recursos.disposicion_pipeline,
                0,
                1,
                &raw const self.recursos.conjunto,
                0,
                nulo.cast(),
            );
            (fd.calculo.despachar)(cmd, self.grupos, 1, 1);
            // GPU: publica suma/contador para el fragmento que muestra la media.
            (fd.barrera)(
                cmd,
                ETAPA_CALCULO,
                ETAPA_PIPELINE_FRAGMENTOS,
                0,
                1,
                (&raw const despues).cast(),
                0,
                nulo,
                0,
                nulo.cast(),
            );
        }
    }
    pub(super) fn inspeccionar(&self, ctx: &Contexto) -> Result<Vec<[u32; 4]>, String> {
        let lectura = Buffer::nuevo(ctx, self.bytes, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
        una_vez(ctx, |fd, cmd| {
            let nulo = core::ptr::null();
            let copia = Copia { origen: 0, destino: 0, tamano: self.bytes };
            let memoria = BarreraMemoria {
                s_type: ST_MEMORY_BARRIER,
                p_next: nulo,
                acceso_origen: ACCESO_SHADER_WRITE | ACCESO_TRANSFER_WRITE,
                acceso_destino: ACCESO_TRANSFER_READ,
            };
            let host = BarreraMemoria {
                s_type: ST_MEMORY_BARRIER,
                p_next: nulo,
                acceso_origen: ACCESO_TRANSFER_WRITE,
                acceso_destino: ACCESO_HOST_READ,
            };
            // SAFETY: buffers de tamaño conocido y vivos; una_vez espera antes de leer/destruir.
            unsafe {
                // GPU: diagnóstico explícito después de inicialización o de la última escritura compute.
                (fd.barrera)(
                    cmd,
                    ETAPA_CALCULO | ETAPA_TRANSFER,
                    ETAPA_TRANSFER,
                    0,
                    1,
                    (&raw const memoria).cast(),
                    0,
                    nulo,
                    0,
                    nulo.cast(),
                );
                (fd.calculo.copiar_buffer)(cmd, self.recursos.buffers[1], lectura.objeto, 1, &raw const copia);
                (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, (&raw const host).cast(), 0, nulo, 0, nulo.cast());
            }
        })?;
        let bytes = lectura.leer(usize::try_from(self.bytes).map_err(|_| "historial excesivo")?)?;
        Ok(bytes
            .as_chunks::<16>()
            .0
            .iter()
            .map(|p| core::array::from_fn(|i| u32::from_le_bytes([p[i * 4], p[i * 4 + 1], p[i * 4 + 2], p[i * 4 + 3]])))
            .collect())
    }
    pub(super) fn validar_bloque(&self, bloque: &[f32; 32]) -> Result<(), String> {
        #[allow(clippy::cast_precision_loss, reason = "dimensiones <=1024")]
        let dimensiones = self.tamano.map(|v| v as f32);
        if bloque[2..4] != dimensiones {
            return Err("constantes no coinciden con dimensiones del historial".into());
        }
        Ok(())
    }
}
