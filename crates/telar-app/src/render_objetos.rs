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
pub(crate) fn preparar(gpu: &mut Gpu, escena: &EscenaObjetos, aspecto: f32) -> Result<(), String> {
    let trabajo = trabajo(escena, Some(aspecto))?;
    let vertices = telar_render::visor_rayos::vertices();
    let fragmentos =
        telar_render::visor_rayos::fragmentos(&trabajo, gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB)?;
    gpu.preparar_visor(Sombreadores { vertices: &vertices, fragmentos: &fragmentos })
}
pub(crate) fn exportar(escena: &EscenaObjetos, ruta: &Path, validacion: bool) -> Result<(), String> {
    if ruta.extension().and_then(|v| v.to_str()) != Some("pfm") {
        return Err("salida HDR requiere .pfm".into());
    }
    let trabajo = trabajo(escena, None)?;
    let codigo = trabajo.compilar()?;
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones { validacion, ..telar_gpu::Opciones::default() })?;
    let datos = gpu.ejecutar(&codigo, trabajo.tamano[0] * trabajo.tamano[1] * 5, 64)?;
    let (pfm, ppm) = crate::salida_hdr::codificar(&datos.datos, trabajo.tamano, trabajo.muestras)?;
    let receta = format!(
        "TELAR objetos · escena v5\nGPU: {}\nEstado: {:?}\nGPU ns: {:?}\n",
        gpu.dispositivo(),
        escena.guardar(),
        datos.nanosegundos
    );
    drop(gpu);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("render HDR con avisos de validación".into());
    }
    crate::salida_hdr::guardar(ruta, &pfm, &ppm, &receta)
}
