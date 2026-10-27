//! Visibilidad real sobre la geometría residente, sin replicar la onda en el shader.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use telar_gpu::{Calculo, CapaInterfaz, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores};

#[test]
fn sombras_quitan_energia_y_reaccionan_a_geometria_con_marcha_acotada() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let mut residente = gpu
        .procedural(65, KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] })
        .unwrap();
    let vertices = telar_render::vertices();
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let camara = nudo::Camara::default();
    let bloque = telar_render::Parametros {
        matriz: camara.matriz(1.0),
        ojo: camara.posicion(),
        luz: [0.0, 1.0, 0.0],
        material: telar_render::Material { color: [0.6; 3], rugosidad: 0.5, metal: 0.0 },
        exposicion: 1.4,
        intensidad: 16.0,
        codificar_srgb: true,
    }
    .bloque()
    .unwrap();
    let luz = telar_render::LuzArea { centro: [2.4, 0.8, 0.0], semilado: 0.4, ..Default::default() };
    let sin = telar_render::fragmentos_rejilla(telar_render::LuzArea { sombras: false, ..luz }).unwrap();
    let con = telar_render::fragmentos_rejilla(luz).unwrap();
    let capturar = |residente: &telar_gpu::Procedural, fragmentos: &[u32]| {
        residente
            .medir_estudio(
                Sombreadores { vertices: &vertices, fragmentos },
                (256, 256),
                [0.0, 0.0, 0.0, 1.0],
                &bloque,
                CapaInterfaz {
                    shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                    vertices: &[],
                    area: [0, 0, 256, 256],
                    atlas: None,
                },
            )
            .unwrap()
    };
    residente.paso(ParametrosProcedurales { lado: 65, fase: 0.6, amplitud: 0.0 }).unwrap();
    let plano_a = capturar(&residente, &sin);
    let plano_b = capturar(&residente, &con);
    assert_eq!(plano_a.pixeles, plano_b.pixeles, "un plano sin obstáculos no se tapa a sí mismo");
    residente.paso(ParametrosProcedurales { lado: 65, fase: 0.6, amplitud: 0.65 }).unwrap();
    let a = capturar(&residente, &sin);
    let b = capturar(&residente, &con);
    let mut oscurecidos = 0;
    for (a, b) in a.pixeles.as_chunks::<4>().0.iter().zip(b.pixeles.as_chunks::<4>().0) {
        for i in 0..3 {
            assert!(b[i] <= a[i], "la visibilidad no agrega energía");
        }
        oscurecidos += usize::from(b[0] < a[0]);
    }
    assert!(oscurecidos > 100, "debe haber autosombra visible: {oscurecidos}");
    assert_eq!(b.pixeles, capturar(&residente, &con).pixeles, "muestreo determinista");
    assert!(b.nanosegundos.is_none_or(|n| n.is_finite() && n > 0.0));
    assert_eq!(residente.actividad().geometria_subida, 0);
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
