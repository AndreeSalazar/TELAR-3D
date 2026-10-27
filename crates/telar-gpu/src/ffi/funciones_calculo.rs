//! Las funciones de Vulkan para calcular: buffers, memoria, módulos de shader,
//! pipelines de cálculo, descriptores, despacho, copias y consultas de tiempo.

use core::ffi::c_void;

use super::tipos::{Asig, Comandos, Disp, Objeto};
use super::tipos_calculo::{
    Copia, EscrituraDescriptor, InfoBuffer, InfoConsultas, InfoDisposicion, InfoDisposicionPipeline, InfoMemoria,
    InfoModulo, InfoPipelineCalculo, InfoPoolDescriptores, InfoReservaConjunto, Requisitos,
};

// Las firmas, tal y como las da vulkan_core.h.
type Destruir = unsafe extern "system" fn(Disp, Objeto, Asig);
type CrearBuffer = unsafe extern "system" fn(Disp, *const InfoBuffer, Asig, *mut Objeto) -> i32;
type LeerRequisitos = unsafe extern "system" fn(Disp, Objeto, *mut Requisitos);
type ReservarMemoria = unsafe extern "system" fn(Disp, *const InfoMemoria, Asig, *mut Objeto) -> i32;
type UnirMemoria = unsafe extern "system" fn(Disp, Objeto, Objeto, u64) -> i32;
type Mapear = unsafe extern "system" fn(Disp, Objeto, u64, u64, u32, *mut *mut c_void) -> i32;
type Desmapear = unsafe extern "system" fn(Disp, Objeto);
type CrearModulo = unsafe extern "system" fn(Disp, *const InfoModulo, Asig, *mut Objeto) -> i32;
type CrearDisposicion = unsafe extern "system" fn(Disp, *const InfoDisposicion, Asig, *mut Objeto) -> i32;
type CrearDisposicionPipeline =
    unsafe extern "system" fn(Disp, *const InfoDisposicionPipeline, Asig, *mut Objeto) -> i32;
type CrearPipelines =
    unsafe extern "system" fn(Disp, Objeto, u32, *const InfoPipelineCalculo, Asig, *mut Objeto) -> i32;
type CrearPoolDescriptores = unsafe extern "system" fn(Disp, *const InfoPoolDescriptores, Asig, *mut Objeto) -> i32;
type ReservarConjuntos = unsafe extern "system" fn(Disp, *const InfoReservaConjunto, *mut Objeto) -> i32;
type EscribirDescriptores = unsafe extern "system" fn(Disp, u32, *const EscrituraDescriptor, u32, *const c_void);
type UsarPipeline = unsafe extern "system" fn(Comandos, u32, Objeto);
type UsarConjuntos = unsafe extern "system" fn(Comandos, u32, Objeto, u32, u32, *const Objeto, u32, *const u32);
type Despachar = unsafe extern "system" fn(Comandos, u32, u32, u32);
type DespacharIndirecto = unsafe extern "system" fn(Comandos, Objeto, u64);
type CopiarBuffer = unsafe extern "system" fn(Comandos, Objeto, Objeto, u32, *const Copia);
type CrearConsultas = unsafe extern "system" fn(Disp, *const InfoConsultas, Asig, *mut Objeto) -> i32;
type ReiniciarConsultas = unsafe extern "system" fn(Comandos, Objeto, u32, u32);
type MarcarTiempo = unsafe extern "system" fn(Comandos, u32, Objeto, u32);
type LeerConsultas = unsafe extern "system" fn(Disp, Objeto, u32, u32, usize, *mut c_void, u64, u32) -> i32;

