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
