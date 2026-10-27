//! Integridad, dominios y reemplazo sin perder la escena anterior.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp, reason = "aserciones de integración")]
#[path = "../src/proyecto.rs"]
mod proyecto;
use proyecto::Proyecto;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use telar_ui::Estudio;

fn ruta() -> PathBuf {
    static IDS: AtomicU64 = AtomicU64::new(0);
    let directorio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/pruebas-proyecto");
    fs::create_dir_all(&directorio).unwrap();
    directorio.join(format!("escena-{}-{}.tlar", std::process::id(), IDS.fetch_add(1, Ordering::Relaxed)))
}

fn escena() -> Proyecto {
    let mut estudio = Estudio::default();
    estudio.configurar_capacidad(256);
    estudio.rejilla.lado = 49;
    estudio.salida.muestras = 2;
    estudio.relieve.factor = 2.3;
    estudio.pausado = true;
    Proyecto::capturar(&estudio, &nudo::Camara::default(), 0.6)
}

#[test]
fn ida_y_vuelta_determinista_conserva_todos_los_campos() {
    let original = escena();
    let bytes = original.codificar().unwrap();
    assert_eq!(bytes.len(), 128);
    let copia = Proyecto::decodificar(&bytes).unwrap();
    assert_eq!(copia.estudio, original.estudio);
    assert_eq!(copia.camara, original.camara);
    assert_eq!(copia.fase.to_bits(), original.fase.to_bits());
    assert_eq!(copia.codificar().unwrap(), bytes);
}

#[test]
fn rechaza_truncamiento_sobrantes_corrupcion_y_version_de_arranque() {
    let bytes = escena().codificar().unwrap();
    for longitud in 0..bytes.len() {
        assert!(Proyecto::decodificar(&bytes[..longitud]).is_err());
    }
    for indice in 0..bytes.len() {
        let mut malo = bytes.clone();
        malo[indice] ^= 0x40;
        assert!(Proyecto::decodificar(&malo).is_err(), "byte {indice}");
    }
    let mut malo = bytes.clone();
    malo.push(0);
    assert!(Proyecto::decodificar(&malo).is_err());
    malo = bytes;
    malo[4..8].copy_from_slice(&1u32.to_le_bytes());
    assert!(Proyecto::decodificar(&malo).is_err());
}

#[test]
fn validar_semantica_tambien_antes_de_escribir_y_no_confia_en_crc() {
    let mut p = escena();
    for invalido in [f32::NAN, f32::INFINITY, 0.0, 180.0] {
        p.camara.campo = invalido;
        assert!(p.codificar().is_err());
    }
    p = escena();
    p.fase = 1000.0;
    assert!(p.codificar().is_err());
    p = escena();
    p.estudio[8] = 256;
    assert!(p.codificar().is_err());
}

#[test]
fn guardar_reemplaza_y_un_error_conserva_el_archivo_anterior() {
    let ruta = ruta();
    let mut p = escena();
    proyecto::guardar(&ruta, &p).unwrap();
    let primero = fs::read(&ruta).unwrap();
    p.fase = 2.0;
    proyecto::guardar(&ruta, &p).unwrap();
    assert_ne!(fs::read(&ruta).unwrap(), primero);
    assert_eq!(proyecto::leer(&ruta).unwrap().fase, 2.0);
    let segundo = fs::read(&ruta).unwrap();
    p.camara.distancia = f32::NAN;
    assert!(proyecto::guardar(&ruta, &p).is_err());
    assert_eq!(fs::read(&ruta).unwrap(), segundo);
    fs::write(&ruta, vec![0; 1_000_000]).unwrap();
    assert!(proyecto::leer(&ruta).is_err(), "lector acotado");
    fs::remove_file(ruta).unwrap();
}

#[test]
fn argumentos_con_rutas_unicode_y_espacios_son_inequivocos() {
    let args: Vec<String> = ["--abrir-escena", "mi escena ñ.tlar"].map(str::to_owned).to_vec();
    assert_eq!(proyecto::argumento(&args, "--abrir-escena").unwrap().as_deref(), Some("mi escena ñ.tlar"));
    assert_eq!(proyecto::argumento(&args, "--guardar-escena").unwrap(), None);
    for malos in [
        vec!["--abrir-escena"],
        vec!["--abrir-escena", "--prueba"],
        vec!["--abrir-escena", "a.bin"],
        vec!["--abrir-escena", "a.tlar", "--abrir-escena", "b.tlar"],
    ] {
        let malos: Vec<String> = malos.into_iter().map(str::to_owned).collect();
        assert!(proyecto::argumento(&malos, "--abrir-escena").is_err());
    }
}

#[test]
fn ejecutable_reabre_escena_con_capacidad_y_calidad_guardadas() {
    let entrada = ruta();
    let salida = ruta();
    let original = escena();
    proyecto::guardar(&entrada, &original).unwrap();
    let inspeccion = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .arg("--inspeccionar-tlar")
        .arg(&entrada)
        .output()
        .unwrap();
    assert!(inspeccion.status.success());
    let inspeccion = String::from_utf8_lossy(&inspeccion.stdout);
    assert!(inspeccion.contains(".tlar v2 · escena procedural"));
    assert!(!inspeccion.contains("validación activa"), "inspeccionar no crea GPU");
    let resultado = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .args(["--prueba", "--puntos", "4", "--abrir-escena"])
        .arg(&entrada)
        .arg("--guardar-escena")
        .arg(&salida)
        .output()
        .unwrap();
    let errores = String::from_utf8_lossy(&resultado.stderr);
    if errores.contains("vulkan-1.dll") {
        fs::remove_file(entrada).unwrap();
        return;
    }
    let texto = String::from_utf8_lossy(&resultado.stdout);
    assert!(resultado.status.success(), "{texto}\n{errores}");
    assert!(texto.contains("capacidad 256×256"), "la escena decide la reserva: {texto}");
    assert!(texto.contains("antialias MSAA 2x"), "{texto}");
    assert!(texto.contains("120 fotogramas · 0 avisos de validación"), "{texto}");
    assert_eq!(fs::read(&entrada).unwrap(), fs::read(&salida).unwrap(), "escena pausada idéntica tras 120 cuadros");
    fs::remove_file(entrada).unwrap();
    fs::remove_file(salida).unwrap();
}

#[test]
fn fallo_al_reemplazar_limpia_solo_el_temporal_y_conserva_destino() {
    let destino = ruta();
    fs::create_dir(&destino).unwrap();
    let testigo = destino.join("conservar.txt");
    fs::write(&testigo, "conservar").unwrap();
    assert!(proyecto::guardar(&destino, &escena()).is_err());
    assert_eq!(fs::read_to_string(&testigo).unwrap(), "conservar");
    let prefijo = format!(".{}.", destino.file_name().unwrap().to_string_lossy());
    assert!(
        fs::read_dir(destino.parent().unwrap()).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&prefijo))
    );
    fs::remove_file(testigo).unwrap();
    fs::remove_dir(destino).unwrap();
}
