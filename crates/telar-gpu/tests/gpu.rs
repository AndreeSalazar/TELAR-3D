//! Las decisiones puras de la GPU y un contexto de Vulkan de verdad (sin
//! ventana), con las capas de validación: cero avisos.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "en los tests, un fallo es un panic con su mensaje; los floats comparados son exactos"
)]

use telar_gpu::eleccion::{
    FORMATO_SRGB, FORMATO_UNORM, TIPO_DEDICADA, TIPO_INTEGRADA, color_para, elegir_extension, elegir_formato,
    elegir_imagenes, hex, mezclar, puntuar, srgb_a_lineal,
};
use telar_gpu::{Calculo, Opciones, avisos_de_validacion};

#[test]
fn la_gpu_dedicada_gana() {
    assert!(puntuar(TIPO_DEDICADA) > puntuar(TIPO_INTEGRADA));
    assert!(puntuar(TIPO_INTEGRADA) > puntuar(4)); // CPU
}

#[test]
fn formato_srgb_si_existe() {
    assert_eq!(elegir_formato(&[(FORMATO_UNORM, 0), (FORMATO_SRGB, 0)]), Some((FORMATO_SRGB, 0)));
    assert_eq!(elegir_formato(&[(FORMATO_UNORM, 0)]), Some((FORMATO_UNORM, 0)));
    assert_eq!(elegir_formato(&[(64, 7)]), Some((64, 7)));
    assert_eq!(elegir_formato(&[]), None);
}

#[test]
fn tamano_e_imagenes() {
    // La superficie fija el tamaño: manda ella.
    assert_eq!(elegir_extension((800, 600), (1, 1), (4096, 4096), (1280, 720)), (800, 600));
    // Lo deja libre (u32::MAX): el de la ventana, dentro de los límites.
    assert_eq!(elegir_extension((u32::MAX, u32::MAX), (1, 1), (1000, 1000), (1280, 720)), (1000, 720));
    assert_eq!(elegir_imagenes(2, 0), 3);
    assert_eq!(elegir_imagenes(2, 2), 2);
}

#[test]
fn colores_de_pantalla_a_luz_lineal() {
    assert_eq!(srgb_a_lineal(0.0), 0.0);
    assert_eq!(srgb_a_lineal(1.0), 1.0);
    assert!((srgb_a_lineal(0.5) - 0.214).abs() < 0.001);
    assert_eq!(hex(0x35_e3ff), [53.0 / 255.0, 227.0 / 255.0, 1.0, 1.0]);
    // En una imagen UNORM el color va tal cual; en una sRGB, en lineal.
    assert_eq!(color_para(FORMATO_UNORM, [0.5; 4]), [0.5; 4]);
    assert!(color_para(FORMATO_SRGB, [0.5; 4])[0] < 0.5);
    assert_eq!(mezclar([0.0; 4], [1.0; 4], 0.25), [0.25; 4]);
}

/// Si el sistema no tiene Vulkan (una máquina sin GPU), el test lo dice y no
/// cuenta como fallo: aquí se prueba TELAR, no el driver.
#[test]
fn contexto_de_vulkan_sin_avisos() {
    let calculo = match Calculo::nuevo(&Opciones::default()) {
        Ok(c) => c,
        Err(e) if e.contains("vulkan-1.dll") => {
            eprintln!("sin Vulkan en esta máquina: {e}");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    eprintln!("GPU: {} · validación: {}", calculo.dispositivo(), calculo.validacion());
    assert_ne!(calculo.dispositivo(), "");
    drop(calculo);
    assert_eq!(avisos_de_validacion(), 0, "las capas de validación avisaron (ver arriba)");
}
