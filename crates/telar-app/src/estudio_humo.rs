//! Laboratorios nativos de humo 2D/3D; física residente, inspección solo por solicitud.
use crate::humo_modos::{crear_editor, nombre_editor, predeterminada, validar_modo};
use crate::{
    humo_archivos::{guardar_editor, leer_editor},
    humo_vista,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use telar_ui::{EditorHumo, EntradaNumero};

pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    args.iter()
        .any(|v| {
            v == "--humo"
                || v == "--humo-3d"
                || v == "--explosion"
                || v == "--particulas"
                || v == "--trazadores"
                || v == "--plano-vfx"
                || v == "--abrir-cache"
                || v == "--guardar-cache"
                || v == "--render-vfx"
                || v == "--secuencia-vfx"
        })
        .then(|| ejecutar(args))
}
fn ejecutar(args: &[String]) -> Result<(), String> {
    validar_modo(args)?;
    let cache = crate::humo_cache::Sesion::nueva(args)?;
    let abrir = crate::proyecto::argumento(args, "--abrir-escena")?;
    let guardar = crate::proyecto::argumento(args, "--guardar-escena")?;
    let volumen = es_volumen(args, cache.abrir.is_some());
    let ruta = Path::new(guardar.as_deref().or(abrir.as_deref()).unwrap_or(predeterminada(args, volumen)));
    let (captura, limite, objetivo) = opciones(args)?;
    let exportar = crate::intercambio_archivo::destino(args, volumen)?;
    let mut editor = crear_editor(args, volumen, abrir.as_deref())?;
    cache.configurar(&mut editor)?;
    let mut salida = crate::vfx_salida::Salida::nueva(args)?;
    salida.iniciar(&editor)?;
    let objetivo = salida.objetivo.unwrap_or(objetivo);
    let prueba = args.iter().any(|v| v == "--prueba");
    let fuente = crate::tipografia::crear()?;
    let (mut ventana, mut gpu) = humo_vista::preparar(
        &editor,
        &fuente,
        !prueba && captura.is_none() && exportar.is_none() && cache.guardar.is_none() && !salida.activa(),
        !args.iter().any(|v| v == "--sin-validacion"),
    )?;
    let mut entrada = crate::humo_entrada::Entrada::default();
    if editor.vfx.is_some() {
        entrada.encuadrar_vfx();
    }
    cache.cargar(&gpu, &mut entrada)?;
    let mut monitor = crate::monitor_gpu::Observador::nuevo(args, &editor, &mut gpu)?;
    let mut estado = Estado {
        ajustes: None,
        paso: false,
        secuencia: secuencia(&editor)?,
        controles: [editor.humo.0[7], editor.volumen_opciones.0[0]].map(f32::to_bits),
        pasos: 0,
    };
    let mut tamano = (0, 0);
    let mut cuadros = 0;
    let inicio = Instant::now();
    'vivo: loop {
        let d = humo_vista::diseno_editor(&editor, gpu.imagenes().1);
        let mut sucio = tamano != d.tamano;
        let eventos = crate::humo_eventos::procesar(ventana.eventos(), &mut editor, &mut entrada, &mut gpu, d);
        sucio |= eventos.sucio;
        if eventos.cerrar {
            break 'vivo;
        }
        if eventos.accion == Some(77) {
            monitor.alternar(&mut editor, &mut gpu)?;
        } else if let Some(accion) = eventos.accion {
            editor.numero(EntradaNumero::Confirmar);
            if !editor.editando() {
                atender(accion, &mut editor, &mut estado, &gpu, ruta);
            }
        }
        actualizar_secuencia(&editor, &mut estado, &mut gpu)?;
        sucio |= monitor.actualizar(&mut editor, &gpu, false);
        if sucio {
            gpu.actualizar_interfaz(&editor.dibujar(d, &fuente)?, d.area())?;
            tamano = d.tamano;
        }
        gpu.activar_campos(editor.animando || estado.paso)?;
        let srgb = gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB;
        let envio = Instant::now();
        if gpu.dibujar_bloque_interfaz([0.03, 0.04, 0.05, 1.0], &entrada.bloque(&editor, srgb)?)? {
            monitor.cuadro(&gpu, envio.elapsed());
            cuadros += 1;
            if prueba {
                probar_tamano(&ventana, cuadros);
            }
            if editor.animando || estado.paso {
                estado.pasos += 1;
                salida.cuadro(&mut gpu, &editor, entrada.bloque(&editor, true)?, estado.pasos)?;
                estado.paso = false;
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
        if (prueba && cuadros >= 120)
            || ((captura.is_some() || exportar.is_some() || cache.guardar.is_some() || salida.activa())
                && estado.pasos >= objetivo)
            || limite.is_some_and(|s| inicio.elapsed() >= Duration::from_secs(s))
        {
            break;
        }
    }
    if let Some(destino) = captura {
        monitor.actualizar(&mut editor, &gpu, true);
        editor.mensaje = humo_vista::datos_editor(&gpu, &editor)?;
        println!("telar: {} · {} pasos", editor.mensaje, estado.pasos);
        humo_vista::capturar_bloque(&mut gpu, &editor, &fuente, Path::new(&destino), &entrada.bloque(&editor, true)?)?;
    }
    salida_geo(&gpu, &editor, exportar.as_deref())?;
    salida.finalizar(&mut gpu, &editor, entrada.bloque(&editor, true)?)?;
    cache.finalizar(&gpu, &editor, &entrada, estado.pasos)?;
    guardar_al_salir(guardar.is_some(), ruta, &editor)?;
    monitor.actualizar(&mut editor, &gpu, true);
    monitor.finalizar()?;
    finalizar(gpu, ventana)?;
    let dimension = nombre_editor(&editor);
    println!("telar: {dimension} · {cuadros} fotogramas · {} pasos · 0 avisos de validación", estado.pasos);
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
    ajustes: Option<[u32; 28]>,
    paso: bool,
    secuencia: Vec<usize>,
    controles: [u32; 2],
    pasos: u32,
}
fn atender(tecla: u32, editor: &mut EditorHumo, estado: &mut Estado, gpu: &telar_gpu::Gpu, ruta: &Path) {
    let resultado = match tecla {
        116 => guardar_editor(ruta, editor).map(|()| "Configuración guardada; sin caché de campos".into()),
        119 => exportar_editor(gpu, editor, &ruta.with_extension("geo"))
            .map(|n| format!("{n} puntos GEO exportados · comparación externa ABIERTA")),
        120 => leer_editor(ruta, editor).and_then(|()| {
            gpu.limpiar_campos()?;
            estado.pasos = 0;
            estado.ajustes = None;
            Ok("Configuración abierta; campos reiniciados".into())
        }),
        82 => gpu.limpiar_campos().map(|()| {
            estado.pasos = 0;
            estado.ajustes = None;
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
pub(crate) fn valor(args: &[String], nombre: &str) -> Result<Option<String>, String> {
    let mut indices = args.iter().enumerate().filter(|(_, v)| v.as_str() == nombre);
    let Some((i, _)) = indices.next() else { return Ok(None) };
    if indices.next().is_some() {
        return Err(format!("{nombre} repetido"));
    }
    Ok(Some(args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or_else(|| format!("falta {nombre}"))?.clone()))
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

fn secuencia(editor: &EditorHumo) -> Result<Vec<usize>, String> {
    if editor.particulas.is_some() {
        return Ok(vec![0, 1]);
    }
    if editor.volumen {
        crate::volumen_vista::secuencia(editor)
    } else {
        humo_vista::secuencia(humo_vista::iteraciones(editor))
    }
}
fn actualizar_secuencia(editor: &EditorHumo, estado: &mut Estado, gpu: &mut telar_gpu::Gpu) -> Result<(), String> {
    let ajustes = editor.vfx.map(|v| v.0.map(f32::to_bits));
    if ajustes != estado.ajustes {
        if let Some(v) = ajustes {
            gpu.cargar_rango_campos(19 * 32u32.pow(3), &v)?;
        }
        estado.ajustes = ajustes;
    }
    let controles = [editor.humo.0[7], editor.volumen_opciones.0[0]].map(f32::to_bits);
    if controles == estado.controles {
        return Ok(());
    }
    let nueva = secuencia(editor)?;
    if nueva != estado.secuencia {
        gpu.secuencia_campos(&nueva)?;
        estado.secuencia = nueva;
    }
    estado.controles = controles;
    Ok(())
}

fn exportar_editor(gpu: &telar_gpu::Gpu, editor: &EditorHumo, ruta: &Path) -> Result<usize, String> {
    if editor.vfx.is_some() {
        crate::particulas_geo::exportar_vfx(gpu, ruta)
    } else if editor.trazadores {
        crate::particulas_geo::exportar_trazadores(gpu, crate::volumen_vista::LADO, ruta)
    } else if editor.particulas.is_some() {
        crate::particulas_geo::exportar(gpu, ruta)
    } else if editor.volumen {
        crate::intercambio_archivo::exportar(gpu, crate::volumen_vista::LADO, ruta)
    } else {
        Err("GEO disponible solo en espacios 3D".into())
    }
}
fn salida_geo(gpu: &telar_gpu::Gpu, editor: &EditorHumo, ruta: Option<&str>) -> Result<(), String> {
    if editor.particulas.is_none() && !editor.trazadores {
        return crate::intercambio_archivo::salida(gpu, crate::volumen_vista::LADO, ruta);
    }
    if let Some(ruta) = ruta {
        let n = exportar_editor(gpu, editor, Path::new(ruta))?;
        println!("telar: {n} partículas vivas exportadas a {ruta}");
    }
    Ok(())
}

fn guardar_al_salir(guardar: bool, ruta: &Path, editor: &EditorHumo) -> Result<(), String> {
    if guardar {
        guardar_editor(ruta, editor)?;
    }
    Ok(())
}

fn es_volumen(args: &[String], cache: bool) -> bool {
    cache
        || args.iter().any(|v| {
            matches!(v.as_str(), "--humo-3d" | "--explosion" | "--particulas" | "--trazadores" | "--plano-vfx")
        })
}

fn probar_tamano(ventana: &telar_ventana::Ventana, cuadros: u32) {
    if cuadros == 40 {
        ventana.cambiar_tamano(500, 300);
    }
    if cuadros == 80 {
        ventana.cambiar_tamano(1280, 800);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|v| (*v).into()).collect()
    }
    #[test]
    fn trazadores_requieren_mac_y_conservan_controles_del_gas() -> Result<(), String> {
        assert!(validar_modo(&args(&["--trazadores"])).is_err());
        assert!(validar_modo(&args(&["--humo", "--trazadores"])).is_err());
        assert!(validar_modo(&args(&["--particulas", "--explosion", "--trazadores"])).is_err());
        let parametros = args(&["--explosion", "--trazadores"]);
        validar_modo(&parametros)?;
        let editor = crear_editor(&parametros, true, None)?;
        assert!(editor.trazadores && editor.particulas.is_none());
        assert_eq!(editor.vista, 3);
        assert_eq!(editor.explosion.0[0].to_bits(), 0.35f32.to_bits());
        assert_eq!(predeterminada(&parametros, true), "trazadores.tlar");
        let plan = secuencia(&editor)?;
        assert_eq!(plan, telar_kernel::humo3d::orden_temporal(64, 2)?);
        Ok(())
    }
}
