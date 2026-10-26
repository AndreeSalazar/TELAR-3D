//! `rejilla`: cifras, normales hacia arriba y parámetros.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

mod comun;

use nudo::{Estadisticas, Nodo, Valor};
use telar_nodos::Rejilla;

#[test]
fn las_cifras_de_la_rejilla_por_defecto() {
    let g = Rejilla::default().cocinar(&[]).unwrap();
    g.comprobar().unwrap();
    let e = Estadisticas::de(&g);
    assert_eq!((e.puntos, e.primitivas, e.vertices), (100, 81, 324));
    assert_eq!(e.minimo, Some([-5.0, 0.0, -5.0]));
    assert_eq!(e.maximo, Some([5.0, 0.0, 5.0]));
    assert!(comun::cerca(e.centro.unwrap(), [0.0; 3], 1e-6));
}

#[test]
fn todas_las_caras_miran_hacia_arriba() {
    let g = Rejilla { filas: 3, columnas: 5, ..Rejilla::default() }.cocinar(&[]).unwrap();
    assert_eq!(g.primitivas.len(), 2 * 4);
    for cara in 0..g.primitivas.len() {
        let n = comun::normal(&g, cara);
        assert!(n[1] > 0.0 && n[0] == 0.0 && n[2] == 0.0, "la cara {cara} mira hacia {n:?}");
    }
}

#[test]
fn menos_de_dos_filas_no() {
    let mut r = Rejilla::default();
    assert!(r.poner("filas", Valor::Entero(1)).is_err());
    assert!(r.poner("filas", Valor::Real(3.0)).is_err());
    r.poner("columnas", Valor::Entero(2)).unwrap();
    assert_eq!(r.cocinar(&[]).unwrap().puntos.len(), 10 * 2);
}
