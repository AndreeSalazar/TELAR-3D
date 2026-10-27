//! El render original responde a materiales y exposición en Vulkan real.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use telar_gpu::{Calculo, CapaInterfaz, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores};

#[test]
fn material_y_exposicion_cambian_pixeles_sin_subir_geometria() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let mut residente = gpu
        .procedural(33, KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] })
        .unwrap();
    residente.paso(ParametrosProcedurales { lado: 33, fase: 0.6, amplitud: 0.35 }).unwrap();
    let render = [telar_render::vertices(), telar_render::fragmentos()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let camara = nudo::Camara::default();
    let mut parametros = telar_render::Parametros {
        matriz: camara.matriz(1.0),
        ojo: camara.posicion(),
        luz: nudo::mate::normalizar([0.45, 0.8, 0.4]),
        material: telar_render::Material { color: [0.5, 0.2, 0.1], rugosidad: 0.12, metal: 1.0 },
        exposicion: 0.5,
        intensidad: 4.0,
        codificar_srgb: true,
    };
    let capturar = |bloque: &[f32; 32]| {
        residente.capturar_estudio_bloque(
            Sombreadores { vertices: &render[0], fragmentos: &render[1] },
            (256, 256),
            [0.0, 0.0, 0.0, 1.0],
            bloque,
            CapaInterfaz {
                shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                vertices: &[],
                area: [0, 0, 256, 256],
                atlas: None,
            },
        )
    };
    let a = capturar(&parametros.bloque().unwrap()).unwrap();
    parametros.exposicion = 2.0;
    let b = capturar(&parametros.bloque().unwrap()).unwrap();
    let energia = |bytes: &[u8]| {
        bytes.as_chunks::<4>().0.iter().map(|p| u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2])).sum::<u64>()
    };
    assert!(energia(&b) > energia(&a));
    parametros.material.rugosidad = 0.65;
    parametros.material.metal = 0.0;
    let c = capturar(&parametros.bloque().unwrap()).unwrap();
    assert!(b.as_chunks::<4>().0.iter().zip(c.as_chunks::<4>().0).filter(|(a, b)| a != b).count() > 1000);
    assert!(c.as_chunks::<4>().0.iter().filter(|p| p[0] > 0).count() > 1000);
    let mut invalido = parametros.bloque().unwrap();
    invalido[0] = f32::NAN;
    assert!(capturar(&invalido).is_err());
    assert_eq!(residente.actividad().geometria_subida, 0);
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
