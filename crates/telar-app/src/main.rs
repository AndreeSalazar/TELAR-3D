//! `telar`: la aplicación de TELAR 3D.
//!
//! Es el único sitio que conoce a la vez la ventana y la GPU (regla
//! `fronteras`): abre la ventana, le pasa a la GPU sus dos manejadores como
//! números y lleva el bucle. Fase 1: la pantalla respira del negro del cosmos
//! al cian, sincronizada con el monitor, con cero avisos de validación.
//!
//! ```text
//! cargo run -p telar-app --release            la ventana (Esc o la X para salir)
//! cargo run -p telar-app -- --prueba          ventana oculta, 120 fotogramas; sale con 1 si hay avisos
//! cargo run -p telar-app -- --sin-validacion  sin las capas de validación (más rápido)
//! cargo run -p telar-app -- --segundos 5      visible, y se cierra sola a los 5 s
//! cargo run -p telar-app --release -- --calculo [--elementos N]
//!                                             Fase 2: el primer kernel en la GPU, bit a bit contra la CPU
//! ```

mod calculo;

use std::process::ExitCode;
use std::time::{Duration, Instant};

use telar_gpu::eleccion::{hex, mezclar};
use telar_gpu::{Gpu, Opciones, Superficie, avisos_de_validacion};
use telar_ventana::{Config, Evento, Tecla, Ventana};

/// El negro del cosmos y el cian de la casa.
const FONDO: u32 = 0x05_070a;
const CIAN: u32 = 0x35_e3ff;
/// Fotogramas del modo `--prueba`.
const CUADROS_DE_PRUEBA: u32 = 120;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let prueba = args.iter().any(|a| a == "--prueba");
    let validacion = !args.iter().any(|a| a == "--sin-validacion");
    // `--segundos N`: se cierra sola (para medir los fps con la ventana visible).
    let segundos = args.iter().position(|a| a == "--segundos").and_then(|i| args.get(i + 1)?.parse::<u64>().ok());
    if args.iter().any(|a| a == "--calculo") {
        let elementos = args.iter().position(|a| a == "--elementos").and_then(|i| args.get(i + 1)?.parse::<u32>().ok());
        return match calculo::medir(validacion, elementos.unwrap_or(1 << 24)) {
            Ok(iguales) => {
                let avisos = avisos_de_validacion();
                println!("telar: {avisos} avisos de validación");
                if iguales && avisos == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
            }
            Err(e) => {
                eprintln!("telar: {e}");
                ExitCode::FAILURE
            }
        };
    }
    match ejecutar(prueba, validacion, segundos.map(Duration::from_secs)) {
        Ok(cuadros) => {
            let avisos = avisos_de_validacion();
            println!("telar: {cuadros} fotogramas · {avisos} avisos de validación");
            if avisos == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Err(e) => {
            eprintln!("telar: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Abre todo y lleva el bucle. Devuelve los fotogramas dibujados.
fn ejecutar(prueba: bool, validacion: bool, limite: Option<Duration>) -> Result<u32, String> {
    let config = Config { titulo: "TELAR 3D".to_owned(), ancho: 1280, alto: 720, visible: !prueba };
    let mut ventana = Ventana::nueva(&config)?;
    let m = ventana.manejadores();
    let opciones = Opciones { validacion, ..Opciones::default() };
    let mut gpu = Gpu::nueva(&opciones, Superficie { instancia: m.instancia, ventana: m.ventana }, ventana.tamano())?;
    let (formato, (ancho, alto)) = gpu.imagenes();
    println!(
        "telar: {} · validación {} · {ancho}×{alto} · formato {formato}",
        gpu.dispositivo(),
        if gpu.validacion() { "activa" } else { "apagada" }
    );

    let inicio = Instant::now();
    let mut segundo = Instant::now();
    let (mut cuadros, mut en_este_segundo) = (0u32, 0u32);
    'principal: loop {
        for evento in ventana.eventos() {
            match evento {
                Evento::Cerrar | Evento::Tecla(Tecla::Escape) => break 'principal,
                Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
                Evento::Tecla(_) | Evento::Raton { .. } => {}
            }
        }
        // Respira: del negro al cian y vuelta, cada ~5 s.
        let t = inicio.elapsed().as_secs_f32();
        let color = mezclar(hex(FONDO), hex(CIAN), 0.5 - 0.5 * (t * 1.25).cos());
        if gpu.dibujar(color)? {
            cuadros += 1;
            en_este_segundo += 1;
        } else {
            std::thread::sleep(Duration::from_millis(16)); // minimizada: no gastar la CPU
        }
        if segundo.elapsed() >= Duration::from_secs(1) {
            ventana.poner_titulo(&format!("TELAR 3D · {} · {en_este_segundo} fps", gpu.dispositivo()));
            (segundo, en_este_segundo) = (Instant::now(), 0);
        }
        if (prueba && cuadros >= CUADROS_DE_PRUEBA) || limite.is_some_and(|l| inicio.elapsed() >= l) {
            break;
        }
        // En la prueba, la ventana cambia de tamaño dos veces: la swapchain se rehace
        // bajo las capas de validación, que es donde suelen aparecer los avisos.
        if prueba && (cuadros == CUADROS_DE_PRUEBA / 3 || cuadros == 2 * CUADROS_DE_PRUEBA / 3) {
            let (ancho, alto) = if cuadros == CUADROS_DE_PRUEBA / 3 { (800, 600) } else { (1280, 720) };
            ventana.cambiar_tamano(ancho, alto);
        }
    }
    let (_, (ancho, alto)) = gpu.imagenes();
    println!("telar: swapchain creada {} veces · tamaño final {ancho}×{alto}", gpu.swapchains_creadas());
    // La GPU antes que la ventana: la superficie de Vulkan vive dentro de la ventana.
    drop(gpu);
    drop(ventana);
    Ok(cuadros)
}
