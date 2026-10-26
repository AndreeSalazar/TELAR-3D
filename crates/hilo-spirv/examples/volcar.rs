//! Escribe el primer kernel en un archivo .spv, para mirarlo con `spirv-dis`:
//! `cargo run -p hilo-spirv --example volcar -- hash.spv`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(destino) = std::env::args().nth(1) else {
        eprintln!("uso: volcar <archivo.spv>");
        return ExitCode::from(2);
    };
    let palabras = hilo_spirv::ejemplos::llenar_con_hash(64);
    let bytes: Vec<u8> = palabras.iter().flat_map(|w| w.to_le_bytes()).collect();
    match std::fs::write(&destino, bytes) {
        Ok(()) => {
            println!("{} palabras ({} bytes) en {destino}", palabras.len(), palabras.len() * 4);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("no se pudo escribir {destino}: {e}");
            ExitCode::FAILURE
        }
    }
}
