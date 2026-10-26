//! La única puerta de la GPU al sistema: Vulkan a mano. Todo el `unsafe` del
//! crate vive aquí, cada uno con su `// SAFETY:`; hacia fuera solo salen
//! `Contexto` y `Presentacion`, con métodos seguros.
//!
//! - `tipos`: estructuras y constantes, comprobadas contra los headers del SDK.
//! - `carga`: `vulkan-1.dll`, las funciones globales y las de la instancia.
//! - `funciones`: las del dispositivo (y, si hay ventana, las de la swapchain).
//! - `contexto`: instancia, validación, superficie, dispositivo y cola.
//! - `presentacion`: swapchain, sincronización y el dibujo de cada fotograma.
//! - `tipos_calculo`, `funciones_calculo` y `calculo`: buffers, pipelines de cálculo y tiempos.

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

mod calculo;
mod carga;
mod contexto;
mod funciones;
mod funciones_calculo;
mod presentacion;
mod recursos;
mod tipos;
mod tipos_calculo;

pub(crate) use calculo::ejecutar;
pub(crate) use contexto::{AVISOS, Contexto};
pub(crate) use presentacion::Presentacion;
