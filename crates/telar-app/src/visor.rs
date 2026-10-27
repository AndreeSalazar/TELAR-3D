//! El visor: una superficie generada y deformada en GPU, con cámara orbital.
//! El modo --esfera-cpu conserva la referencia de la Fase 4.
//!
//! Arrastrar con el botón izquierdo orbita; la rueda acerca y aleja; quieta
//! unos segundos, la cámara gira sola.

use std::time::{Duration, Instant};

use crate::controles::Controles;
use nudo::mate::normalizar;
use telar_gpu::eleccion::hex;
use telar_gpu::{Constantes, Gpu, Opciones, ParametrosProcedurales, Sombreadores, Superficie};
use telar_ventana::{Config, Evento, Tecla, Ventana};

/// El negro del cosmos, la brasa (lo que un día será fuego) y la luz ambiente.
const FONDO: u32 = 0x0b_1118;
const BRASA: u32 = 0xff_8a3d;
const CIELO: u32 = 0x4a_5a78;
const SUELO: u32 = 0x20_1812;
/// Fotogramas del modo `--prueba`.
const CUADROS_DE_PRUEBA: u32 = 120;

/// Cómo se abre el visor.
pub(crate) struct Modo {
    pub(crate) procedural: bool,
    /// Ventana oculta, 120 fotogramas y dos cambios de tamaño.
    pub(crate) prueba: bool,
    pub(crate) validacion: bool,
    /// Se cierra sola pasado este tiempo.
    pub(crate) limite: Option<Duration>,
    /// Puntos aproximados de la superficie; la rejilla admite hasta 1024².
    pub(crate) puntos: u32,
    /// Personalizaciones CSS cargadas una vez al abrir el estudio.
    pub(crate) estilo: Option<String>,
}

/// Lo que pasó.
pub(crate) struct Resumen {
    pub(crate) cuadros: u32,
    pub(crate) segundos: f32,
}

