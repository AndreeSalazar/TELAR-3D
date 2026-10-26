//! Matemática propia, en `no_std`: en `core` no hay `sin`, `cos` ni `sqrt`
//! (viven en `std`), así que aquí se escriben con sumas, restas,
//! multiplicaciones y divisiones. Solo esas cuatro operaciones, que IEEE 754
//! define exactas, así que el resultado es **el mismo en cualquier máquina**.
//!
//! - `sin` y `cos`: se reduce el ángulo a [−π/4, π/4] restando múltiplos de π/2
//!   y se suma la serie de Taylor en `f64` (con 10 términos el error queda muy
//!   por debajo de lo que distingue un `f32`).
//! - `raiz`: Newton en `f64` desde una primera aproximación sacada de los bits.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    reason = "conversiones medidas (el ángulo reducido cabe en f32 y k en i64), y en matemática x, y, z, s y c son los nombres de siempre"
)]

use core::f64::consts::{FRAC_2_PI, FRAC_PI_2};

/// Un punto o un vector: `[x, y, z]`.
pub type Vec3 = [f32; 3];

/// El entero más cercano (los medios, lejos del cero). Para |v| < 2⁶³.
fn redondear(v: f64) -> i64 {
    let t = v as i64; // trunca hacia el cero
    let resto = v - t as f64;
    if resto >= 0.5 {
        t + 1
    } else if resto <= -0.5 {
        t - 1
    } else {
        t
    }
}

/// Seno y coseno de `r` ∈ [−π/4, π/4], con la serie de Taylor.
fn sin_cos_reducido(r: f64) -> (f64, f64) {
    let r2 = r * r;
    let (mut seno, mut termino_s) = (r, r);
    let (mut coseno, mut termino_c) = (1.0, 1.0);
    // término siguiente = −anterior · r² / ((n+1)(n+2))
    let mut n = 1.0;
    for _ in 0..10 {
        termino_s = -termino_s * r2 / ((n + 1.0) * (n + 2.0));
        termino_c = -termino_c * r2 / (n * (n + 1.0));
        seno += termino_s;
        coseno += termino_c;
        n += 2.0;
    }
    (seno, coseno)
}

/// Seno y coseno de `x` (radianes), a la vez. Precisos para |x| hasta unos 10⁶.
#[must_use]
pub fn sin_cos(x: f32) -> (f32, f32) {
    let x = f64::from(x);
    let k = redondear(x * FRAC_2_PI);
    let r = x - k as f64 * FRAC_PI_2;
    let (s, c) = sin_cos_reducido(r);
    let (s, c) = match k.rem_euclid(4) {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    };
    (s as f32, c as f32)
}

/// Seno de `x` (radianes).
#[must_use]
pub fn sin(x: f32) -> f32 {
    sin_cos(x).0
}

/// Coseno de `x` (radianes).
#[must_use]
pub fn cos(x: f32) -> f32 {
    sin_cos(x).1
}

/// Raíz cuadrada. `NaN` para negativos, como `f32::sqrt`.
#[must_use]
pub fn raiz(x: f32) -> f32 {
    if x.is_nan() || x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 || x.is_infinite() {
        return x;
    }
    let v = f64::from(x);
    // Primera aproximación: dividir el exponente entre dos, directamente en los bits.
    let mut g = f64::from_bits((v.to_bits() >> 1) + 0x1FF8_0000_0000_0000);
    for _ in 0..6 {
        g = f64::midpoint(g, v / g);
    }
    g as f32
}

/// Grados a radianes.
#[must_use]
pub fn radianes(grados: f32) -> f32 {
    (f64::from(grados) * core::f64::consts::PI / 180.0) as f32
}

/// `a + b`.
#[must_use]
pub fn sumar(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `a − b`.
#[must_use]
pub fn restar(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Producto componente a componente.
#[must_use]
pub fn multiplicar(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// Longitud de `v`.
#[must_use]
pub fn longitud(v: Vec3) -> f32 {
    raiz(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}

/// Gira `v` `grados` alrededor del eje X, Y o Z (`eje` = 0, 1, 2), en el sentido
/// de la mano derecha.
#[must_use]
pub fn rotar(v: Vec3, eje: usize, grados: f32) -> Vec3 {
    let (s, c) = sin_cos(radianes(grados));
    let [x, y, z] = v;
    match eje {
        0 => [x, y * c - z * s, y * s + z * c],
        1 => [x * c + z * s, y, -x * s + z * c],
        _ => [x * c - y * s, x * s + y * c, z],
    }
}
