//! Preparación de la escena y contrato de recursos: separado del bucle de ventana.

use crate::visor::Modo;
use nudo::{Malla, Nodo};
use std::time::Instant;
use telar_gpu::{Gpu, KernelsProcedurales, Sombreadores};
use telar_nodos::Esfera;

pub(crate) fn preparar(gpu: &mut Gpu, modo: &Modo) -> Result<(u32, usize), String> {
    let (vertices, fragmentos) = (hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos());
    gpu.preparar_visor(Sombreadores { vertices: &vertices, fragmentos: &fragmentos })?;
    let capacidad = modo.puntos.isqrt().clamp(2, 1024);
    let puntos = if modo.procedural {
        let (preparar, generar, deformar) =
            (hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar());
        gpu.preparar_procedural(
            capacidad,
            KernelsProcedurales { preparar: &preparar, generar: &generar, deformar: &deformar },
        )?;
        println!("telar: rejilla residente · capacidad {capacidad}×{capacidad} · generación y deformación GPU");
        (capacidad * capacidad) as usize
    } else {
        let antes = Instant::now();
        let filas = i32::try_from((modo.puntos / 2).isqrt()).map_err(|_| "demasiados puntos")?.max(3);
        let esfera = Esfera { filas, columnas: 2 * filas, ..Esfera::default() };
        let malla = Malla::de(&esfera.cocinar(&[])?);
        println!(
            "telar: esfera de {} puntos · {} triángulos · cocinada en {:.0} ms",
            malla.vertices.len(),
            malla.triangulos(),
            antes.elapsed().as_secs_f64() * 1000.0
        );
        gpu.subir_malla(&malla.vertices, &malla.indices)?;
        malla.vertices.len()
    };
    Ok((capacidad, puntos))
}

pub(crate) fn comprobar(gpu: &Gpu, cuadros: u32) -> Result<(), String> {
    let actividad = gpu.actividad_procedural();
    println!(
        "telar: residente · {} buffers · {} pipelines · {} grabaciones compute · {} pasos · {} bytes parámetros · {} bytes geometría subida · {} bytes descargados",
        actividad.buffers,
        actividad.pipelines,
        actividad.grabaciones,
        actividad.pasos,
        actividad.parametros,
        actividad.geometria_subida,
        actividad.descargados
    );
    println!("telar: {} mapeos persistentes de parámetros", actividad.mapeos);
    if actividad.buffers != 8
        || actividad.pipelines != 6
        || actividad.grabaciones != 2
        || actividad.mapeos != 2
        || actividad.pasos != u64::from(cuadros)
        || actividad.parametros != u64::from(cuadros) * 16
        || actividad.geometria_subida != 0
        || actividad.descargados != 0
    {
        return Err("la ruta residente incumplió su presupuesto de recursos o transferencias".to_owned());
    }
    Ok(())
}
