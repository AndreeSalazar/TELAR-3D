//! Las cifras de una geometría, las que se comparan con Houdini (su hoja de
//! cálculo de geometría y su información del nodo): recuentos, la caja que la
//! contiene y el promedio de las posiciones.

#![allow(clippy::cast_precision_loss, reason = "promedios: el recuento como f64 es exacto hasta 2⁵³ puntos")]

use crate::geometria::Geometria;
use crate::mate::Vec3;

/// Las cifras de una geometría.
#[derive(Debug, Clone, PartialEq)]
pub struct Estadisticas {
    /// Puntos.
    pub puntos: usize,
    /// Vértices.
    pub vertices: usize,
    /// Primitivas.
    pub primitivas: usize,
    /// La esquina mínima de la caja que contiene todos los puntos (`None` si no hay).
    pub minimo: Option<Vec3>,
    /// La esquina máxima.
    pub maximo: Option<Vec3>,
    /// El promedio de `P` (sumado en `f64`, para no perder precisión con millones de puntos).
    pub centro: Option<Vec3>,
}

impl Estadisticas {
    /// Calcula las cifras.
    #[must_use]
    pub fn de(g: &Geometria) -> Self {
        let mut minimo = [f32::INFINITY; 3];
        let mut maximo = [f32::NEG_INFINITY; 3];
        let mut suma = [0.0f64; 3];
        for p in &g.puntos {
            for i in 0..3 {
                minimo[i] = minimo[i].min(p[i]);
                maximo[i] = maximo[i].max(p[i]);
                suma[i] += f64::from(p[i]);
            }
        }
        let hay = !g.puntos.is_empty();
        let n = g.puntos.len() as f64;
        #[allow(clippy::cast_possible_truncation, reason = "un promedio de f32 vuelve a caber en f32")]
        let centro = suma.map(|s| (s / n) as f32);
        Self {
            puntos: g.puntos.len(),
            vertices: g.vertices.len(),
            primitivas: g.primitivas.len(),
            minimo: hay.then_some(minimo),
            maximo: hay.then_some(maximo),
            centro: hay.then_some(centro),
        }
    }
}
