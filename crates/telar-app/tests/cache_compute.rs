//! Comparación de ruta reutilizada y ruta directa en el ejecutable, incluidos estado VFX y HDR.
#![allow(clippy::unwrap_used, reason = "archivos propios y fallos explícitos del ensayo")]
#[test]
fn vfx_cache_y_directo_son_identicos() {
    let carpeta = std::env::temp_dir().join(format!("telar-compute-cache-{}", std::process::id()));
    std::fs::create_dir(&carpeta).unwrap();
    for (nombre, directa, sin_indirectos) in
        [("cache", false, false), ("directa", true, false), ("sin-indirectos", false, true)]
    {
        let mut comando = std::process::Command::new(env!("CARGO_BIN_EXE_telar"));
        comando
            .args(["--plano-vfx", "--pasos", "36", "--spp", "2", "--guardar-cache"])
            .arg(carpeta.join(format!("{nombre}.tlcache")))
            .arg("--render-vfx")
            .arg(carpeta.join(format!("{nombre}.pfm")));
        if directa {
            comando.arg("--sin-cache-compute");
        }
        if sin_indirectos {
            comando.arg("--sin-indirectos");
        }
        let resultado = comando.output().unwrap();
        assert!(resultado.status.success(), "{}", String::from_utf8_lossy(&resultado.stderr));
        assert!(String::from_utf8_lossy(&resultado.stdout).contains("0 avisos de validación"));
    }
    for extension in ["tlcache", "pfm", "ppm"] {
        assert_eq!(
            std::fs::read(carpeta.join(format!("cache.{extension}"))).unwrap(),
            std::fs::read(carpeta.join(format!("directa.{extension}"))).unwrap()
        );
        assert_eq!(
            std::fs::read(carpeta.join(format!("cache.{extension}"))).unwrap(),
            std::fs::read(carpeta.join(format!("sin-indirectos.{extension}"))).unwrap()
        );
    }
    for nombre in ["cache", "directa", "sin-indirectos"] {
        for extension in ["tlcache", "pfm", "ppm", "txt"] {
            std::fs::remove_file(carpeta.join(format!("{nombre}.{extension}"))).unwrap();
        }
    }
    std::fs::remove_dir(carpeta).unwrap();
}
