//! HILO: representación intermedia tipada por punto, independiente de Vulkan.
//! Valida SSA lineal, interpreta sobre CPU y compila a un kernel SPIR-V.
//! Esta primera entrega aún no incluye analizador de texto ni grafos generales.
#![no_std]

extern crate alloc;

pub mod ejemplos;
mod interprete;
mod ir;
mod spirv;

pub use interprete::{Entrada, Resultado, Valor};
pub use ir::{Error, Id, Operacion, Programa, Salida, Tipo};
