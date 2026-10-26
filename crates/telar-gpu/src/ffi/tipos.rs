//! Vulkan a mano: las estructuras y constantes que usa TELAR, copiadas de
//! `vulkan_core.h` y `vulkan_win32.h` (SDK 1.4.350) y comprobadas contra ellos.
//! Los manejadores despachables son punteros; los no despachables, `u64`.

use core::ffi::c_void;

pub(crate) type Instancia = *mut c_void;
pub(crate) type Fisico = *mut c_void;
pub(crate) type Disp = *mut c_void;
pub(crate) type Cola = *mut c_void;
pub(crate) type Comandos = *mut c_void;
/// Superficie, swapchain, imagen, semáforo, valla, pool…: `u64` en 64 bits.
pub(crate) type Objeto = u64;
/// `VkAllocationCallbacks*`: siempre null (el asignador de Vulkan).
pub(crate) type Asig = *const c_void;

pub(crate) const VK_SUCCESS: i32 = 0;
pub(crate) const VK_SUBOPTIMAL_KHR: i32 = 1_000_001_003;
pub(crate) const VK_ERROR_OUT_OF_DATE_KHR: i32 = -1_000_001_004;
/// `VK_API_VERSION_1_3`: `(1 << 22) | (3 << 12)`.
pub(crate) const VK_API_1_3: u32 = (1 << 22) | (3 << 12);

// VkStructureType
pub(crate) const ST_APPLICATION_INFO: u32 = 0;
pub(crate) const ST_INSTANCE_CREATE_INFO: u32 = 1;
pub(crate) const ST_DEVICE_QUEUE_CREATE_INFO: u32 = 2;
pub(crate) const ST_DEVICE_CREATE_INFO: u32 = 3;
pub(crate) const ST_SUBMIT_INFO: u32 = 4;
pub(crate) const ST_FENCE_CREATE_INFO: u32 = 8;
pub(crate) const ST_SEMAPHORE_CREATE_INFO: u32 = 9;
pub(crate) const ST_COMMAND_POOL_CREATE_INFO: u32 = 39;
pub(crate) const ST_COMMAND_BUFFER_ALLOCATE_INFO: u32 = 40;
pub(crate) const ST_COMMAND_BUFFER_BEGIN_INFO: u32 = 42;
pub(crate) const ST_IMAGE_MEMORY_BARRIER: u32 = 45;
pub(crate) const ST_SWAPCHAIN_CREATE_INFO_KHR: u32 = 1_000_001_000;
pub(crate) const ST_PRESENT_INFO_KHR: u32 = 1_000_001_001;
pub(crate) const ST_WIN32_SURFACE_CREATE_INFO_KHR: u32 = 1_000_009_000;
pub(crate) const ST_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT: u32 = 1_000_128_004;

pub(crate) const PRESENT_MODE_FIFO: u32 = 2;
pub(crate) const LAYOUT_UNDEFINED: u32 = 0;
pub(crate) const LAYOUT_TRANSFER_DST: u32 = 7;
pub(crate) const LAYOUT_PRESENT_SRC: u32 = 1_000_001_002;
pub(crate) const USO_TRANSFER_DST: u32 = 0x0000_0002;
pub(crate) const USO_COLOR_ATTACHMENT: u32 = 0x0000_0010;
pub(crate) const ETAPA_TRANSFER: u32 = 0x0000_1000;
pub(crate) const ETAPA_BOTTOM_OF_PIPE: u32 = 0x0000_2000;
pub(crate) const ACCESO_TRANSFER_WRITE: u32 = 0x0000_1000;
pub(crate) const ASPECTO_COLOR: u32 = 0x0000_0001;
pub(crate) const COLA_GRAFICOS: u32 = 0x0000_0001;
pub(crate) const ALFA_OPACO: u32 = 0x0000_0001;
pub(crate) const COMPARTIR_EXCLUSIVO: u32 = 0;
pub(crate) const NIVEL_PRIMARIO: u32 = 0;
pub(crate) const POOL_REINICIABLE: u32 = 0x0000_0002;
pub(crate) const VALLA_SENALADA: u32 = 0x0000_0001;
pub(crate) const USO_UNA_VEZ: u32 = 0x0000_0001;
pub(crate) const FAMILIA_IGNORADA: u32 = !0;
pub(crate) const GRAVEDAD_AVISO: u32 = 0x0000_0100;
pub(crate) const GRAVEDAD_ERROR: u32 = 0x0000_1000;
/// GENERAL | VALIDATION | PERFORMANCE.
pub(crate) const TIPOS_DE_MENSAJE: u32 = 0x0000_0007;
/// GENERAL: avisos del cargador y del sistema (capas instaladas por otros programas).
pub(crate) const TIPO_GENERAL: u32 = 0x0000_0001;

