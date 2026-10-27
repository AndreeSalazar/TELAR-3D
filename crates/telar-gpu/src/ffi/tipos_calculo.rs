//! Vulkan a mano, la parte de calcular: buffers, memoria, pipelines de cálculo,
//! descriptores y consultas de tiempo. Copiado de `vulkan_core.h` (SDK 1.4.350)
//! y comprobado contra él, como `tipos.rs`.

use core::ffi::c_void;

use super::tipos::Objeto;

pub(crate) const ST_MEMORY_ALLOCATE_INFO: u32 = 5;
pub(crate) const ST_QUERY_POOL_CREATE_INFO: u32 = 11;
pub(crate) const ST_BUFFER_CREATE_INFO: u32 = 12;
pub(crate) const ST_SHADER_MODULE_CREATE_INFO: u32 = 16;
pub(crate) const ST_PIPELINE_SHADER_STAGE_CREATE_INFO: u32 = 18;
pub(crate) const ST_COMPUTE_PIPELINE_CREATE_INFO: u32 = 29;
pub(crate) const ST_PIPELINE_LAYOUT_CREATE_INFO: u32 = 30;
pub(crate) const ST_DESCRIPTOR_SET_LAYOUT_CREATE_INFO: u32 = 32;
pub(crate) const ST_DESCRIPTOR_POOL_CREATE_INFO: u32 = 33;
pub(crate) const ST_DESCRIPTOR_SET_ALLOCATE_INFO: u32 = 34;
pub(crate) const ST_WRITE_DESCRIPTOR_SET: u32 = 35;
pub(crate) const ST_MEMORY_BARRIER: u32 = 46;

pub(crate) const BUFFER_TRANSFER_SRC: u32 = 0x0000_0001;
pub(crate) const BUFFER_TRANSFER_DST: u32 = 0x0000_0002;
pub(crate) const BUFFER_ALMACEN: u32 = 0x0000_0020;
pub(crate) const BUFFER_INDIRECTO: u32 = 0x0000_0100;
pub(crate) const ETAPA_INDIRECTO: u32 = 0x0000_0002;
pub(crate) const ACCESO_INDIRECTO: u32 = 0x0000_0001;
pub(crate) const ACCESO_SHADER_READ: u32 = 0x0000_0020;
pub(crate) const DESCRIPTOR_ALMACEN: u32 = 7;
pub(crate) const ETAPA_SHADER_CALCULO: u32 = 0x0000_0020;
pub(crate) const PUNTO_CALCULO: u32 = 1;
pub(crate) const CONSULTA_TIEMPO: u32 = 2;
pub(crate) const RESULTADO_64: u32 = 0x0000_0001;
pub(crate) const RESULTADO_ESPERAR: u32 = 0x0000_0002;
pub(crate) const ETAPA_TOP_OF_PIPE: u32 = 0x0000_0001;
pub(crate) const ETAPA_CALCULO: u32 = 0x0000_0800;
pub(crate) const ETAPA_HOST: u32 = 0x0000_4000;
pub(crate) const ACCESO_SHADER_WRITE: u32 = 0x0000_0040;
pub(crate) const ACCESO_TRANSFER_READ: u32 = 0x0000_0800;
pub(crate) const ACCESO_HOST_READ: u32 = 0x0000_2000;

/// `VkBufferCreateInfo`.
#[repr(C)]
pub(crate) struct InfoBuffer {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) tamano: u64,
    pub(crate) uso: u32,
    pub(crate) compartir: u32,
    pub(crate) n_familias: u32,
    pub(crate) familias: *const u32,
}

/// `VkMemoryRequirements`.
#[repr(C)]
#[derive(Default)]
pub(crate) struct Requisitos {
    pub(crate) tamano: u64,
    pub(crate) alineacion: u64,
    pub(crate) tipos_permitidos: u32,
}

/// `VkMemoryAllocateInfo`.
#[repr(C)]
pub(crate) struct InfoMemoria {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) tamano: u64,
    pub(crate) tipo: u32,
}

/// `VkMemoryType`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub(crate) struct TipoMemoria {
    pub(crate) propiedades: u32,
    pub(crate) monton: u32,
}

