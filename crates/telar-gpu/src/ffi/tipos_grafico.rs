//! Vulkan a mano, la parte de dibujar: imágenes, vistas, el pipeline de
//! gráficos (con sus nueve estados), el renderizado dinámico de Vulkan 1.3 y
//! las copias de imagen a buffer. Copiado de `vulkan_core.h` (SDK 1.4.350) y
//! comprobado contra él.

use core::ffi::c_void;

use super::tipos::{Extension2D, Objeto, Rango};

pub(crate) const ST_IMAGE_CREATE_INFO: u32 = 14;
pub(crate) const ST_IMAGE_VIEW_CREATE_INFO: u32 = 15;
pub(crate) const ST_VERTEX_INPUT: u32 = 19;
pub(crate) const ST_INPUT_ASSEMBLY: u32 = 20;
pub(crate) const ST_VIEWPORT_STATE: u32 = 22;
pub(crate) const ST_RASTERIZATION: u32 = 23;
pub(crate) const ST_MULTISAMPLE: u32 = 24;
pub(crate) const ST_DEPTH_STENCIL: u32 = 25;
pub(crate) const ST_COLOR_BLEND: u32 = 26;
pub(crate) const ST_DYNAMIC_STATE: u32 = 27;
pub(crate) const ST_GRAPHICS_PIPELINE: u32 = 28;
pub(crate) const ST_VULKAN_1_3_FEATURES: u32 = 53;
pub(crate) const ST_RENDERING_INFO: u32 = 1_000_044_000;
pub(crate) const ST_RENDERING_ATTACHMENT_INFO: u32 = 1_000_044_001;
pub(crate) const ST_PIPELINE_RENDERING: u32 = 1_000_044_002;

pub(crate) const FORMATO_PROFUNDIDAD: u32 = 126; // D32_SFLOAT
pub(crate) const FORMATO_RGB32F: u32 = 106; // R32G32B32_SFLOAT: un vec3 por atributo
pub(crate) const FORMATO_RGBA8: u32 = 37; // R8G8B8A8_UNORM: la imagen de las capturas
pub(crate) const IMAGEN_2D: u32 = 1;
pub(crate) const VISTA_2D: u32 = 1;
pub(crate) const TESELADO_OPTIMO: u32 = 0;
pub(crate) const MUESTRAS_1: u32 = 1;
pub(crate) const USO_PROFUNDIDAD: u32 = 0x0000_0020;
pub(crate) const USO_TRANSFER_SRC: u32 = 0x0000_0001;
pub(crate) const ASPECTO_PROFUNDIDAD: u32 = 0x0000_0002;
pub(crate) const LAYOUT_COLOR: u32 = 2;
pub(crate) const LAYOUT_PROFUNDIDAD: u32 = 1_000_241_000;
pub(crate) const LAYOUT_TRANSFER_SRC: u32 = 6;
pub(crate) const CARGAR_LIMPIAR: u32 = 1;
pub(crate) const GUARDAR: u32 = 0;
pub(crate) const NO_GUARDAR: u32 = 1;
pub(crate) const TRIANGULOS: u32 = 3;
pub(crate) const RELLENAR: u32 = 0;
pub(crate) const DESCARTAR_TRASERAS: u32 = 0x0000_0002;
pub(crate) const FRENTE_ANTIHORARIO: u32 = 0;
pub(crate) const COMPARAR_MAYOR: u32 = 4;
pub(crate) const DINAMICO_VIEWPORT: u32 = 0;
pub(crate) const DINAMICO_TIJERA: u32 = 1;
pub(crate) const POR_VERTICE: u32 = 0;
pub(crate) const INDICES_U32: u32 = 1;
pub(crate) const ETAPA_VERTICES: u32 = 0x0000_0001;
pub(crate) const ETAPA_FRAGMENTOS: u32 = 0x0000_0010;
/// Etapa de pipeline; distinta de la máscara de visibilidad del shader.
pub(crate) const ETAPA_PIPELINE_FRAGMENTOS: u32 = 0x0000_0080;
pub(crate) const PUNTO_GRAFICOS: u32 = 0;
pub(crate) const BUFFER_VERTICES: u32 = 0x0000_0080;
pub(crate) const BUFFER_INDICES: u32 = 0x0000_0040;
pub(crate) const ETAPA_SALIDA_COLOR: u32 = 0x0000_0400;
pub(crate) const ETAPA_PRUEBAS_TEMPRANAS: u32 = 0x0000_0100;
pub(crate) const ETAPA_PRUEBAS_TARDIAS: u32 = 0x0000_0200;
pub(crate) const ETAPA_ENTRADA_VERTICES: u32 = 0x0000_0004;
pub(crate) const ACCESO_ESCRIBIR_COLOR: u32 = 0x0000_0100;
pub(crate) const ACCESO_LEER_PROFUNDIDAD: u32 = 0x0000_0200;
pub(crate) const ACCESO_ESCRIBIR_PROFUNDIDAD: u32 = 0x0000_0400;
pub(crate) const ACCESO_LEER_ATRIBUTOS: u32 = 0x0000_0004;
pub(crate) const ACCESO_LEER_INDICES: u32 = 0x0000_0002;
/// R | G | B | A.
pub(crate) const ESCRIBIR_RGBA: u32 = 0x0000_000F;

