//! # hilo-spirv
//!
//! SPIR-V emitido a mano, en `no_std`: el «ensamblador» de TELAR antes de tener
//! el lenguaje HILO, como TALK-CPU antes de los compiladores. Una API en Rust
//! que escribe las instrucciones palabra a palabra; sin glslc, sin shaderc,
//! sin crates.
//!
//! ```
//! let spirv = hilo_spirv::ejemplos::llenar_con_hash(64);
//! assert_eq!(spirv[0], hilo_spirv::op::MAGIA);
//! assert_eq!(hilo_spirv::ejemplos::hash_pcg(0), 129_708_002);
//! ```
//!
//! Es del núcleo (regla `nucleo-sin-std`): solo `core` y `alloc`, así que el
//! mismo compilador correrá algún día dentro de BMO-X.

#![no_std]

extern crate alloc;

mod bucles;
pub mod ejemplos;
mod funcion;
pub mod interfaz;
mod modulo;
pub mod op;
pub mod rejilla;
pub mod visor;

pub use funcion::Funcion;
pub use modulo::{Id, Modulo, texto};
