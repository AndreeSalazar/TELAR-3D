//! Captura reproducible de escena y overlay, sin tocar la ventana del usuario.
use super::{
    buffer::{Buffer, una_vez},
    contexto::Contexto,
    dibujo::{Destino, Escena, grabar},
    imagen::Imagen,
    interfaz::DibujoInterfaz,
    tipos::{ASPECTO_COLOR, ETAPA_TRANSFER, USO_COLOR_ATTACHMENT},
    tipos_calculo::{ACCESO_HOST_READ, BUFFER_TRANSFER_DST, BarreraMemoria, ETAPA_HOST, ST_MEMORY_BARRIER},
    tipos_grafico::{
        ASPECTO_PROFUNDIDAD, CopiaImagen, Extension3D, FORMATO_PROFUNDIDAD, FORMATO_RGBA8, LAYOUT_TRANSFER_SRC,
        USO_PROFUNDIDAD, USO_TRANSFER_SRC,
    },
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_VISIBLE};

pub(super) fn capturar(
    ctx: &Contexto,
    escena: &Escena,
    interfaz: &DibujoInterfaz,
    tamano: (u32, u32),
    fondo: [f32; 4],
) -> Result<Vec<u8>, String> {
    let (ancho, alto) = tamano;
    if ancho == 0 || alto == 0 || ancho > 4096 || alto > 4096 {
        return Err("captura de estudio fuera de 1..=4096 por lado".to_owned());
    }
    let color = Imagen::nueva(ctx, FORMATO_RGBA8, tamano, USO_COLOR_ATTACHMENT | USO_TRANSFER_SRC, ASPECTO_COLOR)?;
    let profundidad = Imagen::nueva(ctx, FORMATO_PROFUNDIDAD, tamano, USO_PROFUNDIDAD, ASPECTO_PROFUNDIDAD)?;
    let bytes = u64::from(ancho) * u64::from(alto) * 4;
    let lectura = Buffer::nuevo(ctx, bytes, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
    let destino = Destino {
        imagen: color.objeto,
        vista: color.vista,
        profundidad: (profundidad.objeto, profundidad.vista),
        extension: tamano,
        despues: (LAYOUT_TRANSFER_SRC, ETAPA_TRANSFER, super::tipos_calculo::ACCESO_TRANSFER_READ),
    };
    una_vez(ctx, |fd, cmd| {
        let copia = CopiaImagen {
            desplazamiento: 0,
            largo_fila: 0,
            alto_imagen: 0,
            aspecto: ASPECTO_COLOR,
            mip: 0,
            capa_base: 0,
            n_capas: 1,
            origen: [0; 3],
            extension: Extension3D { ancho, alto, fondo: 1 },
        };
        let host = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: core::ptr::null(),
            acceso_origen: super::tipos::ACCESO_TRANSFER_WRITE,
            acceso_destino: ACCESO_HOST_READ,
        };
        // SAFETY: imágenes, escena, pipeline y buffers viven hasta completar la
        // valla; la copia cubre exactamente ancho*alto*4 bytes RGBA8.
        unsafe {
            grabar(fd, cmd, &destino, fondo, Some(escena), Some(interfaz));
            (fd.grafico.copiar_imagen)(cmd, color.objeto, LAYOUT_TRANSFER_SRC, lectura.objeto, 1, &raw const copia);
            let nulo = core::ptr::null();
            // GPU: publica la copia de píxeles para la lectura diagnóstica del host.
            (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, (&raw const host).cast(), 0, nulo, 0, nulo.cast());
        }
    })?;
    lectura.leer(usize::try_from(bytes).map_err(|_| "captura demasiado grande")?)
}
