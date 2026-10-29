//! Plano reactivo: combustible, humo y partículas sobre el MAC existente.
use crate::{humo, humo3d};
/// Reserva: MAC 17, combustible 2, configuración 1 y cola de partículas/historial.
#[must_use]
pub fn canales(lado: u32) -> u32 {
    20 + 3072u32.div_ceil(lado.saturating_pow(3).max(1))
}
/// Compila los dieciséis programas con reacción y secundarios; no ejecuta física host.
/// # Errors
/// Rechaza lado fuera de 8..64 o presión/subpasos inválidos.
pub fn compilar(config: humo3d::Configuracion, subpasos: u32) -> Result<humo::Programas, &'static str> {
    if !(8..=64).contains(&config.lado) {
        return Err("plano VFX requiere 8..64 celdas por eje");
    }
    let orden = humo3d::orden_temporal(config.iteraciones, subpasos)?;
    let c = humo::Configuracion { lado: config.lado, iteraciones: config.iteraciones };
    let codigos =
        (0..16).map(|i| humo::programa_extension(c, if i >= 14 { i + 1 } else { i }, true, true, 2)).collect();
    Ok(humo::Programas { codigos, orden })
}
