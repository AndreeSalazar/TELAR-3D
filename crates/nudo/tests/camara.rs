//! La cámara orbital: dónde cae cada punto en la imagen y qué profundidad tiene.

#![allow(clippy::float_cmp, reason = "algunos valores son exactos por construcción")]

use nudo::camara::{Camara, multiplicar, transformar};
use nudo::mate::longitud;

/// Coordenadas normalizadas `(x, y, profundidad)` de un punto.
fn ndc(camara: &Camara, aspecto: f32, punto: [f32; 3]) -> [f32; 3] {
    let [x, y, z, w] = transformar(&camara.matriz(aspecto), punto);
    assert!(w > 0.0, "{punto:?} queda detrás de la cámara");
    [x / w, y / w, z / w]
}

fn frente() -> Camara {
    // Desde +Z, sin girar ni inclinar, a 5 del origen.
    Camara { giro: 0.0, inclinacion: 0.0, distancia: 5.0, ..Camara::default() }
}

#[test]
fn el_objetivo_cae_en_el_centro_y_la_posicion_es_la_de_la_orbita() {
    let c = Camara { objetivo: [1.0, 2.0, 3.0], ..Camara::default() };
    let centro = ndc(&c, 16.0 / 9.0, c.objetivo);
    assert!(centro[0].abs() < 1e-6 && centro[1].abs() < 1e-6, "{centro:?}");
    let d = longitud(nudo::mate::restar(c.posicion(), c.objetivo));
    assert!((d - c.distancia).abs() < 1e-5);
    assert!(frente().posicion().iter().zip([0.0, 0.0, 5.0]).all(|(a, b)| (a - b).abs() < 1e-6));
}

#[test]
fn arriba_sale_arriba_y_la_derecha_a_la_derecha() {
    let c = frente();
    // En Vulkan, la Y de la imagen va hacia abajo: lo de arriba tiene y negativa.
    assert!(ndc(&c, 1.0, [0.0, 1.0, 0.0])[1] < 0.0);
    assert!(ndc(&c, 1.0, [1.0, 0.0, 0.0])[0] > 0.0);
}

#[test]
fn profundidad_invertida_uno_cerca_cero_lejos() {
    let c = frente();
    // Justo en el plano cercano (a 0,01 de la cámara, que está en z = 5): profundidad 1.
    assert!((ndc(&c, 1.0, [0.0, 0.0, 5.0 - c.cerca])[2] - 1.0).abs() < 1e-4);
    // Más lejos, menos profundidad; nunca negativa (sin plano lejano).
    let (cerca, lejos) = (ndc(&c, 1.0, [0.0, 0.0, 0.0])[2], ndc(&c, 1.0, [0.0, 0.0, -1000.0])[2]);
    assert!(cerca > lejos && lejos > 0.0, "{cerca} {lejos}");
}

#[test]
fn el_campo_de_vision_llega_al_borde_de_la_imagen() {
    // Con 90° de campo y a distancia 5, un punto a 5 de altura sobre el objetivo cae en el borde.
    let c = Camara { campo: 90.0, ..frente() };
    assert!((ndc(&c, 1.0, [0.0, 5.0, 0.0])[1] + 1.0).abs() < 1e-5);
    // Con una imagen el doble de ancha, lo mismo en horizontal llega solo a la mitad.
    assert!((ndc(&c, 2.0, [5.0, 0.0, 0.0])[0] - 0.5).abs() < 1e-5);
}

#[test]
fn orbitar_y_acercar_tienen_limites() {
    let mut c = Camara::default();
    c.orbitar(0.0, 500.0);
    assert_eq!(c.inclinacion, 89.0);
    c.acercar(1e-9);
    assert_eq!(c.distancia, c.cerca * 2.0);
    let i = multiplicar(
        &[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        &c.vista(),
    );
    assert_eq!(i, c.vista(), "multiplicar por la identidad no cambia nada");
}
