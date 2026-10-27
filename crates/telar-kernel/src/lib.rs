//! Kernel coordinador residente: servicios GPU sobre un buzón acotado.
//! Rust emite SPIR-V al preparar; las decisiones del coordinador ocurren en GPU.
#![no_std]
extern crate alloc;

pub mod abi;
mod buzon;
mod coordinador;
mod maquina;
mod servicios;
pub use buzon::Buzon;
pub use coordinador::compilar;
pub use servicios::solicitar_en;
