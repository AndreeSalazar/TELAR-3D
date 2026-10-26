//! `telar --calculo`: el primer kernel de TELAR en la GPU, contra la CPU.
//!
//! `hilo-spirv` escribe el kernel (llenar un buffer con `hash_pcg(i)`),
//! `telar-gpu` lo ejecuta y mide con los temporizadores de la GPU, y aquí se
//! compara **bit a bit** con la referencia en la CPU. Ninguno de los dos crates
//! conoce al otro: los une la app.

use std::time::Instant;

use hilo_spirv::ejemplos::{hash_pcg, llenar_con_hash};
use telar_gpu::{Calculo, Opciones};

/// Hilos por grupo de trabajo.
const GRUPO: u32 = 256;
/// Veces que se repite el kernel: cuenta el mejor tiempo (la primera calienta la GPU).
const REPETICIONES: usize = 5;

/// Ejecuta el kernel sobre `elementos` números y lo compara con la CPU.
/// Devuelve `Ok(true)` si la GPU y la CPU dan los mismos bits.
pub(crate) fn medir(validacion: bool, elementos: u32) -> Result<bool, String> {
    let gpu = Calculo::nuevo(&Opciones { validacion, ..Opciones::default() })?;
    let spirv = llenar_con_hash(GRUPO);
    println!(
        "telar: {} · kernel de {} palabras SPIR-V escrito a mano · {elementos} números",
        gpu.dispositivo(),
        spirv.len()
    );

    let mut mejor: Option<f64> = None;
    let mut datos = Vec::new();
    for _ in 0..REPETICIONES {
        let r = gpu.ejecutar(&spirv, elementos, GRUPO)?;
        mejor = match (mejor, r.nanosegundos) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        datos = r.datos;
    }

    let reloj = Instant::now();
    let referencia: Vec<u32> = (0..elementos).map(hash_pcg).collect();
    let cpu_ms = reloj.elapsed().as_secs_f64() * 1000.0;

    let distinto = datos.iter().zip(&referencia).position(|(g, c)| g != c);
    let iguales = datos.len() == referencia.len() && distinto.is_none();
    match mejor {
        Some(ns) => println!("telar: GPU (solo el kernel): {:.3} ms · CPU (un hilo): {cpu_ms:.3} ms", ns / 1e6),
        None => println!("telar: esta cola no tiene temporizadores · CPU (un hilo): {cpu_ms:.3} ms"),
    }
    if iguales {
        println!("telar: GPU y CPU iguales bit a bit ({elementos} de {elementos})");
    } else if let Some(i) = distinto {
        println!("telar: DISTINTOS en el índice {i}: GPU {:#010x} · CPU {:#010x}", datos[i], referencia[i]);
    } else {
        println!("telar: DISTINTOS: la GPU devolvió {} números", datos.len());
    }
    Ok(iguales)
}
