//! Render por lotes: radiancia lineal, sombras entre objetos y controles reales.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "verificación con fallos explícitos")]
#[path = "../src/escena_final.rs"]
mod escena_final;
#[path = "../src/produccion.rs"]
mod produccion;
#[path = "../src/salida_hdr.rs"]
mod salida_hdr;
use telar_render::escena_rayos::{Camara, Trabajo, Triangulo};

fn plano(muestras: u32) -> Trabajo {
    Trabajo {
        tamano: [1, 1],
        exposicion: 1.0,
        muestras,
        semilla: 7,
        camara: Camara {
            origen: [0.0, 2.0, 0.0],
            frente: [0.0, -1.0, 0.0],
            derecha: [0.5, 0.0, 0.0],
            arriba: [0.0, 0.0, -0.5],
        },
        triangulos: vec![Triangulo {
            vertices: [[-3.0, 0.0, -3.0], [3.0, 0.0, -3.0], [0.0, 0.0, 3.0]],
            color: [0.3, 0.5, 0.8],
        }],
        luz: [0.0, 3.0, 0.0],
        radio_luz: 0.5,
        energia: 10.0,
        sombras: true,
    }
}

fn validar(codigo: &[u32]) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-final-{}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}

// Reproduce solamente el entero aleatorio publicado, no el integrador GPU.
fn azar(semilla: u32, sal: u32) -> f64 {
    let mut valor = semilla.wrapping_add(sal);
    for (giro, mul) in [(16, 0x7feb_352d), (15, 0x846c_a68b)] {
        valor = (valor ^ (valor >> giro)).wrapping_mul(mul);
    }
    f64::from(valor >> 8) / 16_777_216.0
}

