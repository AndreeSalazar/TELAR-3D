//! De la geometría de NUDO a lo que dibuja la GPU: triángulos y, por vértice,
//! su posición y su normal.
//!
//! - **Triángulos:** cada polígono se abre en abanico desde su primer vértice:
//!   `(0, 1, 2), (0, 2, 3)…`; el sentido (y con él hacia dónde mira) se conserva.
//! - **Normales:** la de cada cara, con el método de Newell (vale también para
//!   polígonos que no son del todo planos).
//!   - Un punto **suave** (todas sus caras miran casi igual, a menos de
//!     [`ANGULO_SUAVE`] de su promedio) comparte un solo vértice con la normal media.
//!   - Un punto **en arista** (la esquina de una caja) se parte: un vértice por
//!     cara, cada uno con la normal plana de su cara.
//!
//!   Se parece al «cusp angle» del visor de Houdini, pero es una aproximación más
//!   simple: no agrupa las caras por suavidad entre ellas, solo contra el promedio.

#![allow(clippy::many_single_char_names, reason = "en geometría g, m, n, p, q, r e i son los nombres de siempre")]

use alloc::vec::Vec;

use crate::geometria::Geometria;
use crate::mate::{Vec3, normalizar, punto, sumar};

/// Hasta cuántos grados de su normal media puede girar una cara para que el
/// punto cuente como suave (el coseno, para comparar sin `acos`). 30°.
pub const ANGULO_SUAVE: f32 = 0.866_025_4;

/// Una malla para la GPU.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Malla {
    /// Por vértice: `[x, y, z, nx, ny, nz]`.
    pub vertices: Vec<[f32; 6]>,
    /// Tres índices por triángulo.
    pub indices: Vec<u32>,
}

/// La normal de una cara (sin normalizar, con el área: método de Newell).
fn normal_de_cara(g: &Geometria, cara: usize) -> Vec3 {
    let vertices = &g.vertices[g.primitivas[cara].start as usize..g.primitivas[cara].end as usize];
    let mut n = [0.0f32; 3];
    for (i, &v) in vertices.iter().enumerate() {
        let a = g.puntos[v as usize];
        let b = g.puntos[vertices[(i + 1) % vertices.len()] as usize];
        n = sumar(n, [(a[1] - b[1]) * (a[2] + b[2]), (a[2] - b[2]) * (a[0] + b[0]), (a[0] - b[0]) * (a[1] + b[1])]);
    }
    n
}

impl Malla {
    /// Convierte una geometría. Las primitivas de menos de 3 vértices no dan triángulos.
    #[must_use]
    pub fn de(g: &Geometria) -> Self {
        let normales: Vec<Vec3> = (0..g.primitivas.len()).map(|c| normal_de_cara(g, c)).collect();
        let unitarias: Vec<Vec3> = normales.iter().map(|&n| normalizar(n)).collect();

        // La normal media de cada punto (con el área de cada cara como peso).
        let mut media = alloc::vec![[0.0f32; 3]; g.puntos.len()];
        for (cara, r) in g.primitivas.iter().enumerate() {
            for &p in &g.vertices[r.start as usize..r.end as usize] {
                media[p as usize] = sumar(media[p as usize], normales[cara]);
            }
        }
        let media: Vec<Vec3> = media.into_iter().map(normalizar).collect();

        // ¿Es suave cada punto?
        let mut suave = alloc::vec![true; g.puntos.len()];
        for (cara, r) in g.primitivas.iter().enumerate() {
            for &p in &g.vertices[r.start as usize..r.end as usize] {
                if punto(unitarias[cara], media[p as usize]) < ANGULO_SUAVE {
                    suave[p as usize] = false;
                }
            }
        }

        // Los puntos suaves, un vértice cada uno (en el mismo orden que los puntos).
        let mut m = Malla::default();
        let mut compartido = alloc::vec![u32::MAX; g.puntos.len()];
        for (i, p) in g.puntos.iter().enumerate() {
            if suave[i] {
                compartido[i] = u32::try_from(m.vertices.len()).unwrap_or(u32::MAX);
                let n = if media[i].iter().all(|c| *c == 0.0) { [0.0, 1.0, 0.0] } else { media[i] };
                m.vertices.push([p[0], p[1], p[2], n[0], n[1], n[2]]);
            }
        }

        // Cada cara: sus vértices (compartidos o propios) y sus triángulos en abanico.
        let mut propios = Vec::new();
        for (cara, r) in g.primitivas.iter().enumerate() {
            propios.clear();
            for &p in &g.vertices[r.start as usize..r.end as usize] {
                let i = p as usize;
                if suave[i] {
                    propios.push(compartido[i]);
                } else {
                    let (q, n) = (g.puntos[i], unitarias[cara]);
                    propios.push(u32::try_from(m.vertices.len()).unwrap_or(u32::MAX));
                    m.vertices.push([q[0], q[1], q[2], n[0], n[1], n[2]]);
                }
            }
            for k in 1..propios.len().saturating_sub(1) {
                m.indices.extend_from_slice(&[propios[0], propios[k], propios[k + 1]]);
            }
        }
        m
    }

    /// Cuántos triángulos tiene.
    #[must_use]
    pub fn triangulos(&self) -> usize {
        self.indices.len() / 3
    }
}
