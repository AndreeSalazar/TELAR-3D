//! # telar-guard
//!
//! El guardián de las reglas de TELAR 3D. Sin dependencias.
//!
//! Marca lo peligroso, hazlo pequeño, justifícalo y que un guardián lo vigile:
//! cada regla dice qué exige, por qué, dónde falla (archivo y línea) y cómo se
//! arregla. Vigila en tres sitios: `cargo guard check`, los hooks de git y los
//! hooks de Claude Code (`telar-guard hook …`). Ver docs/GUARDIANES.md.

pub mod commit;
pub mod escaner;
pub mod hooks;
pub mod informe;
pub mod json;
pub mod reglas;
pub mod repo;
mod toml;
pub mod verificar;
