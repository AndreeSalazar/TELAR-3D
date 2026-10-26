//! Lo que comparten los tests de los nodos.

#![allow(
    dead_code,
    clippy::many_single_char_names,
    reason = "cada test usa solo parte de lo común; y en geometría a, b, c, u, v, n y p son los nombres de siempre"
)]

use nudo::Geometria;
use nudo::mate::{Vec3, restar};

/// La normal (sin normalizar) de una cara: `(b − a) × (c − a)`, con sus tres primeros vértices.
pub(crate) fn normal(g: &Geometria, cara: usize) -> Vec3 {
    let r = g.primitivas[cara].clone();
    let p = |i: u32| g.puntos[g.vertices[i as usize] as usize];
    let (a, b, c) = (p(r.start), p(r.start + 1), p(r.start + 2));
    let (u, v) = (restar(b, a), restar(c, a));
    [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
}

/// El promedio de los puntos de una cara.
pub(crate) fn centro_de(g: &Geometria, cara: usize) -> Vec3 {
    let r = g.primitivas[cara].clone();
    let n = f32::from(u16::try_from(r.len()).unwrap_or(1));
    let mut s = [0.0; 3];
    for i in r {
        let p = g.puntos[g.vertices[i as usize] as usize];
        s = [s[0] + p[0], s[1] + p[1], s[2] + p[2]];
    }
    s.map(|x| x / n)
}

/// ¿Mira cada cara hacia fuera de `centro` (normal · (centro de la cara − centro) > 0)?
pub(crate) fn caras_hacia_fuera(g: &Geometria, centro: Vec3) -> bool {
    (0..g.primitivas.len()).all(|i| {
        let n = normal(g, i);
        let d = restar(centro_de(g, i), centro);
        n[0] * d[0] + n[1] * d[1] + n[2] * d[2] > 0.0
    })
}

/// ¿Están `a` y `b` a menos de `tolerancia` en cada componente?
pub(crate) fn cerca(a: Vec3, b: Vec3, tolerancia: f32) -> bool {
    a.iter().zip(&b).all(|(x, y)| (x - y).abs() <= tolerancia)
}
