//! Captura reproducible de escena y overlay, sin tocar la ventana del usuario.
use super::{
    buffer::{Buffer, una_vez},
    contexto::{Contexto, ok},
    dibujo::{Destino, Escena, grabar},
    imagen::Imagen,
    interfaz::DibujoInterfaz,
    recursos::Recursos,
    tipos::{ASPECTO_COLOR, ETAPA_TRANSFER, USO_COLOR_ATTACHMENT},
    tipos_calculo::{ACCESO_HOST_READ, BUFFER_TRANSFER_DST, BarreraMemoria, ETAPA_HOST, ST_MEMORY_BARRIER},
    tipos_grafico::{CopiaImagen, Extension3D, FORMATO_RGBA8, LAYOUT_TRANSFER_SRC, USO_TRANSFER_SRC},
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_VISIBLE};

pub(crate) fn capturar_malla(
    ctx: &Contexto,
    shaders: crate::Sombreadores,
    malla: (&[[f32; 6]], &[u32]),
    tamano: (u32, u32),
    fondo: [f32; 4],
    constantes: &[f32; 32],
    capa: crate::CapaInterfaz,
) -> Result<crate::CapturaEstudio, String> {
    if capa.area[2] == 0
        || capa.area[3] == 0
        || capa.area[0].checked_add(capa.area[2]).is_none_or(|v| v > tamano.0)
        || capa.area[1].checked_add(capa.area[3]).is_none_or(|v| v > tamano.1)
    {
        return Err("área de captura inválida".into());
    }
    let pipeline =
        super::pipeline_grafico::PipelineGrafico::nuevo(ctx, shaders.vertices, shaders.fragmentos, FORMATO_RGBA8)?;
    let malla = super::malla_gpu::MallaGpu::nueva(ctx, malla.0, malla.1)?;
    let mut ui = super::interfaz::InterfazGpu::nueva(ctx, capa.shaders, FORMATO_RGBA8)?;
    if let Some(atlas) = capa.atlas {
        ui.fuente(ctx, atlas.ancho, atlas.alto, atlas.cobertura)?;
    }
    ui.actualizar(capa.vertices, capa.area)?;
    let dibujo = ui.juego(0)?;
    let escena = Escena {
        pipeline: &pipeline,
        malla: &malla,
        constantes,
        indirecto: None,
        conjunto: None,
        historial: None,
        campos: None,
        calculo: None,
    };
    capturar(ctx, &escena, &dibujo, tamano, fondo)
}

pub(super) fn capturar(
    ctx: &Contexto,
    escena: &Escena,
    interfaz: &DibujoInterfaz,
    tamano: (u32, u32),
    fondo: [f32; 4],
) -> Result<crate::CapturaEstudio, String> {
    let (ancho, alto) = tamano;
    if ancho == 0 || alto == 0 || ancho > 4096 || alto > 4096 {
        return Err("captura de estudio fuera de 1..=4096 por lado".to_owned());
    }
    let color = Imagen::nueva(ctx, FORMATO_RGBA8, tamano, USO_COLOR_ATTACHMENT | USO_TRANSFER_SRC, ASPECTO_COLOR)?;
    let adjuntos = super::multimuestreo::Adjuntos::nuevos(ctx, FORMATO_RGBA8, tamano)?;
    let bytes = u64::from(ancho) * u64::from(alto) * 4;
    let lectura = Buffer::nuevo(ctx, bytes, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
    let fd = ctx.fd.ok_or("falta el dispositivo")?;
    let mut medicion = Recursos::nuevo(&fd, ctx.dispositivo);
    let medir = ctx.bits_tiempo > 0;
    let _ = medicion.preparar_consultas(ctx, if medir { 2 } else { 0 })?;
    let destino = Destino {
        imagen: color.objeto,
        vista: color.vista,
        profundidad: (adjuntos.profundidad.objeto, adjuntos.profundidad.vista),
        multimuestreo: adjuntos.color(),
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
            if medir {
                (fd.calculo.reiniciar_consultas)(cmd, medicion.consultas, 0, 2);
                (fd.calculo.marcar_tiempo)(cmd, super::tipos_calculo::ETAPA_TOP_OF_PIPE, medicion.consultas, 0);
            }
            grabar(fd, cmd, &destino, fondo, Some(escena), Some(interfaz));
            if medir {
                (fd.calculo.marcar_tiempo)(cmd, super::tipos::ETAPA_BOTTOM_OF_PIPE, medicion.consultas, 1);
            }
            (fd.grafico.copiar_imagen)(cmd, color.objeto, LAYOUT_TRANSFER_SRC, lectura.objeto, 1, &raw const copia);
            let nulo = core::ptr::null();
            // GPU: publica la copia de píxeles para la lectura diagnóstica del host.
            (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, (&raw const host).cast(), 0, nulo, 0, nulo.cast());
        }
    })?;
    let nanosegundos = if medir {
        let mut tics = [0u64; 2];
        // SAFETY: una_vez completó su valla; ambas consultas fueron escritas y caben en tics.
        unsafe {
            ok(
                (fd.calculo.leer_consultas)(
                    ctx.dispositivo,
                    medicion.consultas,
                    0,
                    2,
                    16,
                    tics.as_mut_ptr().cast(),
                    8,
                    super::tipos_calculo::RESULTADO_64,
                ),
                "leer tiempos de render",
            )?;
        }
        Some(crate::eleccion::nanosegundos(tics[0], tics[1], ctx.bits_tiempo, ctx.periodo_ns))
    } else {
        None
    };
    let pixeles = lectura.leer(usize::try_from(bytes).map_err(|_| "captura demasiado grande")?)?;
    Ok(crate::CapturaEstudio { pixeles, nanosegundos, muestras: adjuntos.muestras })
}
