//! `caja`: cifras, caras hacia fuera y parámetros.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

mod comun;

use nudo::{Estadisticas, Nodo, Valor};
use telar_nodos::Caja;

#[test]
fn las_cifras_de_la_caja_por_defecto() {
    let g = Caja::default().cocinar(&[]).unwrap();
    g.comprobar().unwrap();
    let e = Estadisticas::de(&g);
    assert_eq!((e.puntos, e.primitivas, e.vertices), (8, 6, 24));
    assert_eq!(e.minimo, Some([-0.5; 3]));
    assert_eq!(e.maximo, Some([0.5; 3]));
    assert_eq!(e.centro, Some([0.0; 3]));
}

#[test]
fn cada_cara_mira_hacia_fuera() {
    let caja = Caja { tamano: [2.0, 3.0, 4.0], centro: [1.0, -2.0, 5.0] };
    let g = caja.cocinar(&[]).unwrap();
    assert!(comun::caras_hacia_fuera(&g, [1.0, -2.0, 5.0]));
    let e = Estadisticas::de(&g);
    assert_eq!(e.minimo, Some([0.0, -3.5, 3.0]));
    assert_eq!(e.maximo, Some([2.0, -0.5, 7.0]));
}

#[test]
fn parametros() {
    let mut caja = Caja::default();
    caja.poner("tamano", Valor::Vector([2.0; 3])).unwrap();
    assert_eq!(caja.tamano, [2.0; 3]);
    assert!(caja.poner("tamano", Valor::Real(1.0)).is_err());
    assert!(caja.poner("color", Valor::Real(1.0)).is_err());
}
