//! Personalización de la aplicación desde un archivo CSS externo.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de verificación")]
use std::process::Command;

#[test]
fn aplicacion_carga_css_externo_y_rechaza_opcion_sin_ruta() {
    let ejecutable = env!("CARGO_BIN_EXE_telar");
    let error = Command::new(ejecutable).arg("--estilo").output().unwrap();
    assert!(!error.status.success());
    assert!(String::from_utf8_lossy(&error.stderr).contains("requiere la ruta"));
    let ruta = std::env::temp_dir().join(format!("telar-estilo-{}.css", std::process::id()));
    std::fs::write(&ruta, ".cabecera { background-color: #223344; } .inspector { width: 310px; }").unwrap();
    let salida =
        Command::new(ejecutable).args(["--prueba", "--puntos", "4096", "--estilo"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(&ruta).unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    assert!(salida.status.success(), "{texto}\n{}", String::from_utf8_lossy(&salida.stderr));
    assert!(texto.contains("120 fotogramas · 0 avisos de validación"), "{texto}");
    assert!(texto.contains("0 bytes geometría subida · 0 bytes descargados"), "{texto}");
}
