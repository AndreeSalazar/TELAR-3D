//! Cargar Vulkan a mano: `vulkan-1.dll` con `LoadLibraryW`, y cada función por
//! su nombre con `vkGetInstanceProcAddr`. Sin la `.lib` del SDK: si el sistema
//! no tiene Vulkan, es un error con su motivo, no un fallo al arrancar.

use core::ffi::c_void;

use super::FnVacia;
use super::tipos::{
    Asig, Capacidades, Disp, Fisico, FormatoSuperficie, InfoDispositivo, InfoInstancia, InfoMensajero,
    InfoSuperficieWin32, Instancia, Objeto, PropiedadesCapa, PropiedadesFamilia, PropiedadesFisico,
};
use super::tipos_calculo::PropiedadesMemoria;

// SAFETY: firmas de libloaderapi.h (kernel32.dll): el nombre de la biblioteca en
// UTF-16 terminado en cero y el de la función en ASCII terminado en cero.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(nombre: *const u16) -> isize;
    fn GetProcAddress(modulo: isize, nombre: *const u8) -> *const c_void;
}

// Las firmas, tal y como las da vulkan_core.h (y vulkan_win32.h).
type ObtenerInstancia = unsafe extern "system" fn(Instancia, *const u8) -> Option<FnVacia>;
pub(crate) type ObtenerDispositivo = unsafe extern "system" fn(Disp, *const u8) -> Option<FnVacia>;
type CrearInstancia = unsafe extern "system" fn(*const InfoInstancia, Asig, *mut Instancia) -> i32;
type ListarCapas = unsafe extern "system" fn(*mut u32, *mut PropiedadesCapa) -> i32;
type DestruirInstancia = unsafe extern "system" fn(Instancia, Asig);
type ListarFisicos = unsafe extern "system" fn(Instancia, *mut u32, *mut Fisico) -> i32;
type LeerPropiedades = unsafe extern "system" fn(Fisico, *mut PropiedadesFisico);
type LeerFamilias = unsafe extern "system" fn(Fisico, *mut u32, *mut PropiedadesFamilia);
type CrearDispositivo = unsafe extern "system" fn(Fisico, *const InfoDispositivo, Asig, *mut Disp) -> i32;
type DestruirDispositivo = unsafe extern "system" fn(Disp, Asig);
type EsperarDispositivo = unsafe extern "system" fn(Disp) -> i32;
type CrearSuperficie = unsafe extern "system" fn(Instancia, *const InfoSuperficieWin32, Asig, *mut Objeto) -> i32;
type DestruirDeInstancia = unsafe extern "system" fn(Instancia, Objeto, Asig);
type SoportaPresentar = unsafe extern "system" fn(Fisico, u32, Objeto, *mut u32) -> i32;
type LeerCapacidades = unsafe extern "system" fn(Fisico, Objeto, *mut Capacidades) -> i32;
type ListarFormatos = unsafe extern "system" fn(Fisico, Objeto, *mut u32, *mut FormatoSuperficie) -> i32;
type LeerMemoria = unsafe extern "system" fn(Fisico, *mut PropiedadesMemoria);
type CrearMensajero = unsafe extern "system" fn(Instancia, *const InfoMensajero, Asig, *mut Objeto) -> i32;

/// Las funciones globales (antes de tener instancia).
#[derive(Clone, Copy)]
pub(crate) struct Global {
    pub(crate) obtener: ObtenerInstancia,
    pub(crate) crear_instancia: CrearInstancia,
    pub(crate) capas: ListarCapas,
}

impl Global {
    /// Abre `vulkan-1.dll` y carga las funciones globales.
    pub(crate) fn cargar() -> Result<Self, String> {
        let nombre: Vec<u16> = "vulkan-1.dll".encode_utf16().chain(core::iter::once(0)).collect();
        // SAFETY: nombre UTF-16 terminado en cero; la biblioteca se queda cargada
        // todo el proceso (nunca se libera: sus funciones se usan hasta el final).
        let modulo = unsafe { LoadLibraryW(nombre.as_ptr()) };
        if modulo == 0 {
            return Err("no encuentro vulkan-1.dll: instala o actualiza el driver de la GPU".to_owned());
        }
        // SAFETY: el módulo está cargado y el nombre es ASCII terminado en cero.
        let puntero = unsafe { GetProcAddress(modulo, c"vkGetInstanceProcAddr".as_ptr().cast::<u8>()) };
        if puntero.is_null() {
            return Err("vulkan-1.dll no exporta vkGetInstanceProcAddr".to_owned());
        }
        // SAFETY: vkGetInstanceProcAddr tiene exactamente la firma `ObtenerInstancia`;
        // un puntero de datos y uno de función miden lo mismo en x86-64.
        let obtener = unsafe { core::mem::transmute::<*const c_void, ObtenerInstancia>(puntero) };
        // SAFETY: con instancia null, vkGetInstanceProcAddr da las funciones globales;
        // el nombre es ASCII terminado en cero.
        let buscar = |n: &[u8]| unsafe { obtener(core::ptr::null_mut(), n.as_ptr()) };
        Ok(Self {
            obtener,
            crear_instancia: cargar!(buscar, "vkCreateInstance", CrearInstancia),
            capas: cargar!(buscar, "vkEnumerateInstanceLayerProperties", ListarCapas),
        })
    }
}

