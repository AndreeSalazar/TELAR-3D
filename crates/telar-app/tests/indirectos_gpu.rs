//! Argumentos GPU reales: grupos limitados, cierre por convergencia y equivalencia del estado.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de integración Vulkan")]
use telar_gpu::{Gpu, Opciones, PlanCampos, RutaCampos, Sombreadores, Superficie};
fn preparar() -> (telar_ventana::Ventana, Gpu, [f32; 32]) {
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 64,
        alto: 64,
        visible: false,
        ..Default::default()
    })
    .unwrap();
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, ..Default::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )
    .unwrap();
    let plan = telar_kernel::humo3d::Configuracion { lado: 8, iteraciones: 8 }.compilar_temporal(1).unwrap();
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::volumen::fragmentos(8, 16).unwrap(),
        },
        PlanCampos { elementos: 512, palabras: 17, programas: &plan.codigos, secuencia: &plan.orden },
    )
    .unwrap();
    let mut bloque = [0.0; 32];
    bloque[..6].copy_from_slice(&[0.15, 3.0, 2.0, 0.08, 0.15, 0.016]);
    bloque[10..14].copy_from_slice(&[8.0, 0.02, 0.005, 1.0]);
    bloque[19] = 0.5;
    bloque[27] = 1.0;
    bloque[31] = 1.0;
    (ventana, gpu, bloque)
}
fn rutas() -> Vec<(usize, RutaCampos)> {
    let mut rutas: Vec<_> = telar_kernel::humo_indirecto::PROGRAMAS
        .into_iter()
        .zip(telar_kernel::humo_indirecto::OFFSETS)
        .map(|(p, o)| (p, RutaCampos::Indirecto(o)))
        .collect();
    rutas.push((8, RutaCampos::Productor(1)));
    rutas.push((13, RutaCampos::Productor(1)));
    rutas
}
fn semilla() -> Vec<u32> {
    let mut estado = vec![0; 17 * 512];
    estado[3 + 3 * 8 + 3 * 64] = 0.25f32.to_bits();
    estado
}
#[test]
fn gpu_publica_grupos_acotados_y_cero_al_terminar() {
    let (ventana, mut gpu, mut bloque) = preparar();
    gpu.cargar_campos(&semilla()).unwrap();
    bloque[11] = 0.0;
    bloque[12] = 0.0;
    gpu.secuencia_campos(&[14, 15, 0, 1, 6, 7, 8]).unwrap();
    gpu.ejecutar_campos(&bloque).unwrap();
    assert_eq!(&gpu.inspeccionar_argumentos_campos().unwrap()[..6], &[8, 1, 1, 1, 1, 1]);
    gpu.configurar_despachos_campos(&rutas()).unwrap();
    assert_eq!(gpu.carga_campos().indirectos, 4);
    for invalida in [
        vec![(9, RutaCampos::Indirecto(53))],
        vec![(9, RutaCampos::Indirecto(64))],
        vec![(99, RutaCampos::Indirecto(0))],
        vec![(9, RutaCampos::Directo(0))],
        vec![(9, RutaCampos::Directo(9))],
        vec![(9, RutaCampos::Indirecto(0)), (9, RutaCampos::Indirecto(12))],
    ] {
        assert!(gpu.configurar_despachos_campos(&invalida).is_err());
        assert_eq!(gpu.carga_campos().indirectos, 4);
    }
    gpu.secuencia_campos(&[9, 10, 11, 12, 13]).unwrap();
    for _ in 0..4 {
        gpu.ejecutar_campos(&bloque).unwrap();
    }
    assert_eq!(&gpu.inspeccionar_argumentos_campos().unwrap()[..6], &[0, 1, 1, 0, 1, 1]);
    let antes = gpu.inspeccionar_campos().unwrap();
    gpu.ejecutar_campos(&bloque).unwrap();
    assert_eq!(gpu.inspeccionar_campos().unwrap(), antes);
    gpu.limpiar_campos().unwrap();
    assert_eq!(gpu.inspeccionar_argumentos_campos().unwrap(), vec![0; 16]);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
#[test]
fn pasos_completos_conservan_cada_palabra_frente_a_despacho_directo() {
    let (ventana, mut gpu, bloque) = preparar();
    let estado = semilla();
    gpu.cargar_campos(&estado).unwrap();
    for _ in 0..6 {
        gpu.ejecutar_campos(&bloque).unwrap();
    }
    let esperado = gpu.inspeccionar_campos().unwrap();
    gpu.limpiar_campos().unwrap();
    gpu.cargar_campos(&estado).unwrap();
    gpu.configurar_despachos_campos(&rutas()).unwrap();
    for _ in 0..6 {
        gpu.ejecutar_campos(&bloque).unwrap();
    }
    assert_eq!(gpu.inspeccionar_campos().unwrap(), esperado);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