/// `VkMemoryHeap`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub(crate) struct Monton {
    pub(crate) tamano: u64,
    pub(crate) flags: u32,
}

/// `VkPhysicalDeviceMemoryProperties` (32 tipos y 16 montones como máximo).
#[repr(C)]
pub(crate) struct PropiedadesMemoria {
    pub(crate) n_tipos: u32,
    pub(crate) tipos: [TipoMemoria; 32],
    pub(crate) n_montones: u32,
    pub(crate) montones: [Monton; 16],
}

/// `VkShaderModuleCreateInfo`: el tamaño va en **bytes**.
#[repr(C)]
pub(crate) struct InfoModulo {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) tamano: usize,
    pub(crate) codigo: *const u32,
}

/// `VkDescriptorSetLayoutBinding`.
#[repr(C)]
pub(crate) struct Enlace {
    pub(crate) numero: u32,
    pub(crate) tipo: u32,
    pub(crate) cantidad: u32,
    pub(crate) etapas: u32,
    pub(crate) muestreadores: *const c_void,
}

/// `VkDescriptorSetLayoutCreateInfo`.
#[repr(C)]
pub(crate) struct InfoDisposicion {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_enlaces: u32,
    pub(crate) enlaces: *const Enlace,
}

/// `VkPipelineLayoutCreateInfo`.
#[repr(C)]
pub(crate) struct InfoDisposicionPipeline {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_conjuntos: u32,
    pub(crate) conjuntos: *const Objeto,
    pub(crate) n_constantes: u32,
    pub(crate) constantes: *const c_void,
}

/// `VkPipelineShaderStageCreateInfo`.
#[repr(C)]
pub(crate) struct InfoEtapa {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) etapa: u32,
    pub(crate) modulo: Objeto,
    pub(crate) nombre: *const u8,
    pub(crate) especializacion: *const c_void,
}

/// `VkComputePipelineCreateInfo` (la etapa va dentro, no por puntero).
#[repr(C)]
pub(crate) struct InfoPipelineCalculo {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) etapa: InfoEtapa,
    pub(crate) disposicion: Objeto,
    pub(crate) base: Objeto,
    pub(crate) indice_base: i32,
}

/// `VkDescriptorPoolSize`.
#[repr(C)]
pub(crate) struct TamanoPool {
    pub(crate) tipo: u32,
    pub(crate) cantidad: u32,
}

/// `VkDescriptorPoolCreateInfo`.
#[repr(C)]
pub(crate) struct InfoPoolDescriptores {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) max_conjuntos: u32,
    pub(crate) n_tamanos: u32,
    pub(crate) tamanos: *const TamanoPool,
}

/// `VkDescriptorSetAllocateInfo`.
#[repr(C)]
pub(crate) struct InfoReservaConjunto {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) pool: Objeto,
    pub(crate) n_conjuntos: u32,
    pub(crate) disposiciones: *const Objeto,
}

/// `VkDescriptorBufferInfo`.
#[repr(C)]
pub(crate) struct InfoBufferDescriptor {
    pub(crate) buffer: Objeto,
    pub(crate) desplazamiento: u64,
    pub(crate) rango: u64,
}

/// `VkWriteDescriptorSet`.
#[repr(C)]
pub(crate) struct EscrituraDescriptor {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) conjunto: Objeto,
    pub(crate) enlace: u32,
    pub(crate) elemento: u32,
    pub(crate) cantidad: u32,
    pub(crate) tipo: u32,
    pub(crate) imagenes: *const c_void,
    pub(crate) buffers: *const InfoBufferDescriptor,
    pub(crate) vistas: *const c_void,
}

/// `VkQueryPoolCreateInfo`.
#[repr(C)]
pub(crate) struct InfoConsultas {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) tipo: u32,
    pub(crate) cantidad: u32,
    pub(crate) estadisticas: u32,
}

/// `VkMemoryBarrier`.
#[repr(C)]
pub(crate) struct BarreraMemoria {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) acceso_origen: u32,
    pub(crate) acceso_destino: u32,
}

/// `VkBufferCopy`.
#[repr(C)]
pub(crate) struct Copia {
    pub(crate) origen: u64,
    pub(crate) destino: u64,
    pub(crate) tamano: u64,
}
