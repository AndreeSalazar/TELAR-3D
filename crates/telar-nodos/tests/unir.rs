//! `unir`: los recuentos se suman, los índices se desplazan y los atributos se
//! rellenan con ceros donde faltan.

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

mod comun;

use nudo::{Clase, Estadisticas, Nodo, Valores};
use telar_nodos::{Caja, Esfera, Unir};

#[test]
fn los_recuentos_se_suman_y_las_caras_siguen_bien() {
    let caja = Caja { centro: [5.0, 0.0, 0.0], ..Caja::default() }.cocinar(&[]).unwrap();
    let esfera = Esfera::default().cocinar(&[]).unwrap();
    let g = Unir.cocinar(&[&caja, &esfera]).unwrap();
    g.comprobar().unwrap();
    let (a, b, u) = (Estadisticas::de(&caja), Estadisticas::de(&esfera), Estadisticas::de(&g));
    assert_eq!(u.puntos, a.puntos + b.puntos);
    assert_eq!(u.vertices, a.vertices + b.vertices);
    assert_eq!(u.primitivas, a.primitivas + b.primitivas);
    // Las caras de la esfera, ya desplazadas, siguen apuntando a sus puntos.
    assert!((6..g.primitivas.len()).all(|i| comun::cerca(
        comun::centro_de(&g, i),
        comun::centro_de(&esfera, i - 6),
        0.0
    )));
}

#[test]
fn los_atributos_que_faltan_se_rellenan_con_ceros() {
    let mut a = Caja::default().cocinar(&[]).unwrap();
    a.poner(Clase::Punto, "peso", Valores::Real(vec![1.0; 8])).unwrap();
    let mut b = Caja::default().cocinar(&[]).unwrap();
    b.poner(Clase::Primitiva, "id", Valores::Entero((0..6).collect())).unwrap();
    let g = Unir.cocinar(&[&a, &b]).unwrap();
    let peso = [vec![1.0; 8], vec![0.0; 8]].concat();
    assert_eq!(g.atributo(Clase::Punto, "peso"), Some(&Valores::Real(peso)));
    let id: Vec<i32> = [0; 6].into_iter().chain(0..6).collect();
    assert_eq!(g.atributo(Clase::Primitiva, "id"), Some(&Valores::Entero(id)));
}

#[test]
fn atributos_de_tipos_distintos_son_un_error() {
    let mut a = Caja::default().cocinar(&[]).unwrap();
    a.poner(Clase::Punto, "x", Valores::Real(vec![1.0; 8])).unwrap();
    let mut b = Caja::default().cocinar(&[]).unwrap();
    b.poner(Clase::Punto, "x", Valores::Entero(vec![1; 8])).unwrap();
    assert!(Unir.cocinar(&[&a, &b]).is_err());
}
