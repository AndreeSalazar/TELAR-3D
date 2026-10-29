//! Kernel coordinador residente: servicios GPU sobre un buzón acotado.
//! Rust emite SPIR-V al preparar; las decisiones del coordinador ocurren en GPU.
#![no_std]
extern crate alloc;

pub mod abi;
pub mod arranque;
mod buzon;
pub mod cache;
mod coordinador;
pub mod humo;
pub mod humo3d;
mod humo3d_colision;
mod humo3d_fuente;
mod humo3d_muestreo;
mod humo3d_pasos;
mod humo3d_tiempo;
mod humo_control;
mod humo_memoria;
mod humo_pasos;
mod maquina;
pub mod paquete;
pub mod particulas;
mod particulas_memoria;
mod particulas_paso;
mod particulas_resumen;
mod servicios;
pub mod trazadores;
mod trazadores_paso;
mod verificacion;
pub use buzon::Buzon;
pub use coordinador::{compilar, compilar_con_arranque, compilar_con_cache};
pub use servicios::solicitar_en;