/// Las funciones de cálculo del dispositivo.
#[derive(Clone, Copy)]
pub(crate) struct FnCalculo {
    pub(crate) crear_buffer: CrearBuffer,
    pub(crate) destruir_buffer: Destruir,
    pub(crate) requisitos: LeerRequisitos,
    pub(crate) reservar_memoria: ReservarMemoria,
    pub(crate) liberar_memoria: Destruir,
    pub(crate) unir_memoria: UnirMemoria,
    pub(crate) mapear: Mapear,
    pub(crate) desmapear: Desmapear,
    pub(crate) crear_modulo: CrearModulo,
    pub(crate) destruir_modulo: Destruir,
    pub(crate) crear_disposicion: CrearDisposicion,
    pub(crate) destruir_disposicion: Destruir,
    pub(crate) crear_disposicion_pipeline: CrearDisposicionPipeline,
    pub(crate) destruir_disposicion_pipeline: Destruir,
    pub(crate) crear_pipelines: CrearPipelines,
    pub(crate) destruir_pipeline: Destruir,
    pub(crate) crear_pool_descriptores: CrearPoolDescriptores,
    pub(crate) destruir_pool_descriptores: Destruir,
    pub(crate) reservar_conjuntos: ReservarConjuntos,
    pub(crate) escribir_descriptores: EscribirDescriptores,
    pub(crate) usar_pipeline: UsarPipeline,
    pub(crate) usar_conjuntos: UsarConjuntos,
    pub(crate) despachar: Despachar,
    pub(crate) despachar_indirecto: DespacharIndirecto,
    pub(crate) copiar_buffer: CopiarBuffer,
    pub(crate) crear_consultas: CrearConsultas,
    pub(crate) destruir_consultas: Destruir,
    pub(crate) reiniciar_consultas: ReiniciarConsultas,
    pub(crate) marcar_tiempo: MarcarTiempo,
    pub(crate) leer_consultas: LeerConsultas,
}

impl FnCalculo {
    /// `buscar`: nombre terminado en cero → puntero (`vkGetDeviceProcAddr`).
    pub(crate) fn cargar(buscar: impl Fn(&[u8]) -> Option<super::FnVacia>) -> Result<Self, String> {
        Ok(Self {
            crear_buffer: cargar!(buscar, "vkCreateBuffer", CrearBuffer),
            destruir_buffer: cargar!(buscar, "vkDestroyBuffer", Destruir),
            requisitos: cargar!(buscar, "vkGetBufferMemoryRequirements", LeerRequisitos),
            reservar_memoria: cargar!(buscar, "vkAllocateMemory", ReservarMemoria),
            liberar_memoria: cargar!(buscar, "vkFreeMemory", Destruir),
            unir_memoria: cargar!(buscar, "vkBindBufferMemory", UnirMemoria),
            mapear: cargar!(buscar, "vkMapMemory", Mapear),
            desmapear: cargar!(buscar, "vkUnmapMemory", Desmapear),
            crear_modulo: cargar!(buscar, "vkCreateShaderModule", CrearModulo),
            destruir_modulo: cargar!(buscar, "vkDestroyShaderModule", Destruir),
            crear_disposicion: cargar!(buscar, "vkCreateDescriptorSetLayout", CrearDisposicion),
            destruir_disposicion: cargar!(buscar, "vkDestroyDescriptorSetLayout", Destruir),
            crear_disposicion_pipeline: cargar!(buscar, "vkCreatePipelineLayout", CrearDisposicionPipeline),
            destruir_disposicion_pipeline: cargar!(buscar, "vkDestroyPipelineLayout", Destruir),
            crear_pipelines: cargar!(buscar, "vkCreateComputePipelines", CrearPipelines),
            destruir_pipeline: cargar!(buscar, "vkDestroyPipeline", Destruir),
            crear_pool_descriptores: cargar!(buscar, "vkCreateDescriptorPool", CrearPoolDescriptores),
            destruir_pool_descriptores: cargar!(buscar, "vkDestroyDescriptorPool", Destruir),
            reservar_conjuntos: cargar!(buscar, "vkAllocateDescriptorSets", ReservarConjuntos),
            escribir_descriptores: cargar!(buscar, "vkUpdateDescriptorSets", EscribirDescriptores),
            usar_pipeline: cargar!(buscar, "vkCmdBindPipeline", UsarPipeline),
            usar_conjuntos: cargar!(buscar, "vkCmdBindDescriptorSets", UsarConjuntos),
            despachar: cargar!(buscar, "vkCmdDispatch", Despachar),
            despachar_indirecto: cargar!(buscar, "vkCmdDispatchIndirect", DespacharIndirecto),
            copiar_buffer: cargar!(buscar, "vkCmdCopyBuffer", CopiarBuffer),
            crear_consultas: cargar!(buscar, "vkCreateQueryPool", CrearConsultas),
            destruir_consultas: cargar!(buscar, "vkDestroyQueryPool", Destruir),
            reiniciar_consultas: cargar!(buscar, "vkCmdResetQueryPool", ReiniciarConsultas),
            marcar_tiempo: cargar!(buscar, "vkCmdWriteTimestamp", MarcarTiempo),
            leer_consultas: cargar!(buscar, "vkGetQueryPoolResults", LeerConsultas),
        })
    }
}
