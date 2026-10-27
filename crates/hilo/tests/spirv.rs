//! El compilador emite SPIR-V verificado por una herramienta independiente.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos de validación son errores de test")]

use hilo::{Id, Operacion as Op, Programa, Salida};
use std::{path::PathBuf, process::Command};

#[test]
fn ir_compilado_pasa_el_validador_oficial() {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|ruta| PathBuf::from(ruta).join("Bin/spirv-val.exe"))
        .filter(|ruta| ruta.exists())
        .unwrap_or_else(|| PathBuf::from("spirv-val"));
    let compuesto = Programa::nuevo(
        vec![
            Op::Posicion,
            Op::Normal,
            Op::Fase,
            Op::Amplitud,
            Op::Escalar(2.0),
            Op::Vector([0.25, 0.5, 0.75]),
            Op::Multiplicar(Id(0), Id(4)),
            Op::Multiplicar(Id(4), Id(1)),
            Op::Multiplicar(Id(6), Id(5)),
            Op::Sumar(Id(7), Id(8)),
            Op::Restar(Id(9), Id(5)),
            Op::ProductoEscalar(Id(0), Id(5)),
            Op::Sumar(Id(2), Id(3)),
            Op::Multiplicar(Id(11), Id(12)),
            Op::Restar(Id(13), Id(4)),
            Op::Extraer(Id(10), 1),
            Op::Componer([Id(14), Id(15), Id(4)]),
            Op::Normalizar(Id(16)),
        ],
        Salida { altura: Id(14), normal: Id(17) },
    )
    .unwrap();
    let mut programas = vec![compuesto];
    programas.push(hilo::ejemplos::relieve_dinamico().unwrap());
    programas
        .push(Programa::nuevo(vec![Op::Parametro(3), Op::Normal], Salida { altura: Id(0), normal: Id(1) }).unwrap());
    for factor in [-2.0, 0.0, 1.3] {
        programas.push(hilo::ejemplos::relieve(factor).unwrap());
    }
    for (indice, programa) in programas.iter().enumerate() {
        let codigo = programa.compilar();
        assert_eq!(codigo, programa.compilar(), "el compilador emite los mismos bytes");
        let ruta = std::env::temp_dir().join(format!("telar-hilo-ir-{}-{indice}.spv", std::process::id()));
        std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
        let salida = Command::new(&validador)
            .args(["--target-env", "vulkan1.3"])
            .arg(&ruta)
            .output()
            .expect("esta prueba requiere spirv-val del SDK de Vulkan");
        let _ = std::fs::remove_file(&ruta);
        assert!(salida.status.success(), "programa {indice}: {}", String::from_utf8_lossy(&salida.stderr));
    }
}
