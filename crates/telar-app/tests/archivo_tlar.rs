//! Un archivo propio se exporta y se inspecciona sin ejecutar sus shaders.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de verificación")]
use std::process::Command;

#[test]
fn exportacion_e_inspeccion_tlar_y_corrupcion_detectada() {
    let exe = env!("CARGO_BIN_EXE_telar");
    let ruta = std::env::temp_dir().join(format!("telar-paquete-{}.tlar", std::process::id()));
    let exportar = Command::new(exe).arg("--exportar-tlar").arg(&ruta).args(["--puntos", "81"]).output().unwrap();
    assert!(exportar.status.success(), "{}", String::from_utf8_lossy(&exportar.stderr));
    let inspeccion = Command::new(exe).arg("--inspeccionar-tlar").arg(&ruta).output().unwrap();
    assert!(inspeccion.status.success());
    assert!(String::from_utf8_lossy(&inspeccion.stdout).contains("capacidad 9"));
    let mut bytes = std::fs::read(&ruta).unwrap();
    bytes[24] ^= 1;
    std::fs::write(&ruta, bytes).unwrap();
    let rechazo = Command::new(exe).arg("--inspeccionar-tlar").arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(!rechazo.status.success());
    assert!(String::from_utf8_lossy(&rechazo.stderr).contains("CRC32"));
}
