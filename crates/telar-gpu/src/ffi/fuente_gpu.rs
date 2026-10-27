//! Atlas de cobertura residente en un SSBO de solo lectura para fragmentos.
//! Se sube al preparar la fuente, sin rasterizar letras desde CPU por fotograma.
use super::{
    buffer::{Buffer, como_bytes, una_vez},
    contexto::{Contexto, ok},
    recursos::Recursos,
    tipos::{ACCESO_TRANSFER_WRITE, ETAPA_TRANSFER, Objeto},
    tipos_calculo::{
        ACCESO_SHADER_READ, BUFFER_ALMACEN, BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia,
        DESCRIPTOR_ALMACEN, Enlace, EscrituraDescriptor, InfoBufferDescriptor, InfoDisposicion,
        ST_DESCRIPTOR_SET_LAYOUT_CREATE_INFO, ST_MEMORY_BARRIER, ST_WRITE_DESCRIPTOR_SET,
    },
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE};

pub(super) struct FuenteGpu {
    // Recursos espera a que termine la GPU antes de liberar el buffer externo.
    pub(super) recursos: Recursos,
    buffer: Option<Buffer>,
}

impl FuenteGpu {
    pub(super) fn nueva(ctx: &Contexto) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("falta el dispositivo")?;
        let mut recursos = Recursos::nuevo(&fd, ctx.dispositivo);
        let enlace =
            Enlace { numero: 0, tipo: DESCRIPTOR_ALMACEN, cantidad: 1, etapas: 0x10, muestreadores: core::ptr::null() };
        let info = InfoDisposicion {
            s_type: ST_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            n_enlaces: 1,
            enlaces: &raw const enlace,
        };
        // SAFETY: estructura y enlace viven durante la llamada; el conjunto será
        // propiedad de Recursos, cuya disposición se libera después de su uso GPU.
        unsafe {
            ok(
                (fd.calculo.crear_disposicion)(
                    ctx.dispositivo,
                    &raw const info,
                    core::ptr::null(),
                    &raw mut recursos.disposicion,
                ),
                "crear disposición tipográfica",
            )?;
        }
        let mut fuente = Self { recursos, buffer: None };
        fuente.actualizar(ctx, 1, 1, &[255])?;
        Ok(fuente)
    }

    pub(super) fn disposicion(&self) -> Objeto {
        self.recursos.disposicion
    }

    pub(super) fn actualizar(&mut self, ctx: &Contexto, ancho: u32, alto: u32, cobertura: &[u8]) -> Result<(), String> {
        if ancho == 0
            || alto == 0
            || ancho > 2048
            || alto > 2048
            || ancho.checked_mul(alto).is_none_or(|n| n as usize != cobertura.len())
            || cobertura.first() != Some(&255)
        {
            return Err("atlas de cobertura inválido (máximo 2048x2048)".to_owned());
        }
        let mut datos = Vec::with_capacity(cobertura.len() + 2);
        datos.extend([ancho, alto]);
        datos.extend(cobertura.iter().map(|v| u32::from(*v)));
        let bytes = como_bytes(&datos);
        let subida =
            Buffer::nuevo(ctx, bytes.len() as u64, BUFFER_TRANSFER_SRC, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
        subida.escribir(0, bytes)?;
        let buffer = Buffer::nuevo(ctx, bytes.len() as u64, BUFFER_ALMACEN | BUFFER_TRANSFER_DST, (MEMORIA_GPU, 0))?;
        una_vez(ctx, |fd, cmd| {
            let nulo = core::ptr::null();
            let copia = Copia { origen: 0, destino: 0, tamano: buffer.tamano };
            let barrera = BarreraMemoria {
                s_type: ST_MEMORY_BARRIER,
                p_next: nulo,
                acceso_origen: ACCESO_TRANSFER_WRITE,
                acceso_destino: ACCESO_SHADER_READ,
            };
            // GPU: publica el atlas para las lecturas de los fragmentos de texto.
            // SAFETY: buffers válidos y del mismo tamaño; una_vez espera su valla.
            unsafe {
                (fd.calculo.copiar_buffer)(cmd, subida.objeto, buffer.objeto, 1, &raw const copia);
                (fd.barrera)(cmd, ETAPA_TRANSFER, 0x80, 0, 1, (&raw const barrera).cast(), 0, nulo, 0, nulo.cast());
            }
        })?;
        // Esperar el lote de subida en la misma cola completa los dibujos previos
        // antes de cambiar su descriptor, y publica la escritura del atlas.
        if self.recursos.conjunto == 0 {
            self.recursos.descriptores_buffers(&[(buffer.objeto, buffer.tamano)])?;
        } else {
            let datos = InfoBufferDescriptor { buffer: buffer.objeto, desplazamiento: 0, rango: buffer.tamano };
            let escritura = EscrituraDescriptor {
                s_type: ST_WRITE_DESCRIPTOR_SET,
                p_next: core::ptr::null(),
                conjunto: self.recursos.conjunto,
                enlace: 0,
                elemento: 0,
                cantidad: 1,
                tipo: DESCRIPTOR_ALMACEN,
                imagenes: core::ptr::null(),
                buffers: &raw const datos,
                vistas: core::ptr::null(),
            };
            // SAFETY: ningún envío anterior queda en vuelo; descriptor y buffer existen.
            unsafe {
                (self.recursos.fd.calculo.escribir_descriptores)(
                    ctx.dispositivo,
                    1,
                    &raw const escritura,
                    0,
                    core::ptr::null(),
                );
            }
        }
        self.buffer = Some(buffer);
        Ok(())
    }
}