/// `VkExtent3D`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Extension3D {
    pub(crate) ancho: u32,
    pub(crate) alto: u32,
    pub(crate) fondo: u32,
}

/// `VkImageCreateInfo`.
#[repr(C)]
pub(crate) struct InfoImagen {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) tipo: u32,
    pub(crate) formato: u32,
    pub(crate) extension: Extension3D,
    pub(crate) mips: u32,
    pub(crate) capas: u32,
    pub(crate) muestras: u32,
    pub(crate) teselado: u32,
    pub(crate) uso: u32,
    pub(crate) compartir: u32,
    pub(crate) n_familias: u32,
    pub(crate) familias: *const u32,
    pub(crate) layout_inicial: u32,
}

/// `VkImageViewCreateInfo` (con `VkComponentMapping` identidad: cuatro ceros).
#[repr(C)]
pub(crate) struct InfoVista {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) imagen: Objeto,
    pub(crate) tipo: u32,
    pub(crate) formato: u32,
    pub(crate) componentes: [u32; 4],
    pub(crate) rango: Rango,
}

/// `VkVertexInputBindingDescription`.
#[repr(C)]
pub(crate) struct EnlaceVertices {
    pub(crate) numero: u32,
    pub(crate) paso: u32,
    pub(crate) ritmo: u32,
}

/// `VkVertexInputAttributeDescription`.
#[repr(C)]
pub(crate) struct AtributoVertice {
    pub(crate) ubicacion: u32,
    pub(crate) enlace: u32,
    pub(crate) formato: u32,
    pub(crate) desplazamiento: u32,
}

/// `VkPipelineVertexInputStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoVertices {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_enlaces: u32,
    pub(crate) enlaces: *const EnlaceVertices,
    pub(crate) n_atributos: u32,
    pub(crate) atributos: *const AtributoVertice,
}

/// `VkPipelineInputAssemblyStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoEnsamblado {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) topologia: u32,
    pub(crate) reinicio: u32,
}

/// `VkPipelineViewportStateCreateInfo` (viewport y tijera dinámicos: punteros nulos).
#[repr(C)]
pub(crate) struct EstadoViewport {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_viewports: u32,
    pub(crate) viewports: *const c_void,
    pub(crate) n_tijeras: u32,
    pub(crate) tijeras: *const c_void,
}

/// `VkPipelineRasterizationStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoRasterizado {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) limitar_profundidad: u32,
    pub(crate) descartar: u32,
    pub(crate) modo: u32,
    pub(crate) descartar_caras: u32,
    pub(crate) frente: u32,
    pub(crate) sesgo: u32,
    pub(crate) sesgo_constante: f32,
    pub(crate) sesgo_limite: f32,
    pub(crate) sesgo_pendiente: f32,
    pub(crate) grosor_linea: f32,
}

/// `VkPipelineMultisampleStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoMuestreo {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) muestras: u32,
    pub(crate) sombreado_por_muestra: u32,
    pub(crate) minimo: f32,
    pub(crate) mascara: *const u32,
    pub(crate) alfa_a_cobertura: u32,
    pub(crate) alfa_a_uno: u32,
}

/// `VkPipelineDepthStencilStateCreateInfo` (con los dos `VkStencilOpState`, 7 × u32 cada uno).
#[repr(C)]
pub(crate) struct EstadoProfundidad {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) prueba: u32,
    pub(crate) escritura: u32,
    pub(crate) comparacion: u32,
    pub(crate) limites: u32,
    pub(crate) plantilla: u32,
    pub(crate) plantilla_frente: [u32; 7],
    pub(crate) plantilla_detras: [u32; 7],
    pub(crate) minimo: f32,
    pub(crate) maximo: f32,
}

/// `VkPipelineColorBlendAttachmentState`.
#[repr(C)]
pub(crate) struct MezclaAdjunto {
    pub(crate) activa: u32,
    pub(crate) factores_color: [u32; 3],
    pub(crate) factores_alfa: [u32; 3],
    pub(crate) mascara: u32,
}

