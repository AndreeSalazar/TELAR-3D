//! El productor GPU gobierna recuentos, despacho y dibujo. La CPU solo verifica
//! mediante descargas explícitas fuera de la ruta viva.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "tests con tamaños pequeños, asserts y referencias numéricas independientes"
)]

use telar_gpu::{
    Calculo, Constantes, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores, avisos_de_validacion,
};

fn parametros(lado: u32, fase: f32, amplitud: f32) -> ParametrosProcedurales {
    ParametrosProcedurales { lado, fase, amplitud }
}

#[test]
fn cadena_residente_correcta_reutilizable_y_visible() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real es necesario para verificar esta ruta");
    assert!(gpu.validacion(), "esta prueba exige las capas de validación");
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let kernels = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] };
    let mut residente = gpu.procedural(17, kernels).unwrap();
    assert!(residente.inspeccionar().is_err(), "la memoria sin producir no se descarga");
    assert!(gpu.procedural(1025, kernels).is_err());
    assert!(residente.paso(parametros(2, f32::NAN, 0.0)).is_err());
    let inicial = residente.actividad();
    assert_eq!((inicial.buffers, inicial.pipelines, inicial.grabaciones), (4, 3, 1));
    for paso in 0..120 {
        let lado = [0, 1, 2, 3, 8, 9, 17, 18, u32::MAX][paso % 9];
        let fase = paso as f32 * 0.031;
        let amplitud = if paso % 2 == 0 { 0.0 } else { 0.37 };
        residente.paso(parametros(lado, fase, amplitud)).unwrap();
        let estado = residente.inspeccionar().unwrap();
        let valido = (2..=17).contains(&lado);
        let puntos = if valido { lado * lado } else { 0 };
        let indices = if valido { (lado - 1).pow(2) * 6 } else { 0 };
        assert_eq!(&estado.control[..8], &[puntos.div_ceil(64), 1, 1, indices, 1, 0, 0, 0]);
        assert_eq!(estado.control[9], puntos);
        assert_eq!(estado.control[10], u32::from(lado > 17));
        assert_eq!(estado.vertices.len(), puntos as usize);
        let mut esperados = Vec::new();
        for fila in 0..lado.min(17).saturating_sub(1) {
            if !valido {
                break;
            }
            for columna in 0..lado - 1 {
                let indice = fila * lado + columna;
                esperados.extend([indice, indice + lado, indice + 1, indice + 1, indice + lado, indice + lado + 1]);
            }
        }
        assert_eq!(estado.indices, esperados);
        for (indice, vertice) in estado.vertices.iter().enumerate() {
            let indice = u32::try_from(indice).unwrap();
            let horizontal = 4.0 * (indice % lado) as f32 / (lado - 1) as f32 - 2.0;
            let profundidad = 4.0 * (indice / lado) as f32 / (lado - 1) as f32 - 2.0;
            let eje_x = horizontal * 2.0 + fase;
            let eje_z = profundidad * 2.0;
            let altura = amplitud * eje_x.sin() * eje_z.cos();
            let normal = [-2.0 * amplitud * eje_x.cos() * eje_z.cos(), 1.0, 2.0 * amplitud * eje_x.sin() * eje_z.sin()];
            let longitud = normal.iter().map(|v| v * v).sum::<f32>().sqrt();
            let esperado =
                [horizontal, altura, profundidad, normal[0] / longitud, normal[1] / longitud, normal[2] / longitud];
            for (actual, referencia) in vertice.iter().zip(esperado) {
                assert!((actual - referencia).abs() < 0.00002, "{actual} != {referencia}");
            }
        }
    }
    let actividad = residente.actividad();
    assert_eq!((actividad.buffers, actividad.pipelines, actividad.grabaciones), (4, 3, 1));
    assert_eq!((actividad.pasos, actividad.parametros, actividad.geometria_subida), (120, 3840, 0));
    assert!(actividad.descargados > 0, "las lecturas de diagnóstico se contabilizan");
    let fijo = parametros(17, 0.25, 0.3);
    residente.paso(fijo).unwrap();
    let primera = residente.inspeccionar().unwrap();
    residente.paso(fijo).unwrap();
    let segunda = residente.inspeccionar().unwrap();
    assert_eq!(&primera.control[..12], &segunda.control[..12]);
    assert_eq!(primera.control[15], segunda.control[15], "repetir parámetros conserva la generación");
    assert_eq!(segunda.control[12], 0, "la GPU omite generar cuando puede reutilizar");
    assert_eq!(primera.indices, segunda.indices);
    let bits = |v: &Vec<[f32; 6]>| v.iter().map(|p| p.map(f32::to_bits)).collect::<Vec<_>>();
    assert_eq!(bits(&primera.vertices), bits(&segunda.vertices));
    assert!(segunda.nanosegundos.as_ref().is_some_and(|t| t.iter().all(|n| n.is_finite() && *n > 0.0)));
    eprintln!("{} · ns preparar/generar/deformar {:?}", gpu.dispositivo(), segunda.nanosegundos);

    let grafico = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    let shaders = Sombreadores { vertices: &grafico[0], fragmentos: &grafico[1] };
    let constantes = Constantes {
        matriz: nudo::Camara::default().matriz(1.0),
        luz: [0.0, 1.0, 0.0],
        color: [1.0; 4],
        cielo: [0.5; 4],
        suelo: [0.1; 4],
    };
    let fondo = [0.0, 0.0, 0.0, 1.0];
    let imagen = residente.capturar(shaders, (96, 96), fondo, &constantes).unwrap();
    assert!(imagen.as_chunks::<4>().0.iter().filter(|p| p[0] > 20).count() > 100);
    residente.paso(parametros(17, 1.8, 0.8)).unwrap();
    let otra = residente.capturar(shaders, (96, 96), fondo, &constantes).unwrap();
    assert_ne!(imagen, otra, "cambiar parámetros debe cambiar los píxeles");
    residente.paso(parametros(0, 0.0, 0.0)).unwrap();
    let vacia = residente.capturar(shaders, (96, 96), fondo, &constantes).unwrap();
    assert!(vacia.as_chunks::<4>().0.iter().all(|p| p == &[0, 0, 0, 255]));
    drop(residente);
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}

