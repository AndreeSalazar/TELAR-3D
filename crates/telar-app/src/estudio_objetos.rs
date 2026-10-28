//! Editor nativo de objetos/luz con vista previa GPU y escena .tlar v5.
use std::{
    path::Path,
    time::{Duration, Instant},
};
use telar_gpu::{Gpu, Opciones, Sombreadores, Superficie};
use telar_ui::{AccionEditor, DisenoObjetos, EditorObjetos};
use telar_ventana::{Config, Evento, Ventana};

pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    if args.iter().any(|a| a == "--render-objetos") {
        return Some(render_archivo(args));
    }
    args.iter().any(|a| a == "--estudio-rayos").then(|| ejecutar(args))
}
fn render_archivo(args: &[String]) -> Result<(), String> {
    let ruta = crate::proyecto::argumento(args, "--render-objetos")?.ok_or("falta escena .tlar v5")?;
    let indice = args.iter().position(|a| a == "--salida").ok_or("falta --salida destino.pfm")?;
    let salida = args.get(indice + 1).filter(|v| !v.starts_with("--")).ok_or("falta destino.pfm")?;
    let escena = crate::objetos::leer(Path::new(&ruta))?;
    crate::render_objetos::exportar(&escena, Path::new(salida), !args.iter().any(|a| a == "--sin-validacion"))?;
    println!("telar: escena v5 renderizada · {} spp · 0 avisos de validación", escena.valores()[30]);
    Ok(())
}
#[allow(clippy::cast_precision_loss, reason = "tamaño del viewport <=1024×768")]
fn ejecutar(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| matches!(a.as_str(), "--render-rayos" | "--calculo" | "--esfera-cpu")) {
        return Err("modo de objetos incompatible con las otras rutas".into());
    }
    let abrir = crate::proyecto::argumento(args, "--abrir-escena")?;
    let guardar = crate::proyecto::argumento(args, "--guardar-escena")?;
    let ruta = Path::new(guardar.as_deref().or(abrir.as_deref()).unwrap_or("objetos.tlar"));
    let mut editor = EditorObjetos::default();
    if let Some(ruta) = &abrir {
        editor.escena = crate::objetos::leer(Path::new(&ruta))?;
    }
    if let Some(i) = args.iter().position(|a| a == "--estilo") {
        let ruta = args.get(i + 1).ok_or("falta hoja CSS")?;
        editor.cargar_estilos(&std::fs::read_to_string(ruta).map_err(|e| e.to_string())?)?;
    }
    let prueba = args.iter().any(|a| a == "--prueba");
    let validacion = !args.iter().any(|a| a == "--sin-validacion");
    let segundos = args
        .iter()
        .position(|a| a == "--segundos")
        .map(|i| args.get(i + 1).ok_or("falta --segundos")?.parse::<u64>().map_err(|_| "--segundos inválido"))
        .transpose()?;
    let (mut ventana, mut gpu, fuente) = preparar(ruta, prueba, validacion, &editor)?;
    let mut aplicada = editor.escena.guardar();
    let mut tamano = gpu.imagenes().1;
    let mut formato = gpu.imagenes().0;
    let mut entrada = crate::entrada_objetos::Entrada::default();
    let mut ui = Vec::new();
    let mut sucio = true;
    let mut ui_tamano = (0, 0);
    let mut cuadros = 0;
    let inicio = Instant::now();
    let mut exportaciones = 0;
    'principal: loop {
        let diseno = DisenoObjetos::nuevo(gpu.imagenes().1);
        for evento in ventana.eventos() {
            sucio = true;
            if let Evento::Tamano { ancho, alto } = evento {
                gpu.redimensionar(ancho, alto);
            } else if entrada.evento(&mut editor, diseno, evento) {
                break 'principal;
            }
        }
        entrada.terminar_lote(&mut editor);
        if let Some(accion) = editor.tomar_accion() {
            sucio = true;
            procesar_accion(&mut editor, ruta, accion, &mut exportaciones, validacion)?;
        }
        let estado = editor.escena.guardar();
        let actual = gpu.imagenes();
        if !editor.arrastrando()
            && !entrada.orbitando
            && (estado[..40] != aplicada[..40] || tamano != actual.1 || formato != actual.0)
        {
            crate::render_objetos::preparar(&mut gpu, &editor.escena, diseno.visor.ancho / diseno.visor.alto)?;
            aplicada = estado;
            tamano = actual.1;
            formato = actual.0;
        }
        if sucio || ui_tamano != actual.1 {
            let nueva = editor.dibujar(diseno, &fuente)?;
            if nueva != ui || ui_tamano != actual.1 {
                gpu.actualizar_interfaz(&nueva, diseno.area())?;
                ui = nueva;
            }
            ui_tamano = actual.1;
            sucio = false;
        }
        if gpu.dibujar_bloque_interfaz([0.04, 0.05, 0.07, 1.0], &[0.0; 32])? {
            cuadros += 1;
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
        if prueba && cuadros == 40 {
            ventana.cambiar_tamano(500, 300);
        }
        if prueba && cuadros == 80 {
            ventana.cambiar_tamano(1280, 800);
        }
        if (prueba && cuadros >= 120) || segundos.is_some_and(|s| inicio.elapsed() >= Duration::from_secs(s)) {
            break;
        }
    }
    if guardar.is_some() {
        crate::objetos::guardar(ruta, &editor.escena)?;
    }
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("editor de objetos con avisos de validación".into());
    }
    println!("telar: objetos · {cuadros} fotogramas · 0 avisos de validación");
    Ok(())
}