/// Las funciones de la instancia. Destruir el dispositivo también está aquí
/// (vale por instancia): así se puede soltar aunque falle la carga del resto.
#[derive(Clone, Copy)]
pub(crate) struct FnInstancia {
    pub(crate) destruir: DestruirInstancia,
    pub(crate) fisicos: ListarFisicos,
    pub(crate) propiedades: LeerPropiedades,
    pub(crate) familias: LeerFamilias,
    pub(crate) crear_dispositivo: CrearDispositivo,
    pub(crate) destruir_dispositivo: DestruirDispositivo,
    pub(crate) esperar_dispositivo: EsperarDispositivo,
    pub(crate) obtener_dispositivo: ObtenerDispositivo,
    pub(crate) crear_superficie: CrearSuperficie,
    pub(crate) destruir_superficie: DestruirDeInstancia,
    pub(crate) soporta_presentar: SoportaPresentar,
    pub(crate) capacidades: LeerCapacidades,
    pub(crate) formatos: ListarFormatos,
    pub(crate) memoria: LeerMemoria,
    /// Solo con validación (`VK_EXT_debug_utils`).
    pub(crate) mensajero: Option<(CrearMensajero, DestruirDeInstancia)>,
}

impl FnInstancia {
    pub(crate) fn cargar(global: &Global, instancia: Instancia, con_mensajero: bool) -> Result<Self, String> {
        let obtener = global.obtener;
        // SAFETY: la instancia es válida y el nombre es ASCII terminado en cero.
        let buscar = |n: &[u8]| unsafe { obtener(instancia, n.as_ptr()) };
        let mensajero = if con_mensajero {
            Some((
                cargar!(buscar, "vkCreateDebugUtilsMessengerEXT", CrearMensajero),
                cargar!(buscar, "vkDestroyDebugUtilsMessengerEXT", DestruirDeInstancia),
            ))
        } else {
            None
        };
        Ok(Self {
            destruir: cargar!(buscar, "vkDestroyInstance", DestruirInstancia),
            fisicos: cargar!(buscar, "vkEnumeratePhysicalDevices", ListarFisicos),
            propiedades: cargar!(buscar, "vkGetPhysicalDeviceProperties", LeerPropiedades),
            familias: cargar!(buscar, "vkGetPhysicalDeviceQueueFamilyProperties", LeerFamilias),
            crear_dispositivo: cargar!(buscar, "vkCreateDevice", CrearDispositivo),
            destruir_dispositivo: cargar!(buscar, "vkDestroyDevice", DestruirDispositivo),
            esperar_dispositivo: cargar!(buscar, "vkDeviceWaitIdle", EsperarDispositivo),
            obtener_dispositivo: cargar!(buscar, "vkGetDeviceProcAddr", ObtenerDispositivo),
            crear_superficie: cargar!(buscar, "vkCreateWin32SurfaceKHR", CrearSuperficie),
            destruir_superficie: cargar!(buscar, "vkDestroySurfaceKHR", DestruirDeInstancia),
            soporta_presentar: cargar!(buscar, "vkGetPhysicalDeviceSurfaceSupportKHR", SoportaPresentar),
            capacidades: cargar!(buscar, "vkGetPhysicalDeviceSurfaceCapabilitiesKHR", LeerCapacidades),
            formatos: cargar!(buscar, "vkGetPhysicalDeviceSurfaceFormatsKHR", ListarFormatos),
            memoria: cargar!(buscar, "vkGetPhysicalDeviceMemoryProperties", LeerMemoria),
            mensajero,
        })
    }
}
