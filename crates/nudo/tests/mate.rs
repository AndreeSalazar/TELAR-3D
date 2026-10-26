//! La matemática propia contra la de `std` (que aquí, en los tests, sí existe).

#![allow(clippy::float_cmp, reason = "se comparan bits exactos a propósito")]

use nudo::mate::{cos, longitud, raiz, rotar, sin, sin_cos};

/// Ángulos de −1000 a 1000 radianes, en pasos raros (para no caer solo en «bonitos»).
fn angulos() -> impl Iterator<Item = f32> {
    (-200_000..=200_000).map(|i| {
        f32::from(i16::try_from(i / 7).unwrap_or(0)) * 0.035 + f32::from(i8::try_from(i % 7).unwrap_or(0)) * 0.005
    })
}

#[test]
fn seno_y_coseno_como_los_de_std() {
    let mut peor = 0.0f32;
    for x in angulos() {
        let (s, c) = sin_cos(x);
        peor = peor.max((s - x.sin()).abs()).max((c - x.cos()).abs());
    }
    // Un f32 cerca de 1 distingue ~6e-8: el error se queda en ese orden.
    assert!(peor < 2e-7, "error máximo {peor}");
}

#[test]
fn valores_exactos_donde_deben() {
    assert_eq!(sin(0.0), 0.0);
    assert_eq!(cos(0.0), 1.0);
    assert!((sin(core::f32::consts::FRAC_PI_2) - 1.0).abs() < 1e-7);
    assert!(cos(core::f32::consts::PI) + 1.0 < 1e-7);
}

#[test]
fn raiz_como_la_de_std() {
    let mut distintas = 0;
    for i in 0..200_000u32 {
        let x = f32::from_bits(i.wrapping_mul(21_473) % 0x7F00_0000);
        let (nuestra, suya) = (raiz(x), x.sqrt());
        if nuestra.to_bits() != suya.to_bits() {
            distintas += 1;
            // Nunca más de una unidad en el último bit.
            assert!(nuestra.to_bits().abs_diff(suya.to_bits()) <= 1, "raiz({x}) = {nuestra}, std da {suya}");
        }
    }
    assert!(distintas < 20, "{distintas} raíces difieren en el último bit");
    assert!(raiz(-1.0).is_nan());
    assert_eq!(raiz(0.0), 0.0);
    assert_eq!(raiz(16.0), 4.0);
}

#[test]
fn rotaciones_de_mano_derecha() {
    let cerca = |a: [f32; 3], b: [f32; 3]| a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-6);
    // 90° en Z lleva X a Y; en X lleva Y a Z; en Y lleva Z a X.
    assert!(cerca(rotar([1.0, 0.0, 0.0], 2, 90.0), [0.0, 1.0, 0.0]));
    assert!(cerca(rotar([0.0, 1.0, 0.0], 0, 90.0), [0.0, 0.0, 1.0]));
    assert!(cerca(rotar([0.0, 0.0, 1.0], 1, 90.0), [1.0, 0.0, 0.0]));
    assert!((longitud(rotar([3.0, 4.0, 0.0], 1, 37.0)) - 5.0).abs() < 1e-6);
}
