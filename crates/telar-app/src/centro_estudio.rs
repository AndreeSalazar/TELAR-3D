//! Centro por categorías. Abre editores existentes sin cerrar sus escenas.
use std::{
    path::Path,
    process::{Child, Command},
    time::{Duration, Instant},
};
use telar_gpu::{Gpu, Opciones, Sombreadores, Superficie};
use telar_ui::{CATEGORIAS, Catalogo, DisenoCatalogo, Espacio};
use telar_ventana::{Boton, Config, Evento, Tecla, Ventana};
pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    args.iter().any(|v| v == "--estudio").then(|| ejecutar(args))
}
fn ejecutar(args: &[String]) -> Result<(), String> {
    validar_modo(args)?;
    let mut catalogo = Catalogo::default();
    if let Some(categoria) = valor(args, "--categoria")? {
        let indice = categoria.parse::<usize>().map_err(|_| "categoría debe ser 1..8")?;
        if indice == 0 || !catalogo.seleccionar_categoria(indice - 1) {
            return Err("categoría fuera de 1..8".into());
        }
    }
    if let Some(ruta) = valor(args, "--estilo")? {
        catalogo.cargar_estilos(&std::fs::read_to_string(ruta).map_err(|e| e.to_string())?)?;
    }
    let fuente = crate::tipografia::crear()?;
    if let Some(ruta) = valor(args, "--captura-estudio")? {
        return capturar(&catalogo, &fuente, Path::new(&ruta));
    }
    let limite = valor(args, "--segundos")?.map(|v| v.parse::<u64>().map_err(|_| "segundos inválidos")).transpose()?;
    let prueba = args.iter().any(|v| v == "--prueba");
    let validacion = !args.iter().any(|v| v == "--sin-validacion");
    let mut ventana = Ventana::nueva(&Config {
        titulo: "TELAR · Estudio · áreas de trabajo".into(),
        ancho: 1280,
        alto: 800,
        visible: !prueba,
    })?;
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, validacion, ..Default::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )?;
    gpu.preparar_interfaz(Sombreadores {
        vertices: &hilo_spirv::interfaz::vertices(),
        fragmentos: &hilo_spirv::interfaz::fragmentos(),
    })?;
    gpu.fuente_interfaz(telar_gpu::AtlasInterfaz {
        ancho: fuente.ancho,
        alto: fuente.alto,
        cobertura: &fuente.cobertura,
    })?;
    let mut raton = (0, 0);
    let mut tamano = (0, 0);
    let mut abiertos = Vec::<(Espacio, Child)>::new();
    let mut cuadros = 0;
    let inicio = Instant::now();
    'vivo: loop {
        let d = DisenoCatalogo::nuevo(gpu.imagenes().1);
        let mut sucio = tamano != d.tamano;
        for evento in ventana.eventos() {
            sucio |= !matches!(evento, Evento::Raton { .. });
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'vivo,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                Evento::Raton { x, y } => raton = (x, y),
                Evento::Boton { boton: Boton::Izquierdo, pulsado: true } => pulsar(&mut catalogo, d, raton),
                Evento::Tecla(Tecla::Otra(tecla)) => teclado(&mut catalogo, tecla),
                _ => {}
            }
        }
        if let Some(espacio) = catalogo.tomar_accion() {
            catalogo.mensaje =
                abrir(espacio, &mut abiertos, validacion).unwrap_or_else(|e| format!("No se pudo abrir: {e}"));
            sucio = true;
        }
        if sucio {
            gpu.actualizar_interfaz(&catalogo.dibujar(d, &fuente)?, [0, 0, d.tamano.0, d.tamano.1])?;
            tamano = d.tamano;
        }
        if gpu.dibujar_interfaz([0.04, 0.05, 0.07, 1.0])? {
            cuadros += 1;
            if prueba {
                recorrer(&mut catalogo, &ventana, cuadros);
                tamano = (0, 0);
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
        if (prueba && cuadros >= 120) || limite.is_some_and(|s| inicio.elapsed() >= Duration::from_secs(s)) {
            break;
        }
    }
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("centro de estudio con avisos Vulkan".into());
    }
    println!("telar: estudio · {cuadros} fotogramas · 0 avisos de validación");
    Ok(())
}
fn argumentos(espacio: Espacio) -> &'static [&'static str] {
    match espacio {
        Espacio::Objetos => &["--estudio-rayos"],
        Espacio::Geometria => &[],
        Espacio::Humo => &["--humo"],
        Espacio::Humo3d => &["--humo-3d"],
        Espacio::Particulas => &["--particulas"],
        Espacio::Fuego => &["--fuego"],
    }
}
fn abrir(espacio: Espacio, abiertos: &mut Vec<(Espacio, Child)>, validacion: bool) -> Result<String, String> {
    abiertos.retain_mut(|(_, proceso)| proceso.try_wait().ok().flatten().is_none());
    if abiertos.iter().any(|(actual, _)| *actual == espacio) {
        return Ok("Ya hay una ventana abierta de este editor desde el centro.".into());
    }
    let ejecutable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut comando = Command::new(ejecutable);
    comando.args(argumentos(espacio));
    if !validacion {
        comando.arg("--sin-validacion");
    }
    abiertos.push((espacio, comando.spawn().map_err(|e| e.to_string())?));
    Ok("Editor abierto en otra ventana. Cada ventana conserva su propia escena.".into())
}
fn teclado(catalogo: &mut Catalogo, tecla: u32) {
    match tecla {
        49..=56 => {
            catalogo.seleccionar_categoria((tecla - 49) as usize);
        }
        38 => catalogo.siguiente(false),
        40 => catalogo.siguiente(true),
        13 => {
            catalogo.abrir();
        }
        _ => {}
    }
}
#[allow(clippy::cast_precision_loss, reason = "píxeles del ratón")]
fn pulsar(catalogo: &mut Catalogo, d: DisenoCatalogo, p: (i32, i32)) {
    catalogo.pulsar(d, p.0 as f32, p.1 as f32);
}
fn recorrer(catalogo: &mut Catalogo, ventana: &Ventana, cuadros: u32) {
    if cuadros.is_multiple_of(12) {
        catalogo.seleccionar_categoria((cuadros as usize / 12) % CATEGORIAS.len());
    }
    if cuadros == 40 {
        ventana.cambiar_tamano(800, 500);
    }
    if cuadros == 80 {
        ventana.cambiar_tamano(1280, 800);
    }
}
fn valor(args: &[String], nombre: &str) -> Result<Option<String>, String> {
    let mut indices = args.iter().enumerate().filter(|(_, v)| v.as_str() == nombre);
    let Some((i, _)) = indices.next() else { return Ok(None) };
    if indices.next().is_some() {
        return Err(format!("{nombre} repetido"));
    }
    Ok(Some(args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or_else(|| format!("falta {nombre}"))?.clone()))
}
fn capturar(catalogo: &Catalogo, fuente: &telar_ui::Fuente, ruta: &Path) -> Result<(), String> {
    use std::io::Write;
    if ruta.extension().is_none_or(|v| !v.eq_ignore_ascii_case("ppm")) {
        return Err("captura requiere .ppm".into());
    }
    let gpu = telar_gpu::Calculo::nuevo(&Opciones::default())?;
    let d = DisenoCatalogo::nuevo((1280, 800));
    let imagen = gpu.capturar_interfaz(
        d.tamano,
        [0.04, 0.05, 0.07, 1.0],
        telar_gpu::CapaInterfaz {
            shaders: Sombreadores {
                vertices: &hilo_spirv::interfaz::vertices(),
                fragmentos: &hilo_spirv::interfaz::fragmentos(),
            },
            vertices: &catalogo.dibujar(d, fuente)?,
            area: [0, 0, 1280, 800],
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
    let mut bytes = b"P6\n1280 800\n255\n".to_vec();
    for p in imagen.pixeles.as_chunks::<4>().0 {
        bytes.extend_from_slice(&p[..3]);
    }
    let mut archivo = std::fs::OpenOptions::new().write(true).create_new(true).open(ruta).map_err(|e| e.to_string())?;
    archivo.write_all(&bytes).map_err(|e| e.to_string())?;
    println!("telar: captura del estudio {}", ruta.display());
    Ok(())
}
fn validar_modo(args: &[String]) -> Result<(), String> {
    if args.iter().any(|v| {
        matches!(
            v.as_str(),
            "--trazadores"
                | "--humo"
                | "--humo-3d"
                | "--explosion"
                | "--particulas"
                | "--fuego"
                | "--estudio-rayos"
                | "--render-objetos"
                | "--render-rayos"
                | "--calculo"
                | "--esfera-cpu"
                | "--abrir-escena"
                | "--guardar-escena"
        )
    }) {
        return Err("--estudio abre el centro; selecciona el editor desde sus categorías".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destinos_no_reabren_el_catalogo_ni_inventan_comandos() {
        assert_eq!(argumentos(Espacio::Geometria), &[] as &[&str]);
        assert_eq!(argumentos(Espacio::Objetos), &["--estudio-rayos"]);
        assert_eq!(argumentos(Espacio::Humo), &["--humo"]);
        assert_eq!(argumentos(Espacio::Humo3d), &["--humo-3d"]);
        assert_eq!(argumentos(Espacio::Particulas), &["--particulas"]);
        assert_eq!(argumentos(Espacio::Fuego), &["--fuego"]);
    }
}
