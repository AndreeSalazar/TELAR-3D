//! Acoplamiento pasivo unidireccional a velocidad MAC; sin fuerzas sobre el gas.
use crate::{humo, humo3d};
/// Capacidad fija, ABI `SoA` de partículas de doce planos.
pub const CAPACIDAD: u32 = 128;
/// Palabras reservadas después de los diecisiete planos MAC.
pub const PALABRAS: u32 = CAPACIDAD * 12;
/// Plan temporal original con advección de trazadores integrada en el diagnóstico final.
/// # Errors
/// Rechaza lado fuera de 8..64 y parámetros temporales/presión inválidos.
pub fn compilar(config: humo3d::Configuracion, subpasos: u32) -> Result<humo::Programas, &'static str> {
    if !(8..=64).contains(&config.lado) {
        return Err("trazadores requieren lado 8..64");
    }
    let mut plan = config.compilar_temporal(subpasos)?;
    plan.codigos[5] = humo::programa_trazadores(
        humo::Configuracion { lado: config.lado, iteraciones: config.iteraciones },
        5,
        true,
        true,
        true,
    );
    Ok(plan)
}
/// Planos completos necesarios; el excedente queda reservado y sin uso.
#[must_use]
pub fn canales(lado: u32) -> u32 {
    let n = lado.saturating_pow(3).max(1);
    17 + PALABRAS.div_ceil(n)
}
/// Referencia escalar de ensayo; no interviene en el producto.
#[path = "trazadores_referencia.rs"]
pub mod referencia;
