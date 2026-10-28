//! `telar`: la aplicación de TELAR 3D.
//!
//! Es el único sitio que conoce a la vez la ventana, la GPU, los shaders y la
//! geometría (regla `fronteras`): abre la ventana, le pasa a la GPU sus dos
//! manejadores como números y lleva el bucle. La ruta predeterminada genera y
//! deforma una rejilla en GPU; --esfera-cpu conserva el visor de referencia.
//!
//! ```text
//! cargo run -p telar-app --release -- --estudio-rayos  editor de objetos/luz y .tlar v5
//! cargo run -p telar-app --release            el visor (Esc o la X para salir)
//! cargo run -p telar-app -- --prueba          ventana oculta, 120 fotogramas; sale con 1 si hay avisos
//! cargo run -p telar-app -- --sin-validacion  sin las capas de validación (más rápido)
//! cargo run -p telar-app -- --segundos 5      visible, y se cierra sola a los 5 s (para medir los fps)
//! cargo run -p telar-app -- --puntos N        rejilla de unos N puntos (máximo 1024²)
//! cargo run -p telar-app -- --esfera-cpu      esfera de referencia, cocinada en CPU
//! cargo run -p telar-app -- --estilo tema.css personaliza la interfaz nativa
//! cargo run -p telar-app -- --abrir-escena escena.tlar  abre datos de escena v2/v3/v4
//! cargo run -p telar-app -- --guardar-escena escena.tlar guarda al salir; F5/F9 en vivo
//! cargo run -p telar-app --release -- --render-rayos target/imagen.pfm --spp 16
//!                                             Ensayo de mallas, sombras y HDR por lotes
//! cargo run -p telar-app --release -- --calculo [--elementos N]
//!                                             Fase 2: el primer kernel en la GPU, bit a bit contra la CPU
//! ```

mod archivo;
mod arranque;
mod calculo;
mod calidad;
mod controles;
mod entrada_objetos;
mod escena;
mod escena_final;
mod estudio_fuego;
mod estudio_objetos;
mod fuego_archivo;
mod grafo;
mod grafo_compilar;
mod objetos;
mod produccion;
mod proyecto;
mod render;
mod render_asincrono;
mod render_objetos;
mod salida_hdr;
mod sesion;
mod tipografia;
mod visor;
mod vista_objetos;

use std::process::ExitCode;
use std::time::Duration;

use telar_gpu::avisos_de_validacion;

/// Los puntos de la esfera: el millón de la Fase 4, y menos en la prueba (que
/// corre con las capas de validación y sin optimizar).
const PUNTOS: u32 = 1_000_000;
const PUNTOS_DE_PRUEBA: u32 = 40_000;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(resultado) = estudio_fuego::procesar(&args).or_else(|| estudio_objetos::procesar(&args)) {
        return match resultado {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("telar: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(resultado) = produccion::procesar(&args) {
        return match resultado {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("telar: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(resultado) = archivo::procesar(&args) {
        return match resultado {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("telar: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let valor = |nombre: &str| args.iter().position(|a| a == nombre).and_then(|i| args.get(i + 1)?.parse::<u32>().ok());
    let prueba = args.iter().any(|a| a == "--prueba");
    let validacion = !args.iter().any(|a| a == "--sin-validacion");
    if args.iter().any(|a| a == "--calculo") {
        return match calculo::medir(validacion, valor("--elementos").unwrap_or(1 << 24)) {
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
    let estilo = if let Some(indice) = args.iter().position(|a| a == "--estilo") {
        let Some(ruta) = args.get(indice + 1).filter(|ruta| !ruta.starts_with("--")) else {
            eprintln!("telar: --estilo requiere la ruta de una hoja CSS");
            return ExitCode::FAILURE;
        };
        Some(ruta.clone())
    } else {
        None
    };
    let modo = visor::Modo {
        prueba,
        validacion,
        limite: valor("--segundos").map(|s| Duration::from_secs(u64::from(s))),
        puntos: valor("--puntos").unwrap_or(if prueba { PUNTOS_DE_PRUEBA } else { PUNTOS }),
        procedural: !args.iter().any(|a| a == "--esfera-cpu"),
        estilo,
        abrir: match proyecto::argumento(&args, "--abrir-escena") {
            Ok(ruta) => ruta,
            Err(error) => {
                eprintln!("telar: {error}");
                return ExitCode::FAILURE;
            }
        },
        guardar: match proyecto::argumento(&args, "--guardar-escena") {
            Ok(ruta) => ruta,
            Err(error) => {
                eprintln!("telar: {error}");
                return ExitCode::FAILURE;
            }
        },
    };
    match visor::ejecutar(&modo) {
        Ok(r) => {
            let avisos = avisos_de_validacion();
            println!("telar: {} fotogramas · {avisos} avisos de validación", r.cuadros);
            #[allow(clippy::cast_precision_loss, reason = "fotogramas de una sesión: caben en un f32")]
            let fps = r.cuadros as f32 / r.segundos.max(f32::EPSILON);
            println!("telar: {:.1} s · {fps:.1} fps de media", r.segundos);
            if avisos == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Err(e) => {
            eprintln!("telar: {e}");
            ExitCode::FAILURE
        }
    }
}
