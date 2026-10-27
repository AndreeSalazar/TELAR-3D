//! El visor: una superficie generada y deformada en GPU, con cámara orbital.
//! El modo --esfera-cpu conserva la referencia de la Fase 4.
//!
//! Arrastrar con el botón izquierdo orbita; la rueda acerca y aleja; quieta
//! unos segundos, la cámara gira sola.

use std::time::{Duration, Instant};

use nudo::Camara;
use nudo::mate::normalizar;
use telar_gpu::eleccion::hex;
use telar_gpu::{Constantes, Gpu, Opciones, ParametrosProcedurales, Superficie};
use telar_ventana::{Boton, Config, Evento, Tecla, Ventana};

/// El negro del cosmos, la brasa (lo que un día será fuego) y la luz ambiente.
const FONDO: u32 = 0x05_070a;
const BRASA: u32 = 0xff_8a3d;
const CIELO: u32 = 0x4a_5a78;
const SUELO: u32 = 0x20_1812;
/// Fotogramas del modo `--prueba`.
const CUADROS_DE_PRUEBA: u32 = 120;
/// Grados que gira la cámara por píxel arrastrado.
const GRADOS_POR_PIXEL: f32 = 0.3;
/// Lo que acerca cada muesca de la rueda.
const ZOOM_POR_MUESCA: f32 = 0.88;
/// Sin tocar el ratón tanto tiempo, la cámara gira sola…
const QUIETO: Duration = Duration::from_secs(3);
/// … a tantos grados por segundo.
const GIRO_SOLO: f32 = 12.0;

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
    let titulo = format!("TELAR 3D · {puntos} puntos · {}", gpu.dispositivo());

    let mut camara = Camara::default();
    let luz = normalizar([0.45, 0.8, 0.4]);
    let (mut arrastrando, mut raton, mut tocado) = (false, None::<(i32, i32)>, None::<Instant>);
    let (inicio, mut anterior, mut segundo) = (Instant::now(), Instant::now(), Instant::now());
    let (mut cuadros, mut en_este_segundo) = (0u32, 0u32);
    'principal: loop {
        for evento in ventana.eventos() {
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'principal,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                Evento::Boton { boton: Boton::Izquierdo, pulsado } => {
                    (arrastrando, tocado) = (pulsado, Some(Instant::now()));
                }
                Evento::Raton { x, y } => {
                    if let (true, Some((x0, y0))) = (arrastrando, raton) {
                        #[allow(clippy::cast_precision_loss, reason = "píxeles de un arrastre: caben en un f32")]
                        let (dx, dy) = ((x - x0) as f32, (y - y0) as f32);
                        camara.orbitar(-dx * GRADOS_POR_PIXEL, dy * GRADOS_POR_PIXEL);
                        tocado = Some(Instant::now());
                    }
                    raton = Some((x, y));
                }
                Evento::Rueda(muescas) => {
                    camara.acercar(ZOOM_POR_MUESCA.powf(muescas));
                    tocado = Some(Instant::now());
                }
                Evento::Tecla(_) | Evento::Boton { .. } => {}
            }
        }
        let dt = anterior.elapsed().as_secs_f32();
        anterior = Instant::now();
        if !arrastrando && tocado.is_none_or(|t| t.elapsed() >= QUIETO) {
            camara.orbitar(GIRO_SOLO * dt, 0.0);
        }
        let (_, (ancho, alto)) = gpu.imagenes();
        #[allow(clippy::cast_precision_loss, reason = "el tamaño de la ventana cabe de sobra en un f32")]
        let aspecto = ancho.max(1) as f32 / alto.max(1) as f32;
        let constantes =
            Constantes { matriz: camara.matriz(aspecto), luz, color: hex(BRASA), cielo: hex(CIELO), suelo: hex(SUELO) };
        let dibujado = if modo.procedural {
            // La prueba cambia el tamaño lógico dentro de capacidad sin volver a
            // grabar compute. La GPU calcula los recuentos y todos los argumentos.
            let lado = if modo.prueba && (40..80).contains(&cuadros) { capacidad / 2 } else { capacidad };
            let parametros =
                ParametrosProcedurales { lado, fase: inicio.elapsed().as_secs_f32() % 1000.0, amplitud: 0.35 };
            gpu.dibujar_procedural(hex(FONDO), &constantes, parametros)?
        } else {
            gpu.dibujar_escena(hex(FONDO), &constantes)?
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