pub(crate) const CAPA_VALIDACION: &[u8] = b"VK_LAYER_KHRONOS_validation\0";
pub(crate) const EXT_SURFACE: &[u8] = b"VK_KHR_surface\0";
pub(crate) const EXT_WIN32_SURFACE: &[u8] = b"VK_KHR_win32_surface\0";
pub(crate) const EXT_DEBUG_UTILS: &[u8] = b"VK_EXT_debug_utils\0";
pub(crate) const EXT_SWAPCHAIN: &[u8] = b"VK_KHR_swapchain\0";

/// `VkExtent2D`.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Extension2D {
    pub(crate) ancho: u32,
    pub(crate) alto: u32,
}

/// `VkApplicationInfo`.
#[repr(C)]
pub(crate) struct InfoApp {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) nombre_app: *const u8,
    pub(crate) version_app: u32,
    pub(crate) nombre_motor: *const u8,
    pub(crate) version_motor: u32,
    pub(crate) version_api: u32,
}

/// `VkInstanceCreateInfo`.
#[repr(C)]
pub(crate) struct InfoInstancia {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) app: *const InfoApp,
    pub(crate) n_capas: u32,
    pub(crate) capas: *const *const u8,
    pub(crate) n_extensiones: u32,
    pub(crate) extensiones: *const *const u8,
}

/// `VkLayerProperties`.
#[repr(C)]
pub(crate) struct PropiedadesCapa {
    pub(crate) nombre: [u8; 256],
    pub(crate) version_spec: u32,
    pub(crate) version_impl: u32,
    pub(crate) descripcion: [u8; 256],
}

/// `PFN_vkDebugUtilsMessengerCallbackEXT`.
pub(crate) type LlamadaAviso = unsafe extern "system" fn(u32, u32, *const DatosAviso, *mut c_void) -> u32;

/// `VkDebugUtilsMessengerCreateInfoEXT`.
#[repr(C)]
pub(crate) struct InfoMensajero {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) gravedad: u32,
    pub(crate) tipos: u32,
    pub(crate) llamada: LlamadaAviso,
    pub(crate) usuario: *mut c_void,
}

/// `VkDebugUtilsMessengerCallbackDataEXT`.
#[repr(C)]
pub(crate) struct DatosAviso {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) id_nombre: *const core::ffi::c_char,
    pub(crate) id_numero: i32,
    pub(crate) mensaje: *const core::ffi::c_char,
    pub(crate) n_etiquetas_cola: u32,
    pub(crate) etiquetas_cola: *const c_void,
    pub(crate) n_etiquetas_comandos: u32,
    pub(crate) etiquetas_comandos: *const c_void,
    pub(crate) n_objetos: u32,
    pub(crate) objetos: *const c_void,
}

/// `VkWin32SurfaceCreateInfoKHR`.
#[repr(C)]
pub(crate) struct InfoSuperficieWin32 {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) hinstance: isize,
    pub(crate) hwnd: isize,
}

/// El principio de `VkPhysicalDeviceProperties` (apiVersion … deviceName), con
/// espacio de sobra para el resto (la estructura real mide 824 bytes).
#[repr(C, align(8))]
pub(crate) struct PropiedadesFisico {
    pub(crate) version_api: u32,
    pub(crate) version_driver: u32,
    pub(crate) fabricante: u32,
    pub(crate) id: u32,
    pub(crate) tipo: u32,
    pub(crate) nombre: [u8; 256],
    pub(crate) resto: [u8; 1024],
}

/// Desplazamientos dentro de `VkPhysicalDeviceProperties`, calculados campo a
/// campo sobre `vulkan_core.h` con las reglas de alineación de C (la estructura
/// mide 824 bytes y `limits` empieza en el 296).
pub(crate) const DESPLAZAMIENTO_GRUPOS_MAX_X: usize = 516; // limits.maxComputeWorkGroupCount[0]
pub(crate) const DESPLAZAMIENTO_PERIODO_TIEMPO: usize = 720; // limits.timestampPeriod
/// Dónde empieza `resto`: 5 × u32 + el nombre (256 bytes).
const INICIO_RESTO: usize = 20 + 256;

impl PropiedadesFisico {
    /// Los 4 bytes que hay en `desplazamiento` (contado desde el principio de la estructura).
    fn cuatro_bytes(&self, desplazamiento: usize) -> [u8; 4] {
        let i = desplazamiento - INICIO_RESTO;
        [self.resto[i], self.resto[i + 1], self.resto[i + 2], self.resto[i + 3]]
    }

    /// Cuántos grupos se pueden lanzar en X de una vez.
    pub(crate) fn grupos_max_x(&self) -> u32 {
        u32::from_le_bytes(self.cuatro_bytes(DESPLAZAMIENTO_GRUPOS_MAX_X))
    }

