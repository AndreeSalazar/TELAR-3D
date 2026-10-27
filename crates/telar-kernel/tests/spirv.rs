//! Contrato y validación de los módulos portables con la herramienta de Khronos.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use std::{path::PathBuf, process::Command};

#[test]
fn coordinador_y_productores_validan_en_vulkan_11_y_13() {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|sdk| PathBuf::from(sdk).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("spirv-val"));
    for (indice, codigo) in
        [telar_kernel::compilar(), telar_kernel::solicitar_en(1, 2), telar_kernel::solicitar_en(999, 4)]
            .into_iter()
            .enumerate()
    {
        let ruta = std::env::temp_dir().join(format!("telar-kernel-{indice}-{}.spv", std::process::id()));
        let bytes: Vec<u8> = codigo.iter().flat_map(|p| p.to_le_bytes()).collect();
        std::fs::write(&ruta, bytes).unwrap();
        for entorno in ["vulkan1.1", "vulkan1.3"] {
            let salida = Command::new(&validador)
                .args(["--target-env", entorno])
                .arg(&ruta)
                .output()
                .expect("requiere spirv-val");
            assert!(salida.status.success(), "{entorno}: {}", String::from_utf8_lossy(&salida.stderr));
        }
        std::fs::remove_file(ruta).unwrap();
    }
    assert_eq!(telar_kernel::abi::PALABRAS * 4, 128);
}
