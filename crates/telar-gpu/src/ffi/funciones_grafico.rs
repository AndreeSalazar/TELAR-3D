//! Las funciones de Vulkan para dibujar: imágenes y vistas, el pipeline de
//! gráficos, el renderizado dinámico (Vulkan 1.3) y los comandos de dibujo.

use core::ffi::c_void;

use super::tipos::{Asig, Comandos, Disp, Objeto};
use super::tipos_calculo::Requisitos;
use super::tipos_grafico::{
    CopiaImagen, InfoImagen, InfoPipelineGrafico, InfoRenderizado, InfoVista, Rect2D, Viewport,
};

// Las firmas, tal y como las da vulkan_core.h.
type Destruir = unsafe extern "system" fn(Disp, Objeto, Asig);
type CrearImagen = unsafe extern "system" fn(Disp, *const InfoImagen, Asig, *mut Objeto) -> i32;
type LeerRequisitos = unsafe extern "system" fn(Disp, Objeto, *mut Requisitos);
type UnirMemoria = unsafe extern "system" fn(Disp, Objeto, Objeto, u64) -> i32;
type CrearVista = unsafe extern "system" fn(Disp, *const InfoVista, Asig, *mut Objeto) -> i32;
type CrearPipelines =
    unsafe extern "system" fn(Disp, Objeto, u32, *const InfoPipelineGrafico, Asig, *mut Objeto) -> i32;
type EmpezarRenderizado = unsafe extern "system" fn(Comandos, *const InfoRenderizado);
type TerminarRenderizado = unsafe extern "system" fn(Comandos);
type PonerViewport = unsafe extern "system" fn(Comandos, u32, u32, *const Viewport);
type PonerTijera = unsafe extern "system" fn(Comandos, u32, u32, *const Rect2D);
type EmpujarConstantes = unsafe extern "system" fn(Comandos, Objeto, u32, u32, u32, *const c_void);
type UsarVertices = unsafe extern "system" fn(Comandos, u32, u32, *const Objeto, *const u64);
type UsarIndices = unsafe extern "system" fn(Comandos, Objeto, u64, u32);
type DibujarIndices = unsafe extern "system" fn(Comandos, u32, u32, u32, i32, u32);
type DibujarIndirecto = unsafe extern "system" fn(Comandos, Objeto, u64, u32, u32);
type CopiarImagen = unsafe extern "system" fn(Comandos, Objeto, u32, Objeto, u32, *const CopiaImagen);

/// Las funciones de dibujo del dispositivo.
#[derive(Clone, Copy)]
pub(crate) struct FnGrafico {
    pub(crate) crear_imagen: CrearImagen,
    pub(crate) destruir_imagen: Destruir,
    pub(crate) requisitos_imagen: LeerRequisitos,
    pub(crate) unir_memoria_imagen: UnirMemoria,
    pub(crate) crear_vista: CrearVista,
    pub(crate) destruir_vista: Destruir,
    pub(crate) crear_pipelines: CrearPipelines,
    pub(crate) empezar_renderizado: EmpezarRenderizado,
    pub(crate) terminar_renderizado: TerminarRenderizado,
    pub(crate) poner_viewport: PonerViewport,
    pub(crate) poner_tijera: PonerTijera,
    pub(crate) empujar_constantes: EmpujarConstantes,
    pub(crate) usar_vertices: UsarVertices,
    pub(crate) usar_indices: UsarIndices,
    pub(crate) dibujar_indices: DibujarIndices,
    pub(crate) dibujar_indirecto: DibujarIndirecto,
    pub(crate) copiar_imagen: CopiarImagen,
}

impl FnGrafico {
    /// `buscar`: nombre terminado en cero → puntero (`vkGetDeviceProcAddr`).
    pub(crate) fn cargar(buscar: impl Fn(&[u8]) -> Option<super::FnVacia>) -> Result<Self, String> {
        Ok(Self {
            crear_imagen: cargar!(buscar, "vkCreateImage", CrearImagen),
            destruir_imagen: cargar!(buscar, "vkDestroyImage", Destruir),
            requisitos_imagen: cargar!(buscar, "vkGetImageMemoryRequirements", LeerRequisitos),
            unir_memoria_imagen: cargar!(buscar, "vkBindImageMemory", UnirMemoria),
            crear_vista: cargar!(buscar, "vkCreateImageView", CrearVista),
            destruir_vista: cargar!(buscar, "vkDestroyImageView", Destruir),
            crear_pipelines: cargar!(buscar, "vkCreateGraphicsPipelines", CrearPipelines),
            empezar_renderizado: cargar!(buscar, "vkCmdBeginRendering", EmpezarRenderizado),
            terminar_renderizado: cargar!(buscar, "vkCmdEndRendering", TerminarRenderizado),
            poner_viewport: cargar!(buscar, "vkCmdSetViewport", PonerViewport),
            poner_tijera: cargar!(buscar, "vkCmdSetScissor", PonerTijera),
            empujar_constantes: cargar!(buscar, "vkCmdPushConstants", EmpujarConstantes),
            usar_vertices: cargar!(buscar, "vkCmdBindVertexBuffers", UsarVertices),
            usar_indices: cargar!(buscar, "vkCmdBindIndexBuffer", UsarIndices),
            dibujar_indices: cargar!(buscar, "vkCmdDrawIndexed", DibujarIndices),
            dibujar_indirecto: cargar!(buscar, "vkCmdDrawIndexedIndirect", DibujarIndirecto),
            copiar_imagen: cargar!(buscar, "vkCmdCopyImageToBuffer", CopiarImagen),
        })
    }
}
