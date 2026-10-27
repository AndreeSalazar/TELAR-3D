//! El constructor de SPIR-V: la codificación palabra a palabra, el hash contra
//! una implementación independiente y el primer kernel validado por `spirv-val`
//! (el validador oficial de Khronos, que viene con el SDK de Vulkan).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use std::path::PathBuf;
use std::process::Command;

use hilo_spirv::ejemplos::{hash_pcg, llenar_con_hash};
use hilo_spirv::{Modulo, op, texto};

#[test]
fn cabecera_y_codificacion() {
    let mut m = Modulo::nuevo();
    let a = m.tipo_u32();
    let b = m.tipo_u32();
    assert_eq!(a, b, "el mismo tipo se declara una sola vez");
    let palabras = m.ensamblar();
    assert_eq!(&palabras[..3], &[op::MAGIA, op::VERSION_1_3, 0]);
    assert_eq!(palabras[3], 2, "límite de ids: el siguiente libre");
    assert_eq!(palabras[4], 0);
    // OpCapability Shader: 2 palabras, código 17.
    assert_eq!(palabras[5], 0x0002_0011);
    assert_eq!(palabras[6], op::CAPACIDAD_SHADER);
}

#[test]
fn textos_terminados_en_cero_y_rellenos() {
    // "main" + \0 = 5 bytes → 2 palabras; "abc" + \0 = 4 bytes → 1.
    assert_eq!(texto("main"), [u32::from_le_bytes(*b"main"), 0]);
    assert_eq!(texto("abc"), [u32::from_le_bytes(*b"abc\0")]);
}

#[test]
fn hash_igual_que_una_implementacion_independiente() {
    // Calculados aparte, en JavaScript con Math.imul (ver el README).
    let esperados =
        [(0, 129_708_002), (1, 2_831_084_092), (2, 2_055_130_248), (12345, 4_099_845_390), (u32::MAX, 3_861_530_882)];
    for (v, h) in esperados {
        assert_eq!(hash_pcg(v), h, "hash_pcg({v})");
    }
}

/// `spirv-val` del SDK (`VULKAN_SDK`) o del `PATH`.
fn spirv_val() -> Option<PathBuf> {
    let del_sdk = std::env::var_os("VULKAN_SDK").map(|sdk| PathBuf::from(sdk).join("Bin").join("spirv-val.exe"));
    match del_sdk {
        Some(ruta) if ruta.exists() => Some(ruta),
        _ => Command::new("spirv-val").arg("--version").output().is_ok().then(|| PathBuf::from("spirv-val")),
    }
}

/// Si la máquina no tiene `spirv-val`, el test lo dice y no cuenta como fallo.
#[test]
fn el_primer_kernel_pasa_spirv_val() {
    let Some(validador) = spirv_val() else {
        eprintln!("sin spirv-val en esta máquina (instala el SDK de Vulkan)");
        return;
    };
    for grupo in [1, 64, 256] {
        let palabras = llenar_con_hash(grupo);
        let bytes: Vec<u8> = palabras.iter().flat_map(|w| w.to_le_bytes()).collect();
        let archivo = std::env::temp_dir().join(format!("telar-hash-{grupo}-{}.spv", std::process::id()));
        std::fs::write(&archivo, bytes).unwrap();
        let salida = Command::new(&validador).args(["--target-env", "vulkan1.3"]).arg(&archivo).output().unwrap();
        let _ = std::fs::remove_file(&archivo);
        assert!(
            salida.status.success(),
            "spirv-val rechaza el kernel (grupo {grupo}):\n{}{}",
            String::from_utf8_lossy(&salida.stdout),
            String::from_utf8_lossy(&salida.stderr)
        );
    }
}

#[test]
fn los_shaders_del_visor_pasan_spirv_val() {
    let Some(validador) = spirv_val() else {
        eprintln!("sin spirv-val en esta máquina (instala el SDK de Vulkan)");
        return;
    };
    for (nombre, palabras) in
        [("vertices", hilo_spirv::visor::vertices()), ("fragmentos", hilo_spirv::visor::fragmentos())]
    {
        let bytes: Vec<u8> = palabras.iter().flat_map(|w| w.to_le_bytes()).collect();
        let archivo = std::env::temp_dir().join(format!("telar-visor-{nombre}-{}.spv", std::process::id()));
        std::fs::write(&archivo, bytes).unwrap();
        let salida = Command::new(&validador).args(["--target-env", "vulkan1.3"]).arg(&archivo).output().unwrap();
        let _ = std::fs::remove_file(&archivo);
        assert!(
            salida.status.success(),
            "spirv-val rechaza el shader de {nombre}:\n{}{}",
            String::from_utf8_lossy(&salida.stdout),
            String::from_utf8_lossy(&salida.stderr)
        );
    }
}

#[test]
fn los_tres_kernels_residentes_pasan_spirv_val() {
    let validador = spirv_val().expect("esta prueba requiere spirv-val del SDK de Vulkan");
    for (nombre, palabras) in [
        ("preparar", hilo_spirv::rejilla::preparar()),
        ("generar", hilo_spirv::rejilla::generar()),
        ("deformar", hilo_spirv::rejilla::deformar()),
    ] {
        let bytes: Vec<u8> = palabras.iter().flat_map(|w| w.to_le_bytes()).collect();
        let archivo = std::env::temp_dir().join(format!("telar-residente-{nombre}-{}.spv", std::process::id()));
        std::fs::write(&archivo, bytes).unwrap();
        let salida = Command::new(&validador).args(["--target-env", "vulkan1.3"]).arg(&archivo).output().unwrap();
        let _ = std::fs::remove_file(&archivo);
        assert!(salida.status.success(), "{nombre}: {}", String::from_utf8_lossy(&salida.stderr));
    }
}
