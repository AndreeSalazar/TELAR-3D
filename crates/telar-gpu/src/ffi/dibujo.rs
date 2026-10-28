//! Grabar un fotograma con renderizado dinámico: preparar las imágenes de color
//! y de profundidad, limpiarlas, dibujar la escena (si hay) y dejar la de color
//! lista para lo que venga después (presentarla o copiarla). Lo usan igual la
//! ventana y las capturas.

use core::ffi::c_void;

use super::funciones::FnDispositivo;
use super::interfaz::DibujoInterfaz;
use super::malla_gpu::MallaGpu;
use super::pipeline_grafico::{PipelineGrafico, TAMANO_CONSTANTES};
use super::residente::Ranura;
use super::tipos::{
    ASPECTO_COLOR, BarreraImagen, Comandos, Extension2D, FAMILIA_IGNORADA, LAYOUT_UNDEFINED, Objeto, Rango,
    ST_IMAGE_MEMORY_BARRIER,
};
use super::tipos_grafico::{
    ACCESO_ESCRIBIR_COLOR, ACCESO_ESCRIBIR_PROFUNDIDAD, ACCESO_LEER_PROFUNDIDAD, ASPECTO_PROFUNDIDAD,
    AdjuntoRenderizado, CARGAR_LIMPIAR, ETAPA_FRAGMENTOS, ETAPA_PRUEBAS_TARDIAS, ETAPA_PRUEBAS_TEMPRANAS,
    ETAPA_SALIDA_COLOR, ETAPA_VERTICES, GUARDAR, INDICES_U32, InfoRenderizado, LAYOUT_COLOR, LAYOUT_PROFUNDIDAD,
    NO_GUARDAR, PUNTO_GRAFICOS, Rect2D, ST_RENDERING_ATTACHMENT_INFO, ST_RENDERING_INFO, Viewport,
};
use crate::ParametrosNodos;

const NULO: *const c_void = core::ptr::null();

/// Dónde se dibuja.
pub(crate) struct Destino {
    pub(crate) imagen: Objeto,
    pub(crate) vista: Objeto,
    /// `(imagen, vista)` de la profundidad, del mismo tamaño.
    pub(crate) profundidad: (Objeto, Objeto),
    /// Color multimuestreado opcional; se resuelve a imagen/vista antes de presentar.
    pub(crate) multimuestreo: Option<(Objeto, Objeto)>,
    pub(crate) extension: (u32, u32),
    /// Cómo queda la imagen de color al acabar: `(layout, etapa, acceso)` de lo
    /// que la use después.
    pub(crate) despues: (u32, u32, u32),
}

/// Qué se dibuja: un pipeline, una malla y sus constantes.
pub(crate) struct Escena<'a> {
    pub(crate) pipeline: &'a PipelineGrafico,
    pub(crate) malla: &'a MallaGpu,
    pub(crate) constantes: &'a [f32; 32],
    pub(crate) indirecto: Option<Objeto>,
    pub(crate) conjunto: Option<Objeto>,
    pub(crate) historial: Option<&'a super::historial::Historial>,
    pub(crate) calculo: Option<(&'a Ranura, ParametrosNodos)>,
}

