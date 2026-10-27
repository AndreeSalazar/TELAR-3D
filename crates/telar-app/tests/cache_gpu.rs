//! La decisión de regenerar se toma en GPU, conservando geometría entre pasos.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "tests con tamaños pequeños y referencias independientes"
)]

use telar_gpu::{Calculo, KernelsProcedurales, Opciones, ParametrosProcedurales, avisos_de_validacion};

#[test]
fn el_productor_reutiliza_topologia_y_se_recupera_de_vacio_y_exceso() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("esta prueba requiere Vulkan real");
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let kernels = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] };
    let mut residente = gpu.procedural(9, kernels).unwrap();
    let (mut anterior, mut generaciones) = (0, 0);
    let mut indices_previos = Vec::new();
    // Arranque vacío, deformación sin regenerar, pausas, error, cambio real y retorno.
    for paso in 0..120 {
        let lado = [0, 1, 9, 9, 9, 0, 9, u32::MAX, 9, 5, 5, 9][paso % 12];
        let permitido = (2..=9).contains(&lado);
        let regenerar = permitido && lado != anterior;
        let fase = paso as f32 * 0.05;
        residente.paso(ParametrosProcedurales { lado, fase, amplitud: 0.4 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        if regenerar {
            generaciones += 1;
            anterior = lado;
        }
        assert_eq!(datos.control[11], anterior);
        assert_eq!(datos.control[15], generaciones);
        let grupos = if permitido { (lado * lado).div_ceil(64) } else { 0 };
        assert_eq!(&datos.control[12..15], &[if regenerar { grupos } else { 0 }, 1, 1]);
        assert_eq!(datos.control[0], grupos, "deformar tiene su propio argumento indirecto");
        assert_eq!(datos.control[10], u32::from(lado > 9));
        if permitido {
            if !regenerar {
                assert_eq!(datos.indices, indices_previos);
            }
            for vertice in &datos.vertices {
                // La deformación se evalúa desde x/z estables; no acumula alturas.
                let esperado = 0.4 * (2.0 * vertice[0] + fase).sin() * (2.0 * vertice[2]).cos();
                assert!((vertice[1] - esperado).abs() < 0.00002);
            }
            indices_previos = datos.indices;
        } else {
            assert!(datos.vertices.is_empty() && datos.indices.is_empty());
        }
    }
    let actividad = residente.actividad();
    assert_eq!((actividad.buffers, actividad.pipelines, actividad.grabaciones), (4, 3, 1));
    assert_eq!(actividad.mapeos, 1, "120 pasos reutilizan el único mapeo de parámetros");
    assert_eq!((actividad.pasos, actividad.parametros, actividad.geometria_subida), (120, 1920, 0));
    drop(residente);
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}