/// `VkPipelineColorBlendStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoMezcla {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) operacion_logica: u32,
    pub(crate) logica: u32,
    pub(crate) n_adjuntos: u32,
    pub(crate) adjuntos: *const MezclaAdjunto,
    pub(crate) constantes: [f32; 4],
}

/// `VkPipelineDynamicStateCreateInfo`.
#[repr(C)]
pub(crate) struct EstadoDinamico {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_estados: u32,
    pub(crate) estados: *const u32,
}

/// `VkPipelineRenderingCreateInfo`: con qué formatos dibuja (renderizado dinámico).
#[repr(C)]
pub(crate) struct InfoRenderizadoPipeline {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) mascara_vista: u32,
    pub(crate) n_colores: u32,
    pub(crate) formatos_color: *const u32,
    pub(crate) formato_profundidad: u32,
    pub(crate) formato_plantilla: u32,
}

/// `VkGraphicsPipelineCreateInfo`.
#[repr(C)]
pub(crate) struct InfoPipelineGrafico {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_etapas: u32,
    pub(crate) etapas: *const c_void,
    pub(crate) vertices: *const EstadoVertices,
    pub(crate) ensamblado: *const EstadoEnsamblado,
    pub(crate) teselado: *const c_void,
    pub(crate) viewport: *const EstadoViewport,
    pub(crate) rasterizado: *const EstadoRasterizado,
    pub(crate) muestreo: *const EstadoMuestreo,
    pub(crate) profundidad: *const EstadoProfundidad,
    pub(crate) mezcla: *const EstadoMezcla,
    pub(crate) dinamico: *const EstadoDinamico,
    pub(crate) disposicion: Objeto,
    pub(crate) pase: Objeto,
    pub(crate) subpase: u32,
    pub(crate) base: Objeto,
    pub(crate) indice_base: i32,
}

/// `VkPushConstantRange`.
#[repr(C)]
pub(crate) struct RangoConstantes {
    pub(crate) etapas: u32,
    pub(crate) desplazamiento: u32,
    pub(crate) tamano: u32,
}

/// `VkRenderingAttachmentInfo`. `limpiar` es el `VkClearValue` (16 bytes: el
/// color, o la profundidad en el primero).
#[repr(C)]
pub(crate) struct AdjuntoRenderizado {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) vista: Objeto,
    pub(crate) layout: u32,
    pub(crate) modo_resolver: u32,
    pub(crate) vista_resolver: Objeto,
    pub(crate) layout_resolver: u32,
    pub(crate) cargar: u32,
    pub(crate) guardar: u32,
    pub(crate) limpiar: [f32; 4],
}

/// `VkRenderingInfo`.
#[repr(C)]
pub(crate) struct InfoRenderizado {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) area: Rect2D,
    pub(crate) capas: u32,
    pub(crate) mascara_vista: u32,
    pub(crate) n_colores: u32,
    pub(crate) colores: *const AdjuntoRenderizado,
    pub(crate) profundidad: *const AdjuntoRenderizado,
    pub(crate) plantilla: *const AdjuntoRenderizado,
}

/// `VkRect2D`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Rect2D {
    pub(crate) origen: [i32; 2],
    pub(crate) extension: Extension2D,
}

/// `VkViewport`.
#[repr(C)]
pub(crate) struct Viewport {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) ancho: f32,
    pub(crate) alto: f32,
    pub(crate) profundidad_min: f32,
    pub(crate) profundidad_max: f32,
}

/// `VkBufferImageCopy` (con `VkImageSubresourceLayers`, `VkOffset3D` y `VkExtent3D`).
#[repr(C)]
pub(crate) struct CopiaImagen {
    pub(crate) desplazamiento: u64,
    pub(crate) largo_fila: u32,
    pub(crate) alto_imagen: u32,
    pub(crate) aspecto: u32,
    pub(crate) mip: u32,
    pub(crate) capa_base: u32,
    pub(crate) n_capas: u32,
    pub(crate) origen: [i32; 3],
    pub(crate) extension: Extension3D,
}

/// `VkPhysicalDeviceVulkan13Features`: 15 `VkBool32`, el 13.º es `dynamicRendering`.
#[repr(C)]
pub(crate) struct Funciones13 {
    pub(crate) s_type: u32,
    pub(crate) p_next: *mut c_void,
    pub(crate) activas: [u32; 15],
}

/// El índice de `dynamicRendering` en [`Funciones13::activas`].
pub(crate) const RENDERIZADO_DINAMICO: usize = 12;