/// Graba en `cmd` el fotograma: limpia a `limpiar` (color ya en el espacio del
/// formato) y a profundidad 0, dibuja `escena` y deja la imagen como dice `destino`.
///
/// # Safety
///
/// `cmd` tiene que estar grabando, y todo lo que nombran `destino` y `escena`
/// tiene que existir hasta que la GPU termine estos comandos. El pipeline tiene
/// que ser del formato de `destino`.
pub(crate) unsafe fn grabar(
    fd: &FnDispositivo,
    cmd: Comandos,
    destino: &Destino,
    limpiar: [f32; 4],
    escena: Option<&Escena>,
    interfaz: Option<&DibujoInterfaz>,
) {
    if let Some(historial) = escena.and_then(|e| e.historial) {
        // SAFETY: contrato de grabar; escena e historial viven hasta la valla del envío.
        unsafe {
            historial.grabar(fd, cmd);
        }
    }
    let color = Rango { aspecto: ASPECTO_COLOR, mip_base: 0, n_mips: 1, capa_base: 0, n_capas: 1 };
    let profundidad = Rango { aspecto: ASPECTO_PROFUNDIDAD, ..color };
    let (imagen_prof, vista_prof) = destino.profundidad;
    let pruebas = ETAPA_PRUEBAS_TEMPRANAS | ETAPA_PRUEBAS_TARDIAS;
    let leer_escribir = ACCESO_LEER_PROFUNDIDAD | ACCESO_ESCRIBIR_PROFUNDIDAD;
    // El contenido de antes no importa (se limpia), pero la profundidad la comparten
    // los fotogramas en vuelo: el de ahora espera a que el anterior acabe de escribirla.
    let imagen_msaa = destino.multimuestreo.map_or(destino.imagen, |(imagen, _)| imagen);
    let antes = [
        barrera(destino.imagen, color, (0, ACCESO_ESCRIBIR_COLOR), (LAYOUT_UNDEFINED, LAYOUT_COLOR)),
        barrera(
            imagen_prof,
            profundidad,
            (ACCESO_ESCRIBIR_PROFUNDIDAD, leer_escribir),
            (LAYOUT_UNDEFINED, LAYOUT_PROFUNDIDAD),
        ),
        // GPU: el adjunto compartido espera las escrituras de color anteriores.
        barrera(imagen_msaa, color, (ACCESO_ESCRIBIR_COLOR, ACCESO_ESCRIBIR_COLOR), (LAYOUT_UNDEFINED, LAYOUT_COLOR)),
    ];
    let (layout, etapa, acceso) = destino.despues;
    let despues = barrera(destino.imagen, color, (ACCESO_ESCRIBIR_COLOR, acceso), (LAYOUT_COLOR, layout));
    let mut adjunto_color = adjunto(destino.vista, LAYOUT_COLOR, GUARDAR, limpiar);
    if let Some((_, vista)) = destino.multimuestreo {
        adjunto_color.vista = vista;
        adjunto_color.guardar = NO_GUARDAR;
        adjunto_color.modo_resolver = 2; // VK_RESOLVE_MODE_AVERAGE_BIT.
        adjunto_color.vista_resolver = destino.vista;
        adjunto_color.layout_resolver = LAYOUT_COLOR;
    }
    // Profundidad invertida: 0 es el infinito. No se guarda: solo vale durante el fotograma.
    let adjunto_prof = adjunto(vista_prof, LAYOUT_PROFUNDIDAD, NO_GUARDAR, [0.0; 4]);
    let (ancho, alto) = destino.extension;
    let area = Rect2D { origen: [0, 0], extension: Extension2D { ancho, alto } };
    let info = InfoRenderizado {
        s_type: ST_RENDERING_INFO,
        p_next: NULO,
        flags: 0,
        area,
        capas: 1,
        mascara_vista: 0,
        n_colores: 1,
        colores: &raw const adjunto_color,
        profundidad: &raw const adjunto_prof,
        plantilla: core::ptr::null(),
    };
    #[allow(clippy::cast_precision_loss, reason = "el tamaño de una imagen cabe de sobra en un f32")]
    let viewport =
        Viewport { x: 0.0, y: 0.0, ancho: ancho as f32, alto: alto as f32, profundidad_min: 0.0, profundidad_max: 1.0 };
    let (fg, etapas_antes) = (fd.grafico, ETAPA_SALIDA_COLOR | pruebas);
    // SAFETY: lo garantiza quien llama (ver arriba); todo lo que se pasa por
    // puntero vive hasta el final de la función.
    unsafe {
        (fd.barrera)(
            cmd,
            etapas_antes,
            etapas_antes,
            0,
            0,
            NULO,
            0,
            NULO,
            if destino.multimuestreo.is_some() { 3 } else { 2 },
            antes.as_ptr(),
        );
        (fg.empezar_renderizado)(cmd, &raw const info);
        if let Some(e) = escena {
            let (viewport_escena, tijera_escena) =
                interfaz.map_or_else(|| region([0, 0, ancho, alto]), |ui| region(ui.area));
            dibujar_escena(fd, cmd, e, &viewport_escena, tijera_escena);
        }
        if let Some(ui) = interfaz {
            (fd.calculo.usar_pipeline)(cmd, PUNTO_GRAFICOS, ui.pipeline.pipeline);
            (fd.calculo.usar_conjuntos)(
                cmd,
                PUNTO_GRAFICOS,
                ui.pipeline.disposicion,
                0,
                1,
                &raw const ui.conjunto,
                0,
                NULO.cast(),
            );
            (fg.poner_viewport)(cmd, 0, 1, &raw const viewport);
            (fg.poner_tijera)(cmd, 0, 1, &raw const area);
            let desde = 0;
            (fg.usar_vertices)(cmd, 0, 1, &raw const ui.buffer.buffer.objeto, &raw const desde);
            (fg.dibujar)(cmd, ui.cantidad, 1, 0, 0);
        }
        (fg.terminar_renderizado)(cmd);
        (fd.barrera)(cmd, ETAPA_SALIDA_COLOR, etapa, 0, 0, NULO, 0, NULO, 1, &raw const despues);
    }
}

