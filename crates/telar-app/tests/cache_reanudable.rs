//! Continuación real del ejecutable: checkpoint intermedio versus simulación ininterrumpida.
#![allow(clippy::expect_used, reason = "fallos explícitos de integración y limpieza de archivos propios")]
use std::path::Path;
fn ejecutar(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_telar")).args(args).output().expect("ejecutar TELAR")
}
fn correcto(args: &[&str]) {
    let salida = ejecutar(args);
    assert!(
        salida.status.success(),
        "{} {}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(String::from_utf8_lossy(&salida.stdout).contains("0 avisos de validación"));
}
#[test]
fn continuar_conserva_todos_los_bytes_y_no_reemplaza() {
    let carpeta = std::env::temp_dir().join(format!("telar-cache-prueba-{}", std::process::id()));
    std::fs::create_dir(&carpeta).expect("crear carpeta exclusiva");
    let a = carpeta.join("prefijo.tlcache");
    let b = carpeta.join("continuado.tlcache");
    let c = carpeta.join("completo.tlcache");
    let ruta = |p: &Path| p.to_str().expect("ruta UTF-8").to_string();
    correcto(&["--explosion", "--trazadores", "--pasos", "18", "--guardar-cache", &ruta(&a)]);
    correcto(&["--abrir-cache", &ruta(&a), "--pasos", "18", "--guardar-cache", &ruta(&b)]);
    correcto(&["--explosion", "--trazadores", "--pasos", "36", "--guardar-cache", &ruta(&c)]);
    let antes = std::fs::read(&b).expect("leer resultado");
    assert_eq!(antes, std::fs::read(&c).expect("leer referencia"));
    assert!(!ejecutar(&["--abrir-cache", &ruta(&a), "--pasos", "1", "--guardar-cache", &ruta(&b)]).status.success());
    assert_eq!(antes, std::fs::read(&b).expect("no reemplazado"));
    let mut roto = std::fs::read(&a).expect("leer prefijo");
    roto[200] ^= 1;
    let d = carpeta.join("roto.tlcache");
    std::fs::write(&d, roto).expect("crear corrupción");
    assert!(
        !ejecutar(&[
            "--abrir-cache",
            &ruta(&d),
            "--pasos",
            "1",
            "--guardar-cache",
            &ruta(&carpeta.join("rechazado.tlcache"))
        ])
        .status
        .success()
    );
    assert!(!carpeta.join("rechazado.tlcache").exists());
    for p in [a, b, c, d] {
        std::fs::remove_file(p).expect("borrar archivo de prueba");
    }
    std::fs::remove_dir(carpeta).expect("carpeta queda vacía");
}

#[test]
fn reactivo_reanuda_y_produce_hdr_identico() {
    let carpeta = std::env::temp_dir().join(format!("telar-reactivo-prueba-{}", std::process::id()));
    std::fs::create_dir(&carpeta).expect("crear carpeta");
    let a = carpeta.join("a.tlcache");
    let b = carpeta.join("b.tlcache");
    let c = carpeta.join("c.tlcache");
    let img_b = carpeta.join("b.pfm");
    let img_c = carpeta.join("c.pfm");
    let ruta = |p: &Path| p.to_str().expect("ruta").to_string();
    correcto(&["--plano-vfx", "--pasos", "8", "--guardar-cache", &ruta(&a)]);
    correcto(&[
        "--abrir-cache",
        &ruta(&a),
        "--pasos",
        "8",
        "--guardar-cache",
        &ruta(&b),
        "--render-vfx",
        &ruta(&img_b),
        "--spp",
        "2",
    ]);
    correcto(&[
        "--plano-vfx",
        "--pasos",
        "16",
        "--guardar-cache",
        &ruta(&c),
        "--render-vfx",
        &ruta(&img_c),
        "--spp",
        "2",
    ]);
    assert_eq!(std::fs::read(&b).expect("caché b"), std::fs::read(&c).expect("caché c"));
    assert_eq!(std::fs::read(&img_b).expect("HDR b"), std::fs::read(&img_c).expect("HDR c"));
    for p in [
        a,
        b,
        c,
        img_b.clone(),
        img_b.with_extension("ppm"),
        img_b.with_extension("txt"),
        img_c.clone(),
        img_c.with_extension("ppm"),
        img_c.with_extension("txt"),
    ] {
        std::fs::remove_file(p).expect("limpiar archivo propio");
    }
    std::fs::remove_dir(carpeta).expect("carpeta vacía");
}
