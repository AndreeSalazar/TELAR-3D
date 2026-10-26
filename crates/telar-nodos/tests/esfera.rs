//! `esfera`: cifras (las fórmulas del libreto), puntos sobre la superficie y
//! caras hacia fuera.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

mod comun;

use nudo::mate::{longitud, restar};
use nudo::{Estadisticas, Nodo};
use telar_nodos::Esfera;

#[test]
fn las_cifras_siguen_las_formulas() {
    for (f, c) in [(13, 24), (3, 3), (4, 7), (30, 60)] {
        let g = Esfera { filas: f, columnas: c, ..Esfera::default() }.cocinar(&[]).unwrap();
        g.comprobar().unwrap();
        let (f, c) = (usize::try_from(f).unwrap(), usize::try_from(c).unwrap());
        let e = Estadisticas::de(&g);
        assert_eq!(e.puntos, 2 + (f - 2) * c, "puntos con {f}×{c}");
        assert_eq!(e.primitivas, 2 * c + (f - 3) * c, "caras con {f}×{c}");
        assert_eq!(e.vertices, 6 * c + 4 * (f - 3) * c, "vértices con {f}×{c}");
    }
}

#[test]
fn la_esfera_por_defecto() {
    let e = Estadisticas::de(&Esfera::default().cocinar(&[]).unwrap());
    assert_eq!((e.puntos, e.primitivas, e.vertices), (266, 288, 1104));
    assert!(comun::cerca(e.minimo.unwrap(), [-1.0; 3], 1e-6));
    assert!(comun::cerca(e.maximo.unwrap(), [1.0; 3], 1e-6));
}

#[test]
fn todos_los_puntos_en_la_superficie_y_caras_hacia_fuera() {
    let centro = [3.0, -1.0, 2.0];
    let g = Esfera { radio: 2.5, filas: 17, columnas: 31, centro }.cocinar(&[]).unwrap();
    for p in &g.puntos {
        assert!((longitud(restar(*p, centro)) - 2.5).abs() < 1e-5, "{p:?} no está a 2,5 del centro");
    }
    assert!(comun::caras_hacia_fuera(&g, centro));
}