#[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana representables como viewport")]
fn region(area: [u32; 4]) -> (Viewport, Rect2D) {
    let [x, y, ancho, alto] = area;
    (
        Viewport {
            x: x as f32,
            y: y as f32,
            ancho: ancho as f32,
            alto: alto as f32,
            profundidad_min: 0.0,
            profundidad_max: 1.0,
        },
        Rect2D {
            origen: [i32::try_from(x).unwrap_or(0), i32::try_from(y).unwrap_or(0)],
            extension: Extension2D { ancho, alto },
        },
    )
}

/// Una barrera de imagen: `(acceso antes, acceso después)` y `(layout viejo, nuevo)`.
pub(crate) fn barrera(imagen: Objeto, rango: Rango, acceso: (u32, u32), layout: (u32, u32)) -> BarreraImagen {
    BarreraImagen {
        s_type: ST_IMAGE_MEMORY_BARRIER,
        p_next: NULO,
        acceso_origen: acceso.0,
        acceso_destino: acceso.1,
        layout_viejo: layout.0,
        layout_nuevo: layout.1,
        familia_origen: FAMILIA_IGNORADA,
        familia_destino: FAMILIA_IGNORADA,
        imagen,
        rango,
    }
}

/// # Safety
/// El comando graba y todos los recursos de la escena viven hasta completar su valla.
unsafe fn dibujar_escena(
    fd: &FnDispositivo,
    cmd: Comandos,
    e: &Escena,
    viewport_escena: &Viewport,
    tijera_escena: Rect2D,
) {
    let fg = fd.grafico;
    // SAFETY: quien llama garantiza vida útil, formato y grabación del comando.
    unsafe {
        (fd.calculo.usar_pipeline)(cmd, PUNTO_GRAFICOS, e.pipeline.pipeline);
        (fg.poner_viewport)(cmd, 0, 1, core::ptr::from_ref(viewport_escena));
        (fg.poner_tijera)(cmd, 0, 1, &raw const tijera_escena);
        let etapas = ETAPA_VERTICES | ETAPA_FRAGMENTOS;
        let constantes = e.constantes.as_ptr().cast();
        (fg.empujar_constantes)(cmd, e.pipeline.disposicion, etapas, 0, TAMANO_CONSTANTES, constantes);
        if let Some(conjunto) = e.conjunto {
            (fd.calculo.usar_conjuntos)(
                cmd,
                PUNTO_GRAFICOS,
                e.pipeline.disposicion,
                0,
                1,
                &raw const conjunto,
                0,
                NULO.cast(),
            );
        }
        let desde = 0u64;
        (fg.usar_vertices)(cmd, 0, 1, &raw const e.malla.vertices.objeto, &raw const desde);
        (fg.usar_indices)(cmd, e.malla.indices.objeto, 0, INDICES_U32);
        if let Some(argumentos) = e.indirecto {
            (fg.dibujar_indirecto)(cmd, argumentos, 12, 1, 20);
        } else {
            (fg.dibujar_indices)(cmd, e.malla.n_indices, 1, 0, 0, 0);
        }
    }
}

fn adjunto(vista: Objeto, layout: u32, guardar: u32, limpiar: [f32; 4]) -> AdjuntoRenderizado {
    AdjuntoRenderizado {
        s_type: ST_RENDERING_ATTACHMENT_INFO,
        p_next: NULO,
        vista,
        layout,
        modo_resolver: 0,
        vista_resolver: 0,
        layout_resolver: 0,
        cargar: CARGAR_LIMPIAR,
        guardar,
        limpiar,
    }
}