    /// Nanosegundos por tic del temporizador de la GPU.
    pub(crate) fn periodo_tiempo(&self) -> f32 {
        f32::from_le_bytes(self.cuatro_bytes(DESPLAZAMIENTO_PERIODO_TIEMPO))
    }
}

/// `VkQueueFamilyProperties`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub(crate) struct PropiedadesFamilia {
    pub(crate) flags: u32,
    pub(crate) n_colas: u32,
    pub(crate) bits_tiempo: u32,
    pub(crate) granularidad: [u32; 3],
}

/// `VkDeviceQueueCreateInfo`.
#[repr(C)]
pub(crate) struct InfoCola {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) familia: u32,
    pub(crate) n_colas: u32,
    pub(crate) prioridades: *const f32,
}

/// `VkDeviceCreateInfo`.
#[repr(C)]
pub(crate) struct InfoDispositivo {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) n_colas: u32,
    pub(crate) colas: *const InfoCola,
    pub(crate) n_capas: u32,
    pub(crate) capas: *const *const u8,
    pub(crate) n_extensiones: u32,
    pub(crate) extensiones: *const *const u8,
    pub(crate) funciones: *const c_void,
}

/// `VkSurfaceCapabilitiesKHR`.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Capacidades {
    pub(crate) min_imagenes: u32,
    pub(crate) max_imagenes: u32,
    pub(crate) actual: Extension2D,
    pub(crate) minima: Extension2D,
    pub(crate) maxima: Extension2D,
    pub(crate) max_capas: u32,
    pub(crate) transformaciones: u32,
    pub(crate) transformacion_actual: u32,
    pub(crate) alfa_compuesto: u32,
    pub(crate) usos: u32,
}

/// `VkSurfaceFormatKHR`.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormatoSuperficie {
    pub(crate) formato: u32,
    pub(crate) espacio_color: u32,
}

/// `VkSwapchainCreateInfoKHR`.
#[repr(C)]
pub(crate) struct InfoSwapchain {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) superficie: Objeto,
    pub(crate) min_imagenes: u32,
    pub(crate) formato: u32,
    pub(crate) espacio_color: u32,
    pub(crate) extension: Extension2D,
    pub(crate) capas: u32,
    pub(crate) uso: u32,
    pub(crate) compartir: u32,
    pub(crate) n_familias: u32,
    pub(crate) familias: *const u32,
    pub(crate) transformacion: u32,
    pub(crate) alfa: u32,
    pub(crate) modo_presentacion: u32,
    pub(crate) recortada: u32,
    pub(crate) anterior: Objeto,
}

/// `VkImageSubresourceRange`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Rango {
    pub(crate) aspecto: u32,
    pub(crate) mip_base: u32,
    pub(crate) n_mips: u32,
    pub(crate) capa_base: u32,
    pub(crate) n_capas: u32,
}

/// `VkImageMemoryBarrier`.
#[repr(C)]
pub(crate) struct BarreraImagen {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) acceso_origen: u32,
    pub(crate) acceso_destino: u32,
    pub(crate) layout_viejo: u32,
    pub(crate) layout_nuevo: u32,
    pub(crate) familia_origen: u32,
    pub(crate) familia_destino: u32,
    pub(crate) imagen: Objeto,
    pub(crate) rango: Rango,
}

/// `VkSubmitInfo`.
#[repr(C)]
pub(crate) struct InfoEnvio {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) n_esperas: u32,
    pub(crate) esperas: *const Objeto,
    pub(crate) etapas_espera: *const u32,
    pub(crate) n_comandos: u32,
    pub(crate) comandos: *const Comandos,
    pub(crate) n_senales: u32,
    pub(crate) senales: *const Objeto,
}

/// `VkPresentInfoKHR`.
#[repr(C)]
pub(crate) struct InfoPresentacion {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) n_esperas: u32,
    pub(crate) esperas: *const Objeto,
    pub(crate) n_swapchains: u32,
    pub(crate) swapchains: *const Objeto,
    pub(crate) indices: *const u32,
    pub(crate) resultados: *mut i32,
}

/// `VkCommandPoolCreateInfo`, `VkSemaphoreCreateInfo` y `VkFenceCreateInfo`
/// comparten forma: tipo, siguiente, flags (y la familia, solo el pool).
#[repr(C)]
pub(crate) struct InfoSimple {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) familia: u32,
}

/// `VkCommandBufferAllocateInfo`.
#[repr(C)]
pub(crate) struct InfoReservaComandos {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) pool: Objeto,
    pub(crate) nivel: u32,
    pub(crate) cantidad: u32,
}

/// `VkCommandBufferBeginInfo`.
#[repr(C)]
pub(crate) struct InfoInicioComandos {
    pub(crate) s_type: u32,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) herencia: *const c_void,
}
