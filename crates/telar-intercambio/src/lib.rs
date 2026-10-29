//! Muestras puntuales tipadas y GEO clásico acotado, sin sistema operativo ni backend.
#![no_std]
extern crate alloc;
mod geo;
mod particulas;
use alloc::vec::Vec;
pub use geo::{codificar_geo, decodificar_geo};
pub use particulas::{Particula, codificar_particulas_geo, decodificar_particulas_geo, validar_particulas};
/// Límite explícito de elementos; no mide disponibilidad de memoria.
pub const MAX_PUNTOS: usize = 65_536;
/// Máximo texto aceptado/producido: 16 MiB.
pub const MAX_BYTES: usize = 16 * 1024 * 1024;
/// Una muestra; no representa por sí sola una partícula con dinámica y vida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Punto {
    /// Posición en unidades del espacio declarado por el exportador.
    pub posicion: [f32; 3],
    /// Velocidad en unidades de ese espacio por segundo.
    pub velocidad: [f32; 3],
    /// Densidad normalizada no negativa.
    pub densidad: f32,
    /// Calor normalizado no negativo; no kelvin.
    pub calor: f32,
    /// Identidad única de la muestra, limitada a int32 positivo.
    pub id: u32,
    /// Instante simulado en segundos, no negativo.
    pub tiempo: f32,
}
/// Valida valores e identidades antes de producir salida o devolver una lectura.
/// # Errors
/// Rechaza exceso de puntos, NaN/inf, magnitudes >1e20, escalares negativos e ids repetidos/fuera de int32.
pub fn validar(puntos: &[Punto]) -> Result<(), &'static str> {
    if puntos.len() > MAX_PUNTOS {
        return Err("exceso de puntos de intercambio");
    }
    for p in puntos {
        if p.posicion
            .iter()
            .chain(&p.velocidad)
            .chain([&p.densidad, &p.calor, &p.tiempo])
            .any(|v| !v.is_finite() || v.abs() > 1e20)
            || p.densidad < 0.0
            || p.calor < 0.0
            || p.tiempo < 0.0
            || p.id > 2_147_483_647
        {
            return Err("atributo de punto fuera de dominio");
        }
    }
    let mut ids: Vec<_> = puntos.iter().map(|p| p.id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|v| v[0] == v[1]) {
        return Err("identificadores de punto repetidos");
    }
    Ok(())
}
