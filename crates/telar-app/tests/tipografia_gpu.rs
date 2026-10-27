//! Muestreo bilineal, mezcla alfa y límites del atlas sobre Vulkan real.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]

use telar_gpu::{AtlasInterfaz, Calculo, CapaInterfaz, Constantes, KernelsProcedurales, Opciones, Sombreadores};

#[test]
fn cobertura_tipografica_se_interpola_y_respeta_transparencia_y_limites() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let mut residente = gpu
        .procedural(2, KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] })
        .unwrap();
    residente.paso(telar_gpu::ParametrosProcedurales { lado: 2, fase: 0.0, amplitud: 0.0 }).unwrap();
    let escena = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let constantes = Constantes {
        matriz: nudo::Camara::default().matriz(1.0),
        luz: [0.0, 1.0, 0.0],
        color: [1.0; 4],
        cielo: [0.0; 4],
        suelo: [0.0; 4],
    };
    // Tres bandas: 0,5 de cobertura bilineal; transparente; blanco fuera del atlas.
    let mut vertices = Vec::new();
    for (izquierda, derecha, uv) in [(-1.0, -0.4, [1.0, 1.0]), (-0.4, 0.2, [1.5, 0.5]), (0.2, 1.0, [-10.0, -10.0])] {
        let esquinas = [[izquierda, -1.0], [derecha, -1.0], [derecha, 1.0], [izquierda, 1.0]];
        for indice in [0, 1, 2, 0, 2, 3] {
            let punto = esquinas[indice];
            vertices.push([punto[0], punto[1], uv[0], uv[1], 1.0, 1.0, 1.0, 1.0]);
        }
    }
    let pixeles = residente
        .capturar_estudio(
            Sombreadores { vertices: &escena[0], fragmentos: &escena[1] },
            (96, 32),
            [0.0, 0.0, 0.0, 1.0],
            &constantes,
            CapaInterfaz {
                shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                vertices: &vertices,
                area: [0, 0, 1, 1],
                atlas: Some(AtlasInterfaz { ancho: 2, alto: 2, cobertura: &[255, 0, 0, 255] }),
            },
        )
        .unwrap();
    let rojo = |x: usize| pixeles[(16 * 96 + x) * 4];
    assert!((126..=129).contains(&rojo(10)), "cobertura interpolada al 50 %");
    assert_eq!(rojo(40), 0, "la cobertura cero conserva el fondo");
    assert_eq!(rojo(80), 255, "coordenadas negativas limitadas al píxel blanco");
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
