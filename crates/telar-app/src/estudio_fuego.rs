//! Estudio de volumen procedural; CPU para eventos/constantes, marcha de rayos GPU.
use std::{
    path::Path,
    time::{Duration, Instant},
};
use telar_gpu::{Gpu, Opciones, Sombreadores, Superficie};
use telar_ui::{DisenoObjetos, EditorFuego, EntradaNumero};
use telar_ventana::{Boton, Config, Evento, Tecla, Ventana};

pub(crate) const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
#[allow(clippy::cast_precision_loss, reason = "dimensiones <=1024×768, vista <=3")]
pub(crate) fn bloque(editor: &EditorFuego, area: [u32; 4], srgb: bool) -> [f32; 32] {
    let v = editor.fuego.0;
    let mut bloque = [0.0; 32];
    bloque[..10].copy_from_slice(&[
        v[0],
        v[1],
        v[2],
        v[3],
        v[4],
        v[5],
        v[6],
        area[2] as f32 / area[3].max(1) as f32,
        editor.vista as f32,
        f32::from(u8::from(srgb)),
    ]);
    bloque
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "pasos enteros validados 16..128")]
fn shader(editor: &EditorFuego) -> Result<Vec<u32>, String> {
    editor.fuego.validar()?;
    telar_render::fuego::fragmentos(editor.fuego.0[7] as u32).map_err(Into::into)
}
pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    args.iter().any(|v| v == "--fuego").then(|| ejecutar(args))
}
fn ejecutar(args: &[String]) -> Result<(), String> {
    if args.iter().any(|v| {
        matches!(v.as_str(), "--estudio-rayos" | "--render-rayos" | "--calculo" | "--esfera-cpu" | "--render-objetos")
    }) {
        return Err("--fuego requiere su propio modo de estudio".into());
    }
    let abrir = crate::proyecto::argumento(args, "--abrir-escena")?;
    let guardar = crate::proyecto::argumento(args, "--guardar-escena")?;
    let ruta = Path::new(guardar.as_deref().or(abrir.as_deref()).unwrap_or("fuego.tlar"));
    let mut editor = EditorFuego::default();
    if let Some(ruta) = &abrir {
        editor.fuego = crate::fuego_archivo::leer(Path::new(&ruta))?;
    }
    let fuente = crate::tipografia::crear()?;
    if let Some(i) = args.iter().position(|v| v == "--captura-fuego") {
        let destino = args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or("falta destino .ppm")?;
        return capturar(&editor, &fuente, Path::new(destino));
    }
    let prueba = args.iter().any(|v| v == "--prueba");
    let limite = limite_segundos(args)?;
    let (mut ventana, mut gpu) = preparar(&editor, prueba, !args.iter().any(|v| v == "--sin-validacion"), &fuente)?;
    let vertices = telar_render::visor_rayos::vertices();
    let mut raton = (0, 0);
    let mut tamano = (0, 0);
    let mut pasos = editor.fuego.0[7].to_bits();
    let mut cuadros = 0;
    let inicio = Instant::now();
    let mut ultimo = inicio;
    'vivo: loop {
        let d = DisenoObjetos::nuevo(gpu.imagenes().1);
        let mut sucio = tamano != d.tamano;
        let ahora = Instant::now();
        if editor.animando && !editor.editando() {
            editor.fuego.0[4] = (editor.fuego.0[4] + ahora.duration_since(ultimo).as_secs_f32().min(0.1)) % 60.0;
            sucio = true;
        }
        ultimo = ahora;
        let mut accion = None;
        for evento in ventana.eventos() {
            sucio = true;
            if editor.editando() {
                let numero = numero(evento);
                if let Some(numero) = numero {
                    editor.numero(numero);
                    continue;
                }
            }
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'vivo,
                Evento::Tecla(Tecla::Otra(32)) => editor.animando = !editor.animando,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                Evento::Tecla(Tecla::Otra(v @ (116 | 120))) => accion = Some(v),
                Evento::Raton { x, y } => {
                    raton = (x, y);
                    mover(&mut editor, d, x);
                }
                Evento::Boton { boton: Boton::Izquierdo, pulsado: true } => pulsar(&mut editor, d, raton),
                Evento::Boton { boton: Boton::Izquierdo, pulsado: false } => editor.soltar(),
                _ => {}
            }
        }
        if let Some(accion) = accion {
            editor.numero(EntradaNumero::Confirmar);
            let resultado = if editor.editando() {
                Err("confirma un parámetro válido primero".into())
            } else if accion == 116 {
                crate::fuego_archivo::guardar(ruta, &editor.fuego)
            } else {
                crate::fuego_archivo::leer(ruta).map(|v| editor.fuego = v)
            };
            editor.mensaje =
                resultado.map_or_else(|e| format!("Error: {e}"), |()| format!("Escena: {}", ruta.display()));
        }
        if pasos != editor.fuego.0[7].to_bits() {
            gpu.preparar_visor(Sombreadores { vertices: &vertices, fragmentos: &shader(&editor)? })?;
            pasos = editor.fuego.0[7].to_bits();
        }
        if sucio {
            gpu.actualizar_interfaz(&editor.dibujar(d, &fuente)?, d.area())?;
            tamano = d.tamano;
        }
        let srgb = gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB;
        if gpu.dibujar_bloque_interfaz([0.03, 0.04, 0.05, 1.0], &bloque(&editor, d.area(), srgb))? {
            cuadros += 1;
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
        if (prueba && cuadros >= 120) || limite.is_some_and(|s| inicio.elapsed() >= Duration::from_secs(s)) {
            break;
        }
    }
    if guardar.is_some() {
        crate::fuego_archivo::guardar(ruta, &editor.fuego)?;
    }
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("fuego con avisos Vulkan".into());
    }
    println!("telar: fuego procedural · {cuadros} fotogramas · 0 avisos de validación");
    Ok(())
}
#[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
fn mover(editor: &mut EditorFuego, d: DisenoObjetos, x: i32) {
    editor.mover(d, x as f32);
}
#[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
fn pulsar(editor: &mut EditorFuego, d: DisenoObjetos, p: (i32, i32)) {
    editor.pulsar(d, p.0 as f32, p.1 as f32);
}