#[test]
fn capacidad_maxima_residente_en_gpu() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real es necesario");
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let kernels = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] };
    let mut residente = gpu.procedural(1024, kernels).unwrap();
    residente.paso(parametros(1024, 0.7, 0.35)).unwrap();
    let estado = residente.inspeccionar().unwrap();
    assert_eq!(estado.control[0], 16_384);
    assert_eq!(estado.control[10], 0);
    assert_eq!(estado.vertices.len(), 1_048_576);
    assert_eq!(estado.indices.len(), 6_279_174);
    assert!(estado.indices.iter().all(|i| *i < 1_048_576));
    assert!(estado.vertices.iter().flatten().all(|v| v.is_finite()));
    assert_eq!(estado.control[15], 1);
    eprintln!(
        "{} · 1024² puntos · 2093058 triángulos · ns preparar/generar/deformar {:?}",
        gpu.dispositivo(),
        estado.nanosegundos
    );
    drop(estado);
    residente.paso(parametros(1024, 1.1, 0.35)).unwrap();
    let reutilizado = residente.inspeccionar().unwrap();
    assert_eq!(reutilizado.control[12], 0, "no genera otra vez el millón de puntos");
    assert_eq!(reutilizado.control[15], 1);
    assert_eq!(reutilizado.vertices.len(), 1_048_576);
    assert_eq!(reutilizado.indices.len(), 6_279_174);
    eprintln!("misma topología reutilizada · ns preparar/generar omitido/deformar {:?}", reutilizado.nanosegundos);
    drop(residente);
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}
