//! Control global real: tolerancia, presupuesto, fallo numérico y descarga de 64 B.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "rejilla pequeña de ensayo")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const LADO: usize = 17;
const CELDAS: usize = LADO * LADO;
const CONTROL: usize = 14 * CELDAS;
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-adaptativo-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn preparar() -> (telar_ventana::Ventana, Gpu) {
    let plan = telar_kernel::humo::Configuracion { lado: 17, iteraciones: 8 }.compilar_adaptativo().unwrap();
    for (i, codigo) in plan.codigos.iter().enumerate() {
        validar(codigo, i);
    }
    validar(&telar_render::humo::fragmentos_adaptativo(17).unwrap(), 14);
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 96,
        alto: 96,
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
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::humo::fragmentos_adaptativo(17).unwrap(),
        },
        PlanCampos { elementos: 289, palabras: 15, programas: &plan.codigos, secuencia: &plan.orden },
    )
    .unwrap();
    gpu.subir_malla(&MALLA, &[0, 2, 1]).unwrap();
    gpu.preparar_interfaz(Sombreadores {
        vertices: &hilo_spirv::interfaz::vertices(),
        fragmentos: &hilo_spirv::interfaz::fragmentos(),
    })
    .unwrap();
    gpu.actualizar_interfaz(&[], [0, 0, 96, 96]).unwrap();
    (ventana, gpu)
}
fn paso(gpu: &mut Gpu, inicial: &[f32], bloque: &[f32; 32]) -> Vec<f32> {
    gpu.activar_campos(true).unwrap();
    gpu.cargar_campos(&inicial.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
    for _ in 0..10 {
        if gpu.dibujar_bloque_interfaz([0.0, 0.0, 0.0, 1.0], bloque).unwrap() {
            return gpu.inspeccionar_campos().unwrap().into_iter().map(f32::from_bits).collect();
        }
    }
    panic!("no se pudo dibujar el ensayo adaptativo");
}
fn inicial() -> Vec<f32> {
    let mut estado = vec![0.0; 15 * CELDAS];
    for y in 0..LADO {
        for x in 0..LADO {
            let pixel = y * LADO + x;
            estado[pixel] = if x + 1 < LADO { ((x + y) as f32 * 0.43).sin() * 0.003 } else { 0.0 };
            estado[CELDAS + pixel] = if y + 1 < LADO { ((x * 2 + y) as f32 * 0.31).cos() * 0.003 } else { 0.0 };
            estado[2 * CELDAS + pixel] = 0.4;
            estado[3 * CELDAS + pixel] = 0.2;
        }
    }
    estado
}
fn referencia(estado: &[f32], p: &[f32; 8], iter: u32) -> Vec<f32> {
    let mut resultado = estado[..12 * CELDAS].to_vec();
    telar_kernel::humo::referencia::paso(LADO, iter, &mut resultado, p).unwrap();
    resultado
}
fn norma(estado: &[f32], plano: usize) -> f32 {
    estado[plano * CELDAS..(plano + 1) * CELDAS].iter().map(|v| v.abs()).fold(0.0, f32::max)
}
fn comparar(actual: &[f32], esperado: &[f32]) {
    for (i, (a, b)) in actual.iter().zip(esperado).enumerate() {
        assert!((a - b).abs() < 0.0003, "campo {i}: GPU {a}, referencia {b}");
    }
}
#[test]
fn gpu_decide_convergencia_limite_e_invalidos() {
    let (ventana, mut gpu) = preparar();
    let parametros = [0.08, 0.0, 0.0, 0.0, 0.0, 0.001, 1.0, 8.0];
    let mut bloque = [0.0; 32];
    bloque[..8].copy_from_slice(&parametros);
    bloque[10] = 8.0;
    bloque[11] = 0.02;
    bloque[12] = 0.005;
    let vacio = vec![0.0; 15 * CELDAS];
    let resultado = paso(&mut gpu, &vacio, &bloque);
    assert_eq!(resultado[CONTROL + 1].to_bits(), 0.0f32.to_bits());
    assert_eq!(resultado[CONTROL + 5].to_bits(), 1.0f32.to_bits());
    let estado = inicial();
    let dos = referencia(&estado, &parametros, 2);
    let cuatro = referencia(&estado, &parametros, 4);
    let tolerancia = f32::midpoint(norma(&dos, 11), norma(&cuatro, 11));
    assert!(norma(&dos, 11) > norma(&cuatro, 11));
    bloque[11] = 0.0;
    bloque[12] = tolerancia;
    let resultado = paso(&mut gpu, &estado, &bloque);
    assert_eq!(resultado[CONTROL + 1].to_bits(), 4.0f32.to_bits());
    assert_eq!(resultado[CONTROL + 5].to_bits(), 1.0f32.to_bits());
    assert!(resultado[CONTROL + 3] <= resultado[CONTROL + 4]);
    franja(&mut gpu, &bloque, 1);
    comparar(&resultado, &cuatro);
    assert!((resultado[CONTROL + 2] - norma(&resultado, 8)).abs() < 0.0001);
    assert!((resultado[CONTROL + 3] - norma(&resultado, 11)).abs() < 0.0003);
    assert_eq!(resultado, paso(&mut gpu, &estado, &bloque));
    let resumen = gpu.inspeccionar_rango_campos(u32::try_from(CONTROL).unwrap(), 16).unwrap();
    assert_eq!(resumen, resultado[CONTROL..CONTROL + 16].iter().map(|v| v.to_bits()).collect::<Vec<_>>());
    assert!(gpu.inspeccionar_rango_campos(0, 0).is_err());
    assert!(gpu.inspeccionar_rango_campos(u32::MAX, 1).is_err());
    bloque[12] = 0.0;
    bloque[10] = 2.0;
    let limitado = paso(&mut gpu, &estado, &bloque);
    assert_eq!(limitado[CONTROL + 1].to_bits(), 2.0f32.to_bits());
    assert_eq!(limitado[CONTROL + 5].to_bits(), 2.0f32.to_bits());
    comparar(&limitado, &dos);
    assert!(limitado[CONTROL + 3] > 0.0);
    franja(&mut gpu, &bloque, 2);
    bloque[10] = 8.0;
    bloque[11] = 0.5;
    let relativa = paso(&mut gpu, &estado, &bloque);
    assert!((relativa[CONTROL + 4] - 0.5 * relativa[CONTROL + 2]).abs() < 0.0001);
    assert_eq!(relativa[CONTROL + 5].to_bits(), 1.0f32.to_bits());
    let mut roto = estado.clone();
    roto[10 * CELDAS + 5] = f32::NAN;
    let invalido = paso(&mut gpu, &roto, &bloque);
    assert_eq!(invalido[CONTROL + 5].to_bits(), 3.0f32.to_bits());
    assert_eq!(invalido[..4 * CELDAS], estado[..4 * CELDAS]);
    franja(&mut gpu, &bloque, 3);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
fn franja(gpu: &mut Gpu, bloque: &[f32; 32], estado: u32) {
    gpu.activar_campos(false).unwrap();
    let imagen = gpu
        .capturar_campos_interfaz(
            (96, 96),
            [0.0, 0.0, 0.0, 1.0],
            bloque,
            telar_gpu::CapaInterfaz {
                shaders: Sombreadores {
                    vertices: &hilo_spirv::interfaz::vertices(),
                    fragmentos: &hilo_spirv::interfaz::fragmentos(),
                },
                vertices: &[],
                area: [0, 0, 96, 96],
                atlas: None,
            },
        )
        .unwrap();
    let p = &imagen.pixeles[48 * 4..48 * 4 + 3];
    match estado {
        1 => assert!(p[1] > p[0] && p[1] > p[2]),
        2 => assert!(p[0] > p[1] && p[1] > p[2]),
        _ => assert!(p[0] > p[1] && p[0] > p[2]),
    }
}
