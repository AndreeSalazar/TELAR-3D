//! De la geometría a triángulos para la GPU (`nudo::Malla`), con los nodos de verdad.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use nudo::mate::{longitud, normalizar, punto};
use nudo::{Malla, Nodo};
use telar_nodos::{Caja, Esfera, Rejilla};

#[test]
fn la_caja_se_parte_en_sus_aristas() {
    let m = Malla::de(&Caja::default().cocinar(&[]).unwrap());
    // Cada esquina está en 3 caras perpendiculares: se parte en 3 vértices.
    assert_eq!(m.vertices.len(), 24);
    assert_eq!(m.triangulos(), 12);
    for v in &m.vertices {
        let n = [v[3], v[4], v[5]];
        // Una normal de caja: un eje, hacia fuera (del mismo signo que la posición).
        assert_eq!(n.iter().filter(|c| c.abs() > 0.999).count(), 1, "{n:?}");
        assert!(punto(n, [v[0], v[1], v[2]]) > 0.0);
    }
}

#[test]
fn la_esfera_es_suave_y_sus_normales_salen_del_centro() {
    let e = Esfera::default();
    let m = Malla::de(&e.cocinar(&[]).unwrap());
    assert_eq!(m.vertices.len(), 266, "un vértice por punto: todos suaves");
    assert_eq!(m.triangulos(), 2 * 24 + 2 * 10 * 24);
    for v in &m.vertices {
        let radial = normalizar([v[0], v[1], v[2]]);
        assert!(punto(radial, [v[3], v[4], v[5]]) > 0.99, "la normal no sale del centro en {v:?}");
        assert!((longitud([v[3], v[4], v[5]]) - 1.0).abs() < 1e-5);
    }
}

#[test]
fn la_rejilla_mira_hacia_arriba() {
    let m = Malla::de(&Rejilla::default().cocinar(&[]).unwrap());
    assert_eq!((m.vertices.len(), m.triangulos()), (100, 162));
    assert!(m.vertices.iter().all(|v| (v[4] - 1.0).abs() < 1e-6));
    assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
}
