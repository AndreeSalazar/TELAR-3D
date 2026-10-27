//! Validación de contratos, matemáticas de entrada y shaders originales.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use std::{path::PathBuf, process::Command};

#[test]
fn shaders_del_render_pasan_validador_oficial() {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|p| PathBuf::from(p).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("spirv-val"));
    for (nombre, codigo) in [("vertices", telar_render::vertices()), ("fragmentos", telar_render::fragmentos())] {
        let ruta = std::env::temp_dir().join(format!("telar-render-{nombre}-{}.spv", std::process::id()));
        std::fs::write(&ruta, codigo.iter().flat_map(|p| p.to_le_bytes()).collect::<Vec<_>>()).unwrap();
        let salida =
            Command::new(&validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().expect("spirv-val");
        std::fs::remove_file(ruta).unwrap();
        assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
    }
}

#[test]
fn material_y_energia_invalidos_se_rechazan_antes_de_gpu() {
    let mut parametros = telar_render::Parametros {
        matriz: [[0.0; 4]; 4],
        ojo: [0.0, 0.0, 5.0],
        luz: [0.0, 1.0, 0.0],
        material: telar_render::Material { color: [0.5; 3], rugosidad: 0.3, metal: 0.8 },
        exposicion: 1.0,
        intensidad: 4.0,
        codificar_srgb: false,
    };
    assert!(parametros.bloque().is_ok());
    parametros.material.rugosidad = f32::NAN;
    assert!(parametros.bloque().is_err());
    parametros.material.rugosidad = 0.3;
    parametros.material.metal = 1.1;
    assert!(parametros.bloque().is_err());
    parametros.material.metal = 0.8;
    parametros.exposicion = 0.0;
    assert!(parametros.bloque().is_err());
}
