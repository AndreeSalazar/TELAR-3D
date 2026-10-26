//! Las funciones del dispositivo, cargadas con `vkGetDeviceProcAddr`. Las de
//! la swapchain son aparte y opcionales: un dispositivo sin ventana no activa
//! `VK_KHR_swapchain`, y entonces no existen.

use core::ffi::c_void;

use super::carga::FnInstancia;
use super::funciones_calculo::FnCalculo;
use super::tipos::{
    Asig, BarreraImagen, Cola, Comandos, Disp, InfoEnvio, InfoInicioComandos, InfoPresentacion, InfoReservaComandos,
    InfoSimple, InfoSwapchain, Objeto, Rango,
};

// Las firmas, tal y como las da vulkan_core.h.
type LeerCola = unsafe extern "system" fn(Disp, u32, u32, *mut Cola);
type Esperar = unsafe extern "system" fn(Disp) -> i32;
type Crear = unsafe extern "system" fn(Disp, *const InfoSimple, Asig, *mut Objeto) -> i32;
type Destruir = unsafe extern "system" fn(Disp, Objeto, Asig);
type EsperarVallas = unsafe extern "system" fn(Disp, u32, *const Objeto, u32, u64) -> i32;
type ReiniciarVallas = unsafe extern "system" fn(Disp, u32, *const Objeto) -> i32;
type ReservarComandos = unsafe extern "system" fn(Disp, *const InfoReservaComandos, *mut Comandos) -> i32;
type ReiniciarComandos = unsafe extern "system" fn(Comandos, u32) -> i32;
type Empezar = unsafe extern "system" fn(Comandos, *const InfoInicioComandos) -> i32;
type Terminar = unsafe extern "system" fn(Comandos) -> i32;
type Barrera = unsafe extern "system" fn(
    Comandos,
    u32,
    u32,
    u32,
    u32,
    *const c_void,
    u32,
    *const c_void,
    u32,
    *const BarreraImagen,
);
type LimpiarColor = unsafe extern "system" fn(Comandos, Objeto, u32, *const [f32; 4], u32, *const Rango);
type Enviar = unsafe extern "system" fn(Cola, u32, *const InfoEnvio, Objeto) -> i32;
type CrearSwapchain = unsafe extern "system" fn(Disp, *const InfoSwapchain, Asig, *mut Objeto) -> i32;
type ListarImagenes = unsafe extern "system" fn(Disp, Objeto, *mut u32, *mut Objeto) -> i32;
type Adquirir = unsafe extern "system" fn(Disp, Objeto, u64, Objeto, Objeto, *mut u32) -> i32;
type Presentar = unsafe extern "system" fn(Cola, *const InfoPresentacion) -> i32;

/// Las funciones del dispositivo.
#[derive(Clone, Copy)]
pub(crate) struct FnDispositivo {
    pub(crate) cola: LeerCola,
    pub(crate) esperar: Esperar,
    pub(crate) crear_semaforo: Crear,
    pub(crate) destruir_semaforo: Destruir,
    pub(crate) crear_valla: Crear,
    pub(crate) destruir_valla: Destruir,
    pub(crate) esperar_vallas: EsperarVallas,
    pub(crate) reiniciar_vallas: ReiniciarVallas,
    pub(crate) crear_pool: Crear,
    pub(crate) destruir_pool: Destruir,
    pub(crate) reservar_comandos: ReservarComandos,
    pub(crate) reiniciar_comandos: ReiniciarComandos,
    pub(crate) empezar: Empezar,
    pub(crate) terminar: Terminar,
    pub(crate) barrera: Barrera,
    pub(crate) limpiar_color: LimpiarColor,
    pub(crate) enviar: Enviar,
    /// Solo si el dispositivo activó `VK_KHR_swapchain` (hay ventana).
    pub(crate) swapchain: Option<FnSwapchain>,
    /// Buffers, pipelines de cálculo, despacho y tiempos.
    pub(crate) calculo: FnCalculo,
}

/// Las funciones de `VK_KHR_swapchain`.
#[derive(Clone, Copy)]
pub(crate) struct FnSwapchain {
    pub(crate) crear: CrearSwapchain,
    pub(crate) destruir: Destruir,
    pub(crate) imagenes: ListarImagenes,
    pub(crate) adquirir: Adquirir,
    pub(crate) presentar: Presentar,
}

impl FnDispositivo {
    pub(crate) fn cargar(instancia: &FnInstancia, dispositivo: Disp, con_swapchain: bool) -> Result<Self, String> {
        let obtener = instancia.obtener_dispositivo;
        // SAFETY: el dispositivo es válido y el nombre es ASCII terminado en cero.
        let buscar = |n: &[u8]| unsafe { obtener(dispositivo, n.as_ptr()) };
        let swapchain = if con_swapchain {
            Some(FnSwapchain {
                crear: cargar!(buscar, "vkCreateSwapchainKHR", CrearSwapchain),
                destruir: cargar!(buscar, "vkDestroySwapchainKHR", Destruir),
                imagenes: cargar!(buscar, "vkGetSwapchainImagesKHR", ListarImagenes),
                adquirir: cargar!(buscar, "vkAcquireNextImageKHR", Adquirir),
                presentar: cargar!(buscar, "vkQueuePresentKHR", Presentar),
            })
        } else {
            None
        };
        Ok(Self {
            cola: cargar!(buscar, "vkGetDeviceQueue", LeerCola),
            esperar: cargar!(buscar, "vkDeviceWaitIdle", Esperar),
            crear_semaforo: cargar!(buscar, "vkCreateSemaphore", Crear),
            destruir_semaforo: cargar!(buscar, "vkDestroySemaphore", Destruir),
            crear_valla: cargar!(buscar, "vkCreateFence", Crear),
            destruir_valla: cargar!(buscar, "vkDestroyFence", Destruir),
            esperar_vallas: cargar!(buscar, "vkWaitForFences", EsperarVallas),
            reiniciar_vallas: cargar!(buscar, "vkResetFences", ReiniciarVallas),
            crear_pool: cargar!(buscar, "vkCreateCommandPool", Crear),
            destruir_pool: cargar!(buscar, "vkDestroyCommandPool", Destruir),
            reservar_comandos: cargar!(buscar, "vkAllocateCommandBuffers", ReservarComandos),
            reiniciar_comandos: cargar!(buscar, "vkResetCommandBuffer", ReiniciarComandos),
            empezar: cargar!(buscar, "vkBeginCommandBuffer", Empezar),
            terminar: cargar!(buscar, "vkEndCommandBuffer", Terminar),
            barrera: cargar!(buscar, "vkCmdPipelineBarrier", Barrera),
            limpiar_color: cargar!(buscar, "vkCmdClearColorImage", LimpiarColor),
            enviar: cargar!(buscar, "vkQueueSubmit", Enviar),
            swapchain,
            calculo: FnCalculo::cargar(buscar)?,
        })
    }
}
