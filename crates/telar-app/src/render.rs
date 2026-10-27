//! Adaptación de cámara y materiales al renderizador aislado de TELAR.
use nudo::Camara;

pub(crate) fn bloque(
    camara: &Camara,
    aspecto: f32,
    color: [f32; 4],
    material: usize,
    codificar_srgb: bool,
) -> Result<[f32; 32], String> {
    let color = telar_gpu::eleccion::color_para(telar_gpu::eleccion::FORMATO_SRGB, color);
    let indice = material.min(2);
    telar_render::Parametros {
        matriz: camara.matriz(aspecto),
        ojo: camara.posicion(),
        luz: nudo::mate::normalizar([0.45, 0.8, 0.4]),
        material: telar_render::Material {
            color: [color[0], color[1], color[2]],
            rugosidad: [0.3, 0.42, 0.15][indice],
            metal: [0.8, 0.0, 0.0][indice],
        },
        exposicion: 1.4,
        intensidad: 4.0,
        codificar_srgb,
    }
    .bloque()
    .map_err(str::to_owned)
}