/// Abre la ventana y lleva el bucle del visor.
pub(crate) fn ejecutar(modo: &Modo) -> Result<Resumen, String> {
    let config = Config { titulo: "TELAR 3D".to_owned(), ancho: 1280, alto: 720, visible: !modo.prueba };
    let mut ventana = Ventana::nueva(&config)?;
    let m = ventana.manejadores();
    let opciones = Opciones { validacion: modo.validacion, ..Opciones::default() };
    let mut gpu = Gpu::nueva(&opciones, Superficie { instancia: m.instancia, ventana: m.ventana }, ventana.tamano())?;
    let (formato, (ancho, alto)) = gpu.imagenes();
    println!(
        "telar: {} · validación {} · {ancho}×{alto} · formato {formato}",
        gpu.dispositivo(),
        if gpu.validacion() { "activa" } else { "apagada" }
    );
    let (capacidad, puntos) = crate::escena::preparar(&mut gpu, modo)?;
    println!("telar: antialias MSAA {}x · resolución nativa", gpu.muestras()?);
    let titulo = format!("TELAR 3D · {puntos} puntos · {}", gpu.dispositivo());

    let fuente = if modo.procedural { Some(crate::tipografia::crear()?) } else { None };
    preparar_interfaz(&mut gpu, fuente.as_ref())?;
    let mut controles = Controles::nuevo(modo.procedural);
    controles.estudio.salida.muestras_reales = gpu.muestras()?;
    let mut muestras_aplicadas = 4;
    controles.estudio.configurar_capacidad(capacidad);
    let fondo = cargar_estilos(modo, &mut controles)?;
    let mut tamano_ui = (0, 0);
    let luz = normalizar([0.45, 0.8, 0.4]);
    let (inicio, mut anterior, mut segundo) = (Instant::now(), Instant::now(), Instant::now());
    let (mut cuadros, mut en_este_segundo) = (0u32, 0u32);
    'principal: loop {
        let diseno = controles.estudio.distribucion(gpu.imagenes().1);
        for evento in ventana.eventos() {
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'principal,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                evento => controles.evento(evento, diseno),
            }
        }
        let dt = anterior.elapsed().as_secs_f32();
        if modo.procedural {
            crate::calidad::sincronizar(&mut gpu, &mut controles, &mut muestras_aplicadas)?;
        }
        anterior = Instant::now();
        controles.avanzar(dt);
        let (_, (ancho, alto)) = gpu.imagenes();
        #[allow(clippy::cast_precision_loss, reason = "el tamaño de la ventana cabe de sobra en un f32")]
        let aspecto = if modo.procedural {
            diseno.visor.ancho / diseno.visor.alto
        } else {
            ancho.max(1) as f32 / alto.max(1) as f32
        };
        if modo.procedural && (controles.sucio || tamano_ui != (ancho, alto)) {
            let fuente = fuente.as_ref().ok_or("falta la fuente de interfaz")?;
            let triangulos =
                controles.estudio.dibujar_con_fuente(diseno, controles.estudio.rejilla.lado.pow(2), fuente)?;
            gpu.actualizar_interfaz(&triangulos, diseno.area())?;
            controles.sucio = false;
            tamano_ui = (ancho, alto);
        }
        let dibujado = if modo.procedural {
            // La prueba cambia el tamaño lógico dentro de capacidad sin volver a
            // grabar compute. La GPU calcula los recuentos y todos los argumentos.
            let lado = if modo.prueba && (40..80).contains(&cuadros) {
                capacidad / 2
            } else {
                controles.estudio.rejilla.lado.min(capacidad)
            };
            dibujar_superficie(&mut gpu, &controles, aspecto, lado, fondo)?
        } else {
            let constantes = Constantes {
                matriz: controles.camara.matriz(aspecto),
                luz,
                color: hex(BRASA),
                cielo: hex(CIELO),
                suelo: hex(SUELO),
            };
            gpu.dibujar_escena(fondo, &constantes)?
        };
        if dibujado {
            cuadros += 1;
            en_este_segundo += 1;
        } else {
            std::thread::sleep(Duration::from_millis(16)); // minimizada: no gastar la CPU
        }
        if segundo.elapsed() >= Duration::from_secs(1) {
            ventana.poner_titulo(&format!("{titulo} · {en_este_segundo} fps"));
            (segundo, en_este_segundo) = (Instant::now(), 0);
        }
        if (modo.prueba && cuadros >= CUADROS_DE_PRUEBA) || modo.limite.is_some_and(|l| inicio.elapsed() >= l) {
            break;
        }
        // En la prueba, la ventana cambia de tamaño dos veces: la swapchain (y su
        // profundidad) se rehace bajo las capas de validación, donde suelen saltar los avisos.
        if modo.prueba && (cuadros == CUADROS_DE_PRUEBA / 3 || cuadros == 2 * CUADROS_DE_PRUEBA / 3) {
            let (ancho, alto) = if cuadros == CUADROS_DE_PRUEBA / 3 { (800, 600) } else { (1280, 720) };
            ventana.cambiar_tamano(ancho, alto);
        }
    }
    let segundos = inicio.elapsed().as_secs_f32();
    let (_, (ancho, alto)) = gpu.imagenes();
    println!("telar: swapchain creada {} veces · tamaño final {ancho}×{alto}", gpu.swapchains_creadas());
    if modo.procedural {
        crate::escena::comprobar(&gpu, cuadros)?;
    }
    // La GPU antes que la ventana: la superficie de Vulkan vive dentro de la ventana.
    drop(gpu);
    drop(ventana);
    Ok(Resumen { cuadros, segundos })
}

fn cargar_estilos(modo: &Modo, controles: &mut Controles) -> Result<[f32; 4], String> {
    if let Some(ruta) = &modo.estilo {
        let css = std::fs::read_to_string(ruta).map_err(|e| format!("leer estilos {ruta}: {e}"))?;
        controles.estudio.cargar_estilos(&css)?;
    }
    Ok(if modo.procedural { controles.estudio.fondo_visor() } else { hex(FONDO) })
}

fn preparar_interfaz(gpu: &mut Gpu, fuente: Option<&telar_ui::Fuente>) -> Result<(), String> {
    if let Some(fuente) = fuente {
        let (vertices, fragmentos) = (hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos());
        gpu.preparar_interfaz(Sombreadores { vertices: &vertices, fragmentos: &fragmentos })?;
        gpu.fuente_interfaz(telar_gpu::AtlasInterfaz {
            ancho: fuente.ancho,
            alto: fuente.alto,
            cobertura: &fuente.cobertura,
        })?;
    }
    Ok(())
}

fn dibujar_superficie(
    gpu: &mut Gpu,
    controles: &Controles,
    aspecto: f32,
    lado: u32,
    fondo: [f32; 4],
) -> Result<bool, String> {
    let estudio = &controles.estudio;
    let parametros = telar_gpu::ParametrosNodos {
        superficie: ParametrosProcedurales { lado, fase: controles.fase, amplitud: estudio.onda.amplitud },
        valores: [estudio.relieve.factor, 0.0, 0.0, 0.0],
    };
    let bloque = crate::render::bloque(
        &controles.camara,
        aspecto,
        estudio.color(),
        &estudio.salida,
        gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB,
    )?;
    gpu.dibujar_nodos(fondo, &bloque, parametros)
}