#[test]
fn lambert_gpu_coincide_con_caso_analitico_y_exporta_hdr_sin_recorte() {
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones::default()).unwrap();
    let trabajo = plano(1);
    let codigo = trabajo.compilar().unwrap();
    validar(&codigo);
    let resultado = gpu.ejecutar(&codigo, 5, 64).unwrap();
    assert_eq!(resultado.datos[3], 1);
    let semilla = 7_u32.wrapping_mul(104_729);
    let dx = azar(semilla, 2) - 0.5;
    let dz = azar(semilla, 3) - 0.5;
    let vertical = 3.0 - 0.001;
    let distancia2 = vertical * vertical + dx * dx + dz * dz;
    let peso = 10.0 / core::f64::consts::PI * vertical * vertical / (distancia2 * distancia2);
    for (actual, color) in resultado.datos[..3].iter().zip([0.3, 0.5, 0.8]) {
        let lineal = f64::from(f32::from_bits(*actual));
        assert!((lineal - color * peso).abs() < 2e-6);
    }
    let revision = resultado.datos[4].to_le_bytes();
    for (canal, actual) in revision[..3].iter().enumerate() {
        let lineal = f64::from(f32::from_bits(resultado.datos[canal]));
        let mapeado = lineal / (1.0 + lineal);
        let srgb = if mapeado <= 0.003_130_8 { 12.92 * mapeado } else { 1.055 * mapeado.powf(1.0 / 2.4) - 0.055 };
        assert!((f64::from(*actual) - srgb * 255.0).abs() <= 0.51);
    }
    for spp in [16, 256, 1024] {
        let trabajo = plano(spp);
        let resultado = gpu.ejecutar(&trabajo.compilar().unwrap(), 5, 64).unwrap();
        assert_eq!(resultado.datos[3], spp);
        assert!(resultado.datos[..3].iter().all(|v| f32::from_bits(*v).is_finite()));
    }
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn sombras_entre_objetos_y_ediciones_cambian_la_imagen_gpu() {
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones::default()).unwrap();
    let mut trabajo = plano(8);
    trabajo.tamano = [32, 24];
    trabajo.camara = escena_final::camara([4.0, 3.0, 6.0], [0.0, 0.5, 0.0], 4.0 / 3.0).unwrap();
    trabajo.triangulos = escena_final::triangulos(0.0);
    trabajo.luz = [-1.0, 4.0, 1.0];
    trabajo.radio_luz = 0.65;
    trabajo.energia = 25.0;
    let render = |trabajo: &Trabajo| {
        let codigo = trabajo.compilar().unwrap();
        validar(&codigo);
        gpu.ejecutar(&codigo, 32 * 24 * 5, 64).unwrap().datos
    };
    let sombra = render(&trabajo);
    assert_eq!(sombra, render(&trabajo), "misma entrada, mismos bytes en esta GPU");
    trabajo.exposicion = 2.0;
    let expuesto = render(&trabajo);
    for (original, expuesto) in sombra.as_chunks::<5>().0.iter().zip(expuesto.as_chunks::<5>().0) {
        assert_eq!(&original[..4], &expuesto[..4], "exposición conserva HDR/recuentos");
    }
    assert_ne!(expuesto, sombra, "exposición sí cambia la revisión sRGB");
    trabajo.exposicion = 1.0;
    trabajo.sombras = false;
    let sin = render(&trabajo);
    let mut oscurecidos = 0;
    for (sombra, sin) in sombra.as_chunks::<5>().0.iter().zip(sin.as_chunks::<5>().0) {
        assert_eq!(sombra[3], 8);
        let sombra = f32::from_bits(sombra[0]);
        let sin = f32::from_bits(sin[0]);
        assert!(sombra <= sin + 1e-6);
        oscurecidos += usize::from(sin - sombra > 0.01);
    }
    assert!(oscurecidos > 5, "rayos de sombra deben oscurecer parte del suelo/objetos");
    trabajo.triangulos = escena_final::triangulos(0.8);
    let movido = render(&trabajo);
    assert_ne!(movido, sin);
    trabajo.energia = 0.0;
    let apagado = render(&trabajo);
    assert_ne!(apagado, movido);
    let (pfm, ppm) = salida_hdr::codificar(&sombra, [32, 24], 8).unwrap();
    assert!(pfm.starts_with(b"PF\n32 24\n-1.0\n"));
    assert!(ppm.starts_with(b"P6\n32 24\n255\n"));
    eprintln!("DISTINTO: sombras entre objetos · {oscurecidos} píxeles oscurecidos · {}", gpu.dispositivo());
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn contratos_y_salida_rechazan_invalidez_y_preservan_archivos_existentes() {
    let mut trabajo = plano(1);
    for tamano in [[0, 1], [513, 1], [512, 512]] {
        trabajo.tamano = tamano;
        trabajo.muestras = 1024;
        assert!(trabajo.compilar().is_err());
    }
    trabajo = plano(1);
    trabajo.camara.derecha = [0.0; 3];
    assert!(trabajo.compilar().is_err());
    trabajo = plano(1);
    trabajo.triangulos[0].vertices = [[0.0; 3]; 3];
    assert!(trabajo.compilar().is_err());
    trabajo = plano(1);
    trabajo.energia = f32::NAN;
    assert!(trabajo.compilar().is_err());
    let datos = [1.0_f32.to_bits(), 2.0_f32.to_bits(), 4.0_f32.to_bits(), 1, 0x00e7_d5bc, 0, 0, 0, 1, 0];
    let (pfm, ppm) = salida_hdr::codificar(&datos, [1, 2], 1).unwrap();
    let cabecera = b"PF\n1 2\n-1.0\n";
    assert_eq!(&pfm[cabecera.len()..cabecera.len() + 12], &[0; 12], "primera fila PFM es la inferior");
    assert_eq!(&pfm[pfm.len() - 12..], &datos[..3].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>());
    assert!(salida_hdr::codificar(&datos, [1, 2], 2).is_err());
    let ruta = std::env::temp_dir().join(format!("telar-hdr-test-{}.pfm", std::process::id()));
    std::fs::write(&ruta, b"conservar").unwrap();
    assert!(salida_hdr::guardar(&ruta, &pfm, &ppm, "receta").is_err());
    assert_eq!(std::fs::read(&ruta).unwrap(), b"conservar");
    std::fs::remove_file(ruta).unwrap();
}

#[test]
fn opciones_invalidas_fallan_antes_de_crear_vulkan_o_archivos() {
    for opciones in [
        "--render-rayos",
        "--render-rayos salida.ppm",
        "--render-rayos salida.pfm --spp 0",
        "--render-rayos salida.pfm --spp 3 --spp 4",
        "--render-rayos salida.pfm --ojo 0,0,0 --mirar 0,0,0",
        "--render-rayos salida.pfm --luz NaN,4,1",
        "--render-rayos salida.pfm --exposicion NaN",
        "--render-rayos salida.pfm --prueba",
        "--render-rayos salida.pfm --ancho 512 --alto 512 --spp 256",
    ] {
        let args = opciones.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        assert!(produccion::procesar(&args).unwrap().is_err(), "{opciones}");
    }
    assert!(produccion::procesar(&[]).is_none());
}
