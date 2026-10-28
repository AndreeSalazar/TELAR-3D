//! Traducción única del estado del editor a vista previa y render HDR.
use std::path::Path;
use telar_gpu::{Gpu, Sombreadores};
use telar_render::escena_rayos::Trabajo;
use telar_ui::EscenaObjetos;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "enteros validados por la escena"
)]
pub(crate) fn trabajo(escena: &EscenaObjetos, aspecto: Option<f32>) -> Result<Trabajo, String> {
    escena.validar()?;
    let v = escena.valores();
    let vector = |i| [v[i], v[i + 1], v[i + 2]];
    let mut triangulos = Vec::new();
    crate::escena_final::cuadrado(
        &mut triangulos,
        [[-4.0, 0.0, -4.0], [4.0, 0.0, -4.0], [4.0, 0.0, 4.0], [-4.0, 0.0, 4.0]],
        [0.65; 3],
    );
    for base in [0, 10] {
        if v[base + 9] != 0.0 {
            crate::escena_final::caja(
                &mut triangulos,
                vector(base),
                vector(base + 3).map(|v| v * 0.5),
                vector(base + 6),
            );
        }
    }
    Ok(Trabajo {
        tamano: [v[31] as u32, v[32] as u32],
        muestras: v[30] as u32,
        semilla: v[33] as u32,
        camara: crate::escena_final::camara(vector(34), vector(37), aspecto.unwrap_or(v[31] / v[32]))?,
        triangulos,
        luz: vector(20),
        radio_luz: v[23],
        energia: v[24],
        color_luz: vector(25),
        exposicion: v[29],
        sombras: v[28] != 0.0,
    })
}
#[allow(clippy::cast_precision_loss, reason = "viewport <=1024×768")]
pub(crate) fn preparar_progresivo(gpu: &mut Gpu, escena: &EscenaObjetos, tamano: [u32; 2]) -> Result<(), String> {
    let trabajo = trabajo(escena, Some(tamano[0] as f32 / tamano[1].max(1) as f32))?;
    let vertices = telar_render::visor_rayos::vertices();
    let fragmentos = telar_render::historial_rayos::resolver();
    let compute = telar_render::historial_rayos::acumular(&trabajo, tamano)?;
    gpu.preparar_historial(Sombreadores { vertices: &vertices, fragmentos: &fragmentos }, &compute, tamano)
}
pub(crate) fn exportar(escena: &EscenaObjetos, ruta: &Path, validacion: bool) -> Result<(), String> {
    exportar_controlado(escena, ruta, validacion, |_| true).map(|_| ())
}

#[derive(Clone, Copy)]
pub(crate) enum Avance {
    Preparando,
    Pixeles(telar_gpu::ProgresoLotes),
    Exportando,
}

/// Falso significa cancelación antes de publicar archivos; no interrumpe escrituras iniciadas.
pub(crate) fn exportar_controlado(
    escena: &EscenaObjetos,
    ruta: &Path,
    validacion: bool,
    mut continuar: impl FnMut(Avance) -> bool,
) -> Result<bool, String> {
    if ruta.extension().and_then(|v| v.to_str()) != Some("pfm") {
        return Err("salida HDR requiere .pfm".into());
    }
    if !continuar(Avance::Preparando) {
        return Ok(false);
    }
    let trabajo = trabajo(escena, None)?;
    let codigo = trabajo.compilar()?;
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones { validacion, ..telar_gpu::Opciones::default() })?;
    let pixeles = trabajo.tamano[0] * trabajo.tamano[1];
    let plan = telar_gpu::PlanLotes {
        elementos: pixeles * 5,
        invocaciones: pixeles,
        grupo: 64,
        grupos_por_lote: (32768 / (64 * trabajo.muestras)).clamp(1, 64),
    };
    let Some(datos) = gpu.ejecutar_lotes(&codigo, plan, |p| continuar(Avance::Pixeles(p)))? else { return Ok(false) };
    let (pfm, ppm) = crate::salida_hdr::codificar(&datos, trabajo.tamano, trabajo.muestras)?;
    let receta = format!(
        "TELAR objetos · escena v5 · ejecución por lotes\nGPU: {}\nEstado: {:?}\nGrupos/lote: {}\nDescarga final: {} B\n",
        gpu.dispositivo(),
        escena.guardar(),
        plan.grupos_por_lote,
        datos.len() * 4
    );
    drop(gpu);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("render HDR con avisos de validación".into());
    }
    if !continuar(Avance::Exportando) {
        return Ok(false);
    }
    crate::salida_hdr::guardar(ruta, &pfm, &ppm, &receta)?;
    Ok(true)
}
