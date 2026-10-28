//! Editor de objetos: archivo, teclado real, escena compartida y GPU.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp, reason = "verificación explícita de integración")]
#[path = "../src/entrada_objetos.rs"]
mod entrada_objetos;
#[path = "../src/escena_final.rs"]
#[allow(dead_code, reason = "la escena editada no usa el antiguo constructor fijo")]
mod escena_final;
#[path = "../src/objetos.rs"]
mod objetos;
#[path = "../src/proyecto.rs"]
#[allow(dead_code, reason = "se reutiliza E/S atómica y CRC, no la escena procedural histórica")]
mod proyecto;
#[path = "../src/render_objetos.rs"]
mod render_objetos;
#[path = "../src/salida_hdr.rs"]
mod salida_hdr;
use telar_ui::{AccionEditor, DisenoObjetos, EditorObjetos, EscenaObjetos};
use telar_ventana::{Boton, Evento, Tecla};

fn ruta(nombre: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("telar-objetos-{}-{nombre}.tlar", std::process::id()))
}
#[test]
fn archivo_v5_valida_crc_dominios_y_guardado_sin_perder_el_anterior() {
    let mut escena = EscenaObjetos::default();
    escena.poner(0, 1.1).unwrap();
    escena.seleccionado = 2;
    escena.poner(4, 32.0).unwrap();
    let bytes = objetos::codificar(&escena).unwrap();
    assert_eq!(bytes.len(), 184);
    assert_eq!(objetos::decodificar(&bytes).unwrap().guardar(), escena.guardar());
    for i in 0..bytes.len() {
        let mut alterado = bytes.clone();
        alterado[i] ^= 1;
        assert!(objetos::decodificar(&alterado).is_err());
    }
    let mut corrupta = bytes.clone();
    corrupta[16..20].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
    let crc = proyecto::crc32(&corrupta[16..]);
    corrupta[12..16].copy_from_slice(&crc.to_le_bytes());
    assert!(objetos::decodificar(&corrupta).is_err());
    let destino = ruta("persistencia");
    objetos::guardar(&destino, &escena).unwrap();
    let anterior = std::fs::read(&destino).unwrap();
    escena.seleccionado = 8;
    assert!(objetos::guardar(&destino, &escena).is_err());
    assert_eq!(std::fs::read(&destino).unwrap(), anterior);
    assert_eq!(objetos::leer(&destino).unwrap().guardar(), objetos::decodificar(&bytes).unwrap().guardar());
    std::fs::remove_file(destino).unwrap();
}

#[test]
fn enter_espera_texto_y_guardar_se_procesa_despues_del_lote() {
    let mut editor = EditorObjetos::default();
    let mut entrada = entrada_objetos::Entrada::default();
    let diseno = DisenoObjetos::nuevo((1280, 800));
    let campo = diseno.valor(0);
    #[allow(clippy::cast_possible_truncation, reason = "coordenadas de ventana acotadas")]
    let posicion = Evento::Raton { x: (campo.x + 4.0) as i32, y: (campo.y + 4.0) as i32 };
    for evento in [
        posicion,
        Evento::Boton { boton: Boton::Izquierdo, pulsado: true },
        Evento::Boton { boton: Boton::Izquierdo, pulsado: false },
        Evento::Tecla(Tecla::Otra(13)),
        Evento::Texto('1'),
        Evento::Texto('.'),
        Evento::Texto('2'),
        Evento::Texto('\r'),
    ] {
        assert!(!entrada.evento(&mut editor, diseno, evento));
    }
    assert_eq!(editor.escena.valores()[0], 1.2);
    assert!(!editor.editando());
    entrada.evento(&mut editor, diseno, Evento::Tecla(Tecla::Otra(116)));
    assert!(editor.tomar_accion().is_none());
    entrada.terminar_lote(&mut editor);
    assert!(editor.tomar_accion() == Some(AccionEditor::Guardar));
}

