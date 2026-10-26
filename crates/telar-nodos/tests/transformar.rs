//! `transformar`: el orden SRT, la rotación XYZ y que solo muevan los puntos.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

mod comun;

use nudo::{Estadisticas, Nodo};
use telar_nodos::{Caja, Transformar};

#[test]
fn escala_luego_gira_luego_traslada() {
    let t = Transformar {
        traslacion: [10.0, 0.0, 0.0],
        rotacion: [0.0, 0.0, 90.0],
        escala: [2.0, 1.0, 1.0],
        escala_uniforme: 1.0,
    };
    // (1,0,0) → escala → (2,0,0) → 90° en Z → (0,2,0) → traslación → (10,2,0).
    assert!(comun::cerca(t.aplicar([1.0, 0.0, 0.0]), [10.0, 2.0, 0.0], 1e-6));
}

#[test]
fn la_rotacion_va_en_orden_xyz() {
    let t = Transformar { rotacion: [90.0, 90.0, 0.0], ..Transformar::default() };
    // (0,1,0) → 90° en X → (0,0,1) → 90° en Y → (1,0,0).
    assert!(comun::cerca(t.aplicar([0.0, 1.0, 0.0]), [1.0, 0.0, 0.0], 1e-6));
}

#[test]
fn mueve_los_puntos_y_conserva_las_caras() {
    let caja = Caja::default().cocinar(&[]).unwrap();
    let t = Transformar { traslacion: [0.0, 5.0, 0.0], escala_uniforme: 2.0, ..Transformar::default() };
    let g = t.cocinar(&[&caja]).unwrap();
    assert_eq!((&g.vertices, &g.primitivas), (&caja.vertices, &caja.primitivas));
    let e = Estadisticas::de(&g);
    assert_eq!(e.minimo, Some([-1.0, 4.0, -1.0]));
    assert_eq!(e.maximo, Some([1.0, 6.0, 1.0]));
    assert!(comun::caras_hacia_fuera(&g, [0.0, 5.0, 0.0]));
}

#[test]
fn necesita_exactamente_una_entrada() {
    assert!(Transformar::default().cocinar(&[]).is_err());
}
