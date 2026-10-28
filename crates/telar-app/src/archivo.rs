//! Exportación e inspección de arranque .tlar, fuera del bucle de fotogramas.
use std::{fs, io::Read, path::Path};

pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    let exportar = args.iter().position(|a| a == "--exportar-tlar");
    let inspeccionar = args.iter().position(|a| a == "--inspeccionar-tlar");
    let indice = exportar.or(inspeccionar)?;
    Some((|| {
        if exportar.is_some() && inspeccionar.is_some() {
            return Err("elige exportar o inspeccionar .tlar".to_owned());
        }
        let ruta = args.get(indice + 1).filter(|r| !r.starts_with("--")).ok_or("falta ruta .tlar")?;
        if !Path::new(ruta).extension().is_some_and(|e| e.eq_ignore_ascii_case("tlar")) {
            return Err("se requiere extensión .tlar".to_owned());
        }
        if exportar.is_some() {
            let puntos = args
                .iter()
                .position(|a| a == "--puntos")
                .map(|i| args.get(i + 1).ok_or("falta --puntos")?.parse::<u32>().map_err(|_| "--puntos inválido"))
                .transpose()?
                .unwrap_or(1_000_000);
            let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones::default())?;
            let cabecera = crate::arranque::crear(gpu.perfil(), puntos.isqrt().clamp(2, 1024), 4)?;
            let bytes = telar_kernel::paquete::codificar(&cabecera, &telar_kernel::compilar_con_arranque())?;
            fs::write(ruta, bytes).map_err(|e| format!("escribir {ruta}: {e}"))?;
            println!("telar: arranque .tlar exportado: {ruta}");
        } else {
            let mut bytes = Vec::new();
            fs::File::open(ruta)
                .map_err(|e| e.to_string())?
                .take(4_194_453)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("leer {ruta}: {e}"))?;
            if bytes.get(4..8).is_some_and(|v| v == 9u32.to_le_bytes()) {
                let humo = crate::volumen_archivo::decodificar(&bytes)?;
                println!("telar: .tlar v9 · humo 3D 32³ · máximo {} iteraciones", humo.0[7]);
                return Ok(());
            }
            if bytes.get(4..8).is_some_and(|v| v == 7u32.to_le_bytes() || v == 8u32.to_le_bytes()) {
                let humo = crate::humo_archivo::decodificar(&bytes)?;
                println!(
                    "telar: .tlar v{} · humo MAC 2D · máximo {} iteraciones · tolerancias {}/{}",
                    bytes[4], humo.0[7], humo.0[8], humo.0[9]
                );
                return Ok(());
            }
            if bytes.get(4..8).is_some_and(|v| v == 6u32.to_le_bytes()) {
                let fuego = crate::fuego_archivo::decodificar(&bytes)?;
                println!("telar: .tlar v6 · fuego procedural · {} pasos", fuego.0[7]);
                return Ok(());
            }
            if bytes.get(4..8).is_some_and(|v| v == 5u32.to_le_bytes()) {
                let escena = crate::objetos::decodificar(&bytes)?;
                println!("telar: .tlar v5 · dos cajas, luz y salida editables · {} spp", escena.valores()[30]);
                return Ok(());
            }
            if bytes.get(4..8).is_some_and(|v| [2u32, 3, 4].iter().any(|n| v == n.to_le_bytes())) {
                let escena = crate::proyecto::Proyecto::decodificar(&bytes)?;
                println!(
                    "telar: .tlar v{} · escena procedural · lado {} · capacidad {} · MSAA solicitado {}x · fase {}",
                    escena.version(),
                    escena.estudio[0],
                    escena.estudio[1],
                    escena.estudio[8],
                    escena.fase
                );
                return Ok(());
            }
            let paquete = telar_kernel::paquete::decodificar(&bytes)?;
            println!(
                "telar: .tlar v1 · arranque, no escena completa · GPU declarada {:04x}:{:04x} · capacidad {} · SPIR-V {} palabras",
                paquete.cabecera[5],
                paquete.cabecera[6],
                paquete.cabecera[16],
                paquete.codigo.len()
            );
        }
        Ok(())
    })())
}