fn capturar(editor: &EditorFuego, fuente: &telar_ui::Fuente, ruta: &Path) -> Result<(), String> {
    use std::io::Write;
    if ruta.extension().is_none_or(|v| !v.eq_ignore_ascii_case("ppm")) {
        return Err("captura requiere .ppm".into());
    }
    let gpu = telar_gpu::Calculo::nuevo(&Opciones::default())?;
    let d = DisenoObjetos::nuevo((1280, 800));
    let captura = gpu.capturar_malla_interfaz(
        Sombreadores { vertices: &telar_render::visor_rayos::vertices(), fragmentos: &shader(editor)? },
        (&MALLA, &[0, 2, 1]),
        d.tamano,
        [0.03, 0.04, 0.05, 1.0],
        &bloque(editor, d.area(), true),
        telar_gpu::CapaInterfaz {
            shaders: Sombreadores {
                vertices: &hilo_spirv::interfaz::vertices(),
                fragmentos: &hilo_spirv::interfaz::fragmentos(),
            },
            vertices: &editor.dibujar(d, fuente)?,
            area: d.area(),
            atlas: Some(telar_gpu::AtlasInterfaz {
                ancho: fuente.ancho,
                alto: fuente.alto,
                cobertura: &fuente.cobertura,
            }),
        },
    )?;
    drop(gpu);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("captura con avisos Vulkan".into());
    }
    let mut archivo = std::fs::OpenOptions::new().write(true).create_new(true).open(ruta).map_err(|e| e.to_string())?;
    let mut bytes = b"P6\n1280 800\n255\n".to_vec();
    for pixel in captura.pixeles.as_chunks::<4>().0 {
        bytes.extend_from_slice(&pixel[..3]);
    }
    archivo.write_all(&bytes).map_err(|e| e.to_string())?;
    println!("telar: captura GPU {} · tiempo de dibujo GPU {:?} ns", ruta.display(), captura.nanosegundos);
    Ok(())
}

fn preparar(
    editor: &EditorFuego,
    prueba: bool,
    validacion: bool,
    fuente: &telar_ui::Fuente,
) -> Result<(Ventana, Gpu), String> {
    let vertices = telar_render::visor_rayos::vertices();
    let ventana = Ventana::nueva(&Config {
        titulo: "TELAR · Esfera de fuego · volumen procedural".into(),
        ancho: 1280,
        alto: 800,
        visible: !prueba,
    })?;
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, validacion, ..Opciones::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )?;
    gpu.preparar_visor(Sombreadores { vertices: &vertices, fragmentos: &shader(editor)? })?;
    gpu.subir_malla(&MALLA, &[0, 2, 1])?;
    gpu.preparar_interfaz(Sombreadores {
        vertices: &hilo_spirv::interfaz::vertices(),
        fragmentos: &hilo_spirv::interfaz::fragmentos(),
    })?;
    gpu.fuente_interfaz(telar_gpu::AtlasInterfaz {
        ancho: fuente.ancho,
        alto: fuente.alto,
        cobertura: &fuente.cobertura,
    })?;
    Ok((ventana, gpu))
}

fn limite_segundos(args: &[String]) -> Result<Option<u64>, String> {
    Ok(args
        .iter()
        .position(|v| v == "--segundos")
        .map(|i| args.get(i + 1).ok_or("falta --segundos")?.parse::<u64>().map_err(|_| "--segundos inválido"))
        .transpose()?)
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
