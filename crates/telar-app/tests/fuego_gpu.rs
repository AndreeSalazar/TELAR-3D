//! Contratos visuales del volumen real en Vulkan; no equivalencia con Pyro.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "pruebas de capturas pequeñas")]
use telar_gpu::{Calculo, CapaInterfaz, Opciones, Sombreadores};
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32]) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-fuego-{}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn captura(gpu: &Calculo, codigo: &[u32], datos: &[f32; 32]) -> Vec<u8> {
    gpu.capturar_malla_interfaz(
        Sombreadores { vertices: &telar_render::visor_rayos::vertices(), fragmentos: codigo },
        (&MALLA, &[0, 2, 1]),
        (96, 96),
        [0.0, 0.0, 0.0, 1.0],
        datos,
        CapaInterfaz {
            shaders: Sombreadores {
                vertices: &hilo_spirv::interfaz::vertices(),
                fragmentos: &hilo_spirv::interfaz::fragmentos(),
            },
            vertices: &[],
            area: [0, 0, 96, 96],
            atlas: None,
        },
    )
    .unwrap()
    .pixeles
}
#[test]
fn volumen_determinista_y_campos_visibles() {
    assert!(telar_render::fuego::fragmentos(15).is_err());
    assert!(telar_render::fuego::fragmentos(129).is_err());
    let gpu = Calculo::nuevo(&Opciones { muestras: 1, ..Opciones::default() }).unwrap();
    let codigo = telar_render::fuego::fragmentos(64).unwrap();
    validar(&codigo);
    for pasos in [16, 128] {
        validar(&telar_render::fuego::fragmentos(pasos).unwrap());
    }
    let mut datos = [0.0; 32];
    datos[..10].copy_from_slice(&[1.0, 0.45, 7.0, 1.0, 0.0, 3.0, 1.0, 1.0, 0.0, 1.0]);
    let belleza = captura(&gpu, &codigo, &datos);
    assert_eq!(belleza, captura(&gpu, &codigo, &datos));
    assert_eq!(&belleza[..3], &[0, 0, 0]);
    let centro = (48 * 96 + 48) * 4;
    assert!(belleza[centro] > 100);
    datos[4] = 1.5;
    assert_ne!(belleza, captura(&gpu, &codigo, &datos));
    datos[4] = 0.0;
    datos[2] = 0.0;
    let apagada = captura(&gpu, &codigo, &datos);
    assert!(apagada.as_chunks::<4>().0.iter().all(|p| p[..3] == [0; 3]));
    let mut campos = Vec::new();
    for vista in 1..=3 {
        datos[8] = vista as f32;
        let campo = captura(&gpu, &codigo, &datos);
        assert!(campo[centro] > 0);
        assert_ne!(campo, apagada);
        campos.push(campo);
    }
    assert_ne!(campos[0], campos[1]);
    assert_ne!(campos[1], campos[2]);
    datos[2] = 7.0;
    datos[8] = 0.0;
    let fina = captura(&gpu, &telar_render::fuego::fragmentos(128).unwrap(), &datos);
    let error: f64 =
        belleza.iter().zip(&fina).map(|(a, b)| f64::from(a.abs_diff(*b))).sum::<f64>() / belleza.len() as f64;
    assert!(error < 3.0, "diferencia media 64/128 pasos: {error}");
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
