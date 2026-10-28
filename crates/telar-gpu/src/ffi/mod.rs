//! La única puerta de la GPU al sistema: Vulkan a mano. Todo el `unsafe` del
//! crate vive aquí, cada uno con su `// SAFETY:`; hacia fuera solo salen
//! `Contexto`, `Presentacion`, `Visor` y las funciones `ejecutar` y `capturar`,
//! con interfaces seguras.
//!
//! - `tipos`: estructuras y constantes, comprobadas contra los headers del SDK.
//! - `carga`: `vulkan-1.dll`, las funciones globales y las de la instancia.
//! - `funciones`: las del dispositivo (y, si hay ventana, las de la swapchain).
//! - `contexto`: instancia, validación, superficie, dispositivo y cola.
//! - `presentacion`: swapchain, sincronización y el dibujo de cada fotograma.
//! - `tipos_calculo`, `funciones_calculo` y `calculo`: buffers, pipelines de cálculo y tiempos.
//! - `tipos_grafico` y `funciones_grafico`: imágenes, pipeline de gráficos y renderizado dinámico.
//! - `buffer`, `imagen`, `malla_gpu` y `pipeline_grafico`: los recursos del visor.
//! - `swapchain`: la swapchain con sus vistas y su profundidad.
//! - `dibujo`: grabar un fotograma; `visor`: la escena de la ventana y las capturas.

#![allow(unsafe_code, reason = "la puerta a Vulkan: cada unsafe lleva su // SAFETY:")]

/// Un puntero a función cualquiera, antes de darle su tipo.
pub(crate) type FnVacia = unsafe extern "system" fn();

/// Carga una función de Vulkan por su nombre (`buscar`: nombre terminado en cero
/// → puntero) y le da su tipo: `$tipo`, la firma de `vulkan_core.h` para ese nombre.
macro_rules! cargar {
    ($buscar:expr, $nombre:literal, $tipo:ty) => {{
        let puntero: $crate::ffi::FnVacia =
            $buscar(concat!($nombre, "\0").as_bytes()).ok_or(concat!("Vulkan no tiene ", $nombre))?;
        // SAFETY: el cargador de Vulkan devolvió la función con ese nombre, y `$tipo`
        // es su firma exacta (vulkan_core.h); ambos son punteros a función del mismo tamaño.
        unsafe { core::mem::transmute::<$crate::ffi::FnVacia, $tipo>(puntero) }
    }};
}

mod buffer;
mod calculo;
mod calculo_lotes;
pub(crate) use calculo_lotes::ejecutar_lotes;
mod campos;
mod captura_interfaz;
mod carga;
mod contexto;
mod dibujo;
mod fuente_gpu;
mod funciones;
mod funciones_calculo;
mod funciones_grafico;
mod historial;
mod imagen;
mod interfaz;
mod malla_gpu;
mod multimuestreo;
mod pipeline_grafico;
mod presentacion;
mod recursos;
mod residente;
mod residente_comandos;
mod residente_lectura;
mod swapchain;
mod tipos;
mod tipos_calculo;
mod tipos_grafico;
mod visor;

pub(crate) use calculo::ejecutar;
pub(crate) use captura_interfaz::capturar_malla;
pub(crate) use contexto::{AVISOS, Contexto};
pub(crate) use interfaz::InterfazGpu;
pub(crate) use pipeline_grafico::TAMANO_CONSTANTES;
pub(crate) use presentacion::Presentacion;
pub(crate) use residente::Residente;
pub(crate) use visor::{Visor, capturar};
