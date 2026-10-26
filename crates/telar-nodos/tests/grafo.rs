//! Una escena con los nodos de verdad: cambiar un parámetro recalcula solo lo
//! que depende de él.
//!
//! ```text
//!   caja ──► transformar ──► unir
//!   esfera ─────────────────┘
//! ```

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use nudo::{Estadisticas, Grafo, Valor};
use telar_nodos::{Caja, Esfera, Transformar, Unir};

#[test]
fn una_escena_que_cocina_solo_lo_necesario() {
    let mut grafo = Grafo::nuevo();
    let caja = grafo.anadir(Caja::default(), &[]).unwrap();
    let mover = grafo.anadir(Transformar::default(), &[caja]).unwrap();
    let esfera = grafo.anadir(Esfera::default(), &[]).unwrap();
    let unir = grafo.anadir(Unir, &[mover, esfera]).unwrap();

    let e = Estadisticas::de(grafo.cocinar(unir).unwrap());
    assert_eq!((e.puntos, e.primitivas), (8 + 266, 6 + 288));
    assert_eq!(grafo.cocinados(), 4);

    // Mover la caja: transformar y unir. La caja y la esfera no se tocan.
    grafo.poner(mover, "traslacion", Valor::Vector([0.0, 10.0, 0.0])).unwrap();
    let e = Estadisticas::de(grafo.cocinar(unir).unwrap());
    assert_eq!(e.maximo.unwrap()[1], 10.5);
    assert_eq!(grafo.cocinados(), 4 + 2);

    // Más detalle en la esfera: esfera y unir.
    grafo.poner(esfera, "filas", Valor::Entero(30)).unwrap();
    let e = Estadisticas::de(grafo.cocinar(unir).unwrap());
    assert_eq!(e.puntos, 8 + 2 + 28 * 24);
    assert_eq!(grafo.cocinados(), 6 + 2);

    // Un parámetro inválido no ensucia nada.
    assert!(grafo.poner(esfera, "filas", Valor::Entero(2)).is_err());
    grafo.cocinar(unir).unwrap();
    assert_eq!(grafo.cocinados(), 8);
}