fn procesar_accion(
    editor: &mut EditorObjetos,
    ruta: &Path,
    accion: AccionEditor,
    exportaciones: &mut u32,
    validacion: bool,
) -> Result<(), String> {
    let resultado = match accion {
        AccionEditor::Guardar => crate::objetos::guardar(ruta, &editor.escena),
        AccionEditor::Abrir => crate::objetos::leer(ruta).map(|escena| editor.escena = escena),
        AccionEditor::Renderizar => {
            let salida = destino_render(ruta, exportaciones)?;
            crate::render_objetos::exportar(&editor.escena, &salida, validacion)
                .map(|()| println!("telar: HDR {}", salida.display()))
        }
    };
    editor.mensaje = resultado.map_or_else(
        |e| format!("Error: {e}"),
        |()| {
            match accion {
                AccionEditor::Guardar => "Escena guardada",
                AccionEditor::Abrir => "Escena abierta",
                AccionEditor::Renderizar => "HDR / PPM exportados",
            }
            .into()
        },
    );
    editor.soltar();
    Ok(())
}

fn preparar(
    ruta: &Path,
    prueba: bool,
    validacion: bool,
    editor: &EditorObjetos,
) -> Result<(Ventana, Gpu, telar_ui::Fuente), String> {
    let ventana = Ventana::nueva(&Config {
        titulo: format!("TELAR · Objetos · {}", ruta.display()),
        ancho: 1280,
        alto: 800,
        visible: !prueba,
    })?;
    let manejadores = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { validacion, muestras: 1, ..Opciones::default() },
        Superficie { instancia: manejadores.instancia, ventana: manejadores.ventana },
        ventana.tamano(),
    )?;
    let fuente = crate::tipografia::crear()?;
    gpu.preparar_interfaz(Sombreadores {
        vertices: &hilo_spirv::interfaz::vertices(),
        fragmentos: &hilo_spirv::interfaz::fragmentos(),
    })?;
    gpu.fuente_interfaz(telar_gpu::AtlasInterfaz {
        ancho: fuente.ancho,
        alto: fuente.alto,
        cobertura: &fuente.cobertura,
    })?;
    let diseno = DisenoObjetos::nuevo(gpu.imagenes().1);
    crate::render_objetos::preparar(&mut gpu, &editor.escena, diseno.visor.ancho / diseno.visor.alto)?;
    gpu.subir_malla(
        &[[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]],
        &[0, 2, 1],
    )?;
    println!("telar: editor de objetos · {} · .tlar v5 · vista 1 spp · F5/F9/F6", gpu.dispositivo());

    Ok((ventana, gpu, fuente))
}

fn destino_render(ruta: &Path, contador: &mut u32) -> Result<std::path::PathBuf, String> {
    let nombre = ruta.file_stem().ok_or("ruta sin nombre")?.to_string_lossy();
    for _ in 0..4096 {
        *contador = contador.checked_add(1).ok_or("contador de renders agotado")?;
        let salida = ruta.with_file_name(format!("{nombre}-render-{contador:04}.pfm"));
        if ["pfm", "ppm", "txt"].iter().all(|extension| !salida.with_extension(extension).exists()) {
            return Ok(salida);
        }
    }
    Err("no se encontró nombre libre para el render".into())
}