#[test]
fn vista_previa_y_hdr_reflejan_escena_guardada_y_cambios_de_color_luz() {
    let mut escena = EscenaObjetos::default();
    escena.seleccionado = 3;
    escena.poner(2, 16.0).unwrap();
    escena.poner(3, 16.0).unwrap();
    let gpu =
        telar_gpu::Calculo::nuevo(&telar_gpu::Opciones { muestras: 1, ..telar_gpu::Opciones::default() }).unwrap();
    let render = |escena: &EscenaObjetos| {
        let trabajo = render_objetos::trabajo(escena, None).unwrap();
        gpu.ejecutar(&trabajo.compilar().unwrap(), 16 * 16 * 5, 64).unwrap().datos
    };
    let original = render(&escena);
    let restaurada = objetos::decodificar(&objetos::codificar(&escena).unwrap()).unwrap();
    assert_eq!(original, render(&restaurada));
    escena.seleccionado = 2;
    escena.pagina = 1;
    escena.poner(0, 0.0).unwrap();
    assert_ne!(original, render(&escena));
    let trabajo = render_objetos::trabajo(&escena, None).unwrap();
    let shaders =
        [telar_render::visor_rayos::vertices(), telar_render::visor_rayos::fragmentos(&trabajo, true).unwrap()];
    validar(&shaders[0]);
    validar(&shaders[1]);
    let malla = [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
    let constantes = telar_gpu::Constantes {
        matriz: [[0.0; 4]; 4],
        luz: [0.0; 3],
        color: [0.0; 4],
        cielo: [0.0; 4],
        suelo: [0.0; 4],
    };
    let captura = gpu
        .capturar(
            telar_gpu::Sombreadores { vertices: &shaders[0], fragmentos: &shaders[1] },
            &malla,
            &[0, 2, 1],
            (64, 64),
            [0.0; 4],
            &constantes,
        )
        .unwrap();
    assert!(captura.as_chunks::<4>().0.iter().any(|v| v[1] > 80), "vista previa contiene geometría iluminada");
    let mut ventana =
        telar_ventana::Ventana::nueva(&telar_ventana::Config { visible: false, ..telar_ventana::Config::default() })
            .unwrap();
    let manejadores = ventana.manejadores();
    let mut contexto = telar_gpu::Gpu::nueva(
        &telar_gpu::Opciones { muestras: 1, ..telar_gpu::Opciones::default() },
        telar_gpu::Superficie { instancia: manejadores.instancia, ventana: manejadores.ventana },
        ventana.tamano(),
    )
    .unwrap();
    render_objetos::preparar(&mut contexto, &escena, 1.0).unwrap();
    drop(contexto);
    let _ = ventana.eventos();
    drop(ventana);
    let salida = ruta("hdr").with_extension("pfm");
    render_objetos::exportar(&escena, &salida, true).unwrap();
    for extension in ["pfm", "ppm", "txt"] {
        std::fs::remove_file(salida.with_extension(extension)).unwrap();
    }
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

fn validar(codigo: &[u32]) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let archivo = std::env::temp_dir().join(format!("telar-vista-{}.spv", std::process::id()));
    std::fs::write(&archivo, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida =
        std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&archivo).output().unwrap();
    std::fs::remove_file(archivo).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}

#[test]
fn binario_reabre_escena_v5_y_renderiza_el_mismo_hdr() {
    let mut escena = EscenaObjetos::default();
    escena.poner(0, 1.1).unwrap();
    escena.seleccionado = 3;
    escena.poner(2, 16.0).unwrap();
    escena.poner(3, 16.0).unwrap();
    let origen = ruta("cli-origen");
    let destino = ruta("cli-destino");
    let hdr = ruta("cli-hdr").with_extension("pfm");
    objetos::guardar(&origen, &escena).unwrap();
    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .args(["--estudio-rayos", "--prueba", "--abrir-escena"])
        .arg(&origen)
        .arg("--guardar-escena")
        .arg(&destino)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(String::from_utf8_lossy(&salida.stdout).contains("120 fotogramas · 0 avisos"));
    assert_eq!(std::fs::read(&origen).unwrap(), std::fs::read(&destino).unwrap());
    let salida = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .arg("--render-objetos")
        .arg(&destino)
        .arg("--salida")
        .arg(&hdr)
        .output()
        .unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
    let inspeccion = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .arg("--inspeccionar-tlar")
        .arg(&destino)
        .output()
        .unwrap();
    assert!(inspeccion.status.success());
    assert!(String::from_utf8_lossy(&inspeccion.stdout).contains(".tlar v5"));
    for archivo in [origen, destino, hdr.clone(), hdr.with_extension("ppm"), hdr.with_extension("txt")] {
        std::fs::remove_file(archivo).unwrap();
    }
}
