//! La aplicación de verdad, de punta a punta: ventana oculta, 120 fotogramas en
//! la GPU, dos cambios de tamaño (la swapchain se rehace) y cero avisos de las
//! capas de validación.

#![allow(clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use std::process::Command;

#[test]
fn ventana_y_gpu_sin_avisos_de_validacion() {
    let salida = Command::new(env!("CARGO_BIN_EXE_telar")).arg("--prueba").output().expect("no se pudo lanzar telar");
    let texto = String::from_utf8_lossy(&salida.stdout);
    let errores = String::from_utf8_lossy(&salida.stderr);
    if errores.contains("vulkan-1.dll") {
        eprintln!("sin Vulkan en esta máquina: {errores}");
        return;
    }
    assert!(salida.status.success(), "telar falló:\n{texto}\n{errores}");
    assert!(texto.contains("120 fotogramas · 0 avisos de validación"), "salida inesperada:\n{texto}\n{errores}");
    // La inicial y una por cada cambio de tamaño.
    assert!(texto.contains("swapchain creada 3 veces"), "la swapchain no se rehízo:\n{texto}");
}

#[test]
fn el_primer_kernel_da_lo_mismo_en_la_gpu_que_en_la_cpu() {
    let salida = Command::new(env!("CARGO_BIN_EXE_telar"))
        .args(["--calculo", "--elementos", "1048576"])
        .output()
        .expect("no se pudo lanzar telar");
    let texto = String::from_utf8_lossy(&salida.stdout);
    let errores = String::from_utf8_lossy(&salida.stderr);
    if errores.contains("vulkan-1.dll") {
        eprintln!("sin Vulkan en esta máquina: {errores}");
        return;
    }
    assert!(salida.status.success(), "telar --calculo falló:\n{texto}\n{errores}");
    assert!(texto.contains("iguales bit a bit (1048576 de 1048576)"), "{texto}");
    assert!(texto.contains("0 avisos de validación"), "{texto}");
}

/// La meta de la Fase 4: la esfera de un millón de puntos (≈ 2 millones de
/// triángulos) en el visor, con la swapchain rehecha dos veces, sin avisos. Los
/// fps no se comprueban aquí (dependen de la máquina): se miden con `--segundos`.
#[test]
fn el_visor_con_un_millon_de_puntos_sin_avisos() {
    let salida = Command::new(env!("CARGO_BIN_EXE_telar"))
        .args(["--prueba", "--puntos", "1000000"])
        .output()
        .expect("no se pudo lanzar telar");
    let texto = String::from_utf8_lossy(&salida.stdout);
    let errores = String::from_utf8_lossy(&salida.stderr);
    if errores.contains("vulkan-1.dll") {
        eprintln!("sin Vulkan en esta máquina: {errores}");
        return;
    }
    assert!(salida.status.success(), "telar falló:\n{texto}\n{errores}");
    assert!(texto.contains("esfera de 996872 puntos · 1993740 triángulos"), "{texto}");
    assert!(texto.contains("120 fotogramas · 0 avisos de validación"), "{texto}");
}
