//! Laboratorios nativos de humo 2D/3D; física residente, inspección solo por solicitud.
use crate::humo_vista;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use telar_ui::{EditorHumo, EntradaNumero};
use telar_ventana::{Boton, Evento, Tecla};
pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    args.iter().any(|v| v == "--humo" || v == "--humo-3d").then(|| ejecutar(args))
}
fn ejecutar(args: &[String]) -> Result<(), String> {
    validar_modo(args)?;
    let abrir = crate::proyecto::argumento(args, "--abrir-escena")?;
    let guardar = crate::proyecto::argumento(args, "--guardar-escena")?;
    let volumen = args.iter().any(|v| v == "--humo-3d");
    let ruta =
        Path::new(guardar.as_deref().or(abrir.as_deref()).unwrap_or(if volumen { "humo3d.tlar" } else { "humo.tlar" }));
    let (captura, limite, objetivo) = opciones(args)?;
    let mut editor = EditorHumo::default();
    editor.volumen = volumen;
    if let Some(ruta) = &abrir {
        editor.humo = leer(Path::new(ruta), volumen)?;
    }
    let prueba = args.iter().any(|v| v == "--prueba");
    let fuente = crate::tipografia::crear()?;
    let (mut ventana, mut gpu) = humo_vista::preparar(
        &editor,
        &fuente,
        !prueba && captura.is_none(),
        !args.iter().any(|v| v == "--sin-validacion"),
    )?;
    let mut entrada = crate::humo_entrada::Entrada::default();
    let mut estado = Estado { paso: false, iteraciones: humo_vista::iteraciones(&editor), pasos: 0 };
    let mut tamano = (0, 0);
    let mut cuadros = 0;
    let inicio = Instant::now();
    'vivo: loop {
        let d = humo_vista::diseno_editor(&editor, gpu.imagenes().1);
        let mut sucio = tamano != d.tamano;
        let mut accion = None;
        for evento in ventana.eventos() {
            sucio = true;
            if editor.editando()
                && let Some(numero) = numero(evento)
            {
                editor.numero(numero);
                continue;
            }
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'vivo,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                Evento::Tecla(Tecla::Otra(v @ (116 | 120 | 82 | 68 | 78 | 32))) => accion = Some(v),
                Evento::Raton { x, y } => entrada.mover(&mut editor, d, x, y),
                Evento::Boton { boton: Boton::Izquierdo, pulsado: true } => entrada.pulsar(&mut editor, d),
                Evento::Boton { boton: Boton::Izquierdo, pulsado: false } => entrada.soltar(&mut editor),
                Evento::Rueda(pasos) => entrada.rueda(&editor, d, pasos),
                _ => {}
            }
        }
        if let Some(accion) = accion {
            editor.numero(EntradaNumero::Confirmar);
            if !editor.editando() {
                atender(accion, &mut editor, &mut estado, &gpu, ruta);
            }
        }
        let iteraciones = humo_vista::iteraciones(&editor);
        if iteraciones != estado.iteraciones {
            gpu.secuencia_campos(&humo_vista::secuencia(iteraciones)?)?;
            estado.iteraciones = iteraciones;
        }
        if sucio {
            gpu.actualizar_interfaz(&editor.dibujar(d, &fuente)?, d.area())?;
            tamano = d.tamano;
        }
        gpu.activar_campos(editor.animando || estado.paso)?;
        let srgb = gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB;
        if gpu.dibujar_bloque_interfaz([0.03, 0.04, 0.05, 1.0], &entrada.bloque(&editor, srgb)?)? {
            cuadros += 1;
            if prueba && cuadros == 40 {
                ventana.cambiar_tamano(500, 300);
            }
            if prueba && cuadros == 80 {
                ventana.cambiar_tamano(1280, 800);
            }
            if editor.animando || estado.paso {
                estado.pasos += 1;
                estado.paso = false;
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
        if (prueba && cuadros >= 120)
            || (captura.is_some() && estado.pasos >= objetivo)
            || limite.is_some_and(|s| inicio.elapsed() >= Duration::from_secs(s))
        {
            break;
        }
    }
    if let Some(destino) = captura {
        editor.mensaje = humo_vista::datos_editor(&gpu, &editor)?;
        println!("telar: {} · {} pasos", editor.mensaje, estado.pasos);
        humo_vista::capturar_bloque(&mut gpu, &editor, &fuente, Path::new(&destino), &entrada.bloque(&editor, true)?)?;
    }
    if guardar.is_some() {
        guardar_editor(ruta, &editor)?;
    }
    finalizar(gpu, ventana)?;
    let dimension = if volumen { "3D" } else { "2D" };
    println!("telar: humo MAC {dimension} · {cuadros} fotogramas · {} pasos · 0 avisos de validación", estado.pasos);
    Ok(())
}
fn finalizar(gpu: telar_gpu::Gpu, ventana: telar_ventana::Ventana) -> Result<(), String> {
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("humo con avisos Vulkan".into());
    }
    Ok(())
}
struct Estado {
    paso: bool,
    iteraciones: u32,
    pasos: u32,
}
fn atender(tecla: u32, editor: &mut EditorHumo, estado: &mut Estado, gpu: &telar_gpu::Gpu, ruta: &Path) {
    let resultado = match tecla {
        116 => guardar_editor(ruta, editor).map(|()| "Configuración guardada; sin caché de campos".into()),
        120 => leer(ruta, editor.volumen).and_then(|humo| {
            gpu.limpiar_campos()?;
            editor.humo = humo;
            estado.pasos = 0;
            Ok("Configuración abierta; campos reiniciados".into())
        }),
        82 => gpu.limpiar_campos().map(|()| {
            estado.pasos = 0;
            "Campos reiniciados en GPU".into()
        }),
        68 => humo_vista::datos_editor(gpu, editor),
        78 => {
            editor.animando = false;
            estado.paso = true;
            Ok("Avance de un paso fijo".into())
        }
        _ => {
            editor.animando = !editor.animando;
            Ok(if editor.animando { "Simulación activa" } else { "Pausada · N: paso · D: datos" }.into())
        }
    };
    editor.mensaje = resultado.unwrap_or_else(|e| format!("Error: {e}"));
}
fn valor(args: &[String], nombre: &str) -> Result<Option<String>, String> {
    let mut indices = args.iter().enumerate().filter(|(_, v)| v.as_str() == nombre);
    let Some((i, _)) = indices.next() else { return Ok(None) };
    if indices.next().is_some() {
        return Err(format!("{nombre} repetido"));
    }
    Ok(Some(args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or_else(|| format!("falta {nombre}"))?.clone()))
}
fn numero(evento: Evento) -> Option<EntradaNumero> {
    match evento {
        Evento::Texto('\r' | '\n') => Some(EntradaNumero::Confirmar),
        Evento::Texto('\u{8}') => Some(EntradaNumero::Retroceso),
        Evento::Texto(c) => Some(EntradaNumero::Caracter(c)),
        Evento::Tecla(Tecla::Escape) => Some(EntradaNumero::Cancelar),
        Evento::Tecla(Tecla::Otra(46)) => Some(EntradaNumero::Borrar),
        _ => None,
    }
}
fn opciones(args: &[String]) -> Result<(Option<String>, Option<u64>, u32), String> {
    let captura = valor(args, "--captura-humo")?;
    let limite = valor(args, "--segundos")?.map(|v| v.parse::<u64>().map_err(|_| "segundos inválidos")).transpose()?;
    let objetivo = valor(args, "--pasos")?.map_or(Ok(240), |v| v.parse::<u32>().map_err(|_| "pasos inválidos"))?;
    if !(1..=2000).contains(&objetivo) {
        return Err("pasos fuera de 1..2000".into());
    }
    Ok((captura, limite, objetivo))
}

fn validar_modo(args: &[String]) -> Result<(), String> {
    if args.iter().any(|v| v == "--humo") && args.iter().any(|v| v == "--humo-3d") {
        return Err("elige humo 2D o 3D".into());
    }
    if args.iter().any(|v| {
        matches!(
            v.as_str(),
            "--fuego" | "--estudio-rayos" | "--render-objetos" | "--render-rayos" | "--calculo" | "--esfera-cpu"
        )
    }) {
        return Err("--humo requiere su propio laboratorio".into());
    }
    Ok(())
}

fn guardar_editor(ruta: &Path, editor: &EditorHumo) -> Result<(), String> {
    if editor.volumen {
        crate::volumen_archivo::guardar(ruta, &editor.humo)
    } else {
        crate::humo_archivo::guardar(ruta, &editor.humo)
    }
}
fn leer(ruta: &Path, volumen: bool) -> Result<telar_ui::Humo, String> {
    if volumen { crate::volumen_archivo::leer(ruta) } else { crate::humo_archivo::leer(ruta) }
}
