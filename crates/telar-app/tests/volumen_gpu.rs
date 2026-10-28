//! Humo MAC 3D real: referencia escalar, presión, vistas y pausa sin descarga por cuadro.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "rejilla pequeña de ensayo")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const LADO: usize = 9;
const CELDAS: usize = LADO * LADO * LADO;
const CONTROL: usize = 16 * CELDAS;
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-volumen-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn preparar() -> (telar_ventana::Ventana, Gpu) {
    let plan = telar_kernel::humo3d::Configuracion { lado: 9, iteraciones: 8 }.compilar().unwrap();
    for (i, codigo) in plan.codigos.iter().enumerate() {
        validar(codigo, i);
    }
    validar(&telar_render::volumen::fragmentos(9, 32).unwrap(), 14);
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
            fragmentos: &telar_render::volumen::fragmentos(9, 32).unwrap(),
        },
        PlanCampos { elementos: 729, palabras: 17, programas: &plan.codigos, secuencia: &plan.orden },
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
    let mut estado = vec![0.0; 17 * CELDAS];
    for pixel in 0..CELDAS {
        let coords = [pixel % LADO, (pixel / LADO) % LADO, pixel / (LADO * LADO)];
        for eje in 0..3 {
            estado[eje * CELDAS + pixel] =
                if coords[eje] + 1 < LADO { ((pixel + eje * 17) as f32 * 0.43).sin() * 0.003 } else { 0.0 };
        }
        estado[3 * CELDAS + pixel] = if coords[0] < 4 && coords[2] < 6 { 0.7 } else { 0.1 };
        estado[4 * CELDAS + pixel] = 0.2;
    }
    estado
}
fn bloque() -> [f32; 32] {
    let mut p = [0.0; 32];
    p[..8].copy_from_slice(&[0.08, 0.0, 0.0, 0.0, 0.0, 0.001, 1.0, 0.0]);
    p[10] = 8.0;
    p[16..19].copy_from_slice(&[0.0, 0.0, -1.0]);
    p[20..23].copy_from_slice(&[0.5, 0.0, 0.0]);
    p[24..27].copy_from_slice(&[0.0, 0.5, 0.0]);
    p[28..31].copy_from_slice(&[0.5, 0.5, 2.0]);
    p
}
fn imagen(gpu: &mut Gpu, p: &[f32; 32]) -> Vec<u8> {
    gpu.capturar_campos_interfaz(
        (96, 96),
        [0.0, 0.0, 0.0, 1.0],
        p,
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
    .unwrap()
    .pixeles
}
#[test]
fn volumen_coincide_con_referencia_y_orbita_sin_alterar_campos() {
    let (ventana, mut gpu) = preparar();
    let mut p = bloque();
    let vacio = vec![0.0; 17 * CELDAS];
    let cero = paso(&mut gpu, &vacio, &p);
    assert_eq!(cero[CONTROL + 1].to_bits(), 0.0f32.to_bits());
    assert_eq!(cero[CONTROL + 5].to_bits(), 1.0f32.to_bits());
    let estado = inicial();
    let resultado = paso(&mut gpu, &estado, &p);
    let mut esperado = estado.clone();
    telar_kernel::humo3d::referencia::paso(LADO, 8, &mut esperado, &p[..8].try_into().unwrap()).unwrap();
    for (i, (a, b)) in resultado[..14 * CELDAS].iter().zip(&esperado).enumerate() {
        assert!(a.is_finite() && (a - b).abs() < 0.0003, "campo {i}: GPU {a}, referencia {b}");
    }
    assert_eq!(resultado[CONTROL + 1].to_bits(), 8.0f32.to_bits());
    assert_eq!(resultado[CONTROL + 5].to_bits(), 2.0f32.to_bits());
    let norma = |plano: usize| resultado[plano * CELDAS..(plano + 1) * CELDAS].iter().map(|v| v * v).sum::<f32>();
    assert!(norma(13) < norma(10));
    assert_eq!(resultado, paso(&mut gpu, &estado, &p));
    control(&mut gpu, &estado, &p);
    paso(&mut gpu, &estado, &p);
    gpu.activar_campos(false).unwrap();
    let antes = gpu.inspeccionar_campos().unwrap();
    let frontal = imagen(&mut gpu, &p);
    p[16..19].copy_from_slice(&[-1.0, 0.0, 0.0]);
    p[20..23].copy_from_slice(&[0.0, 0.0, -0.5]);
    p[28..31].copy_from_slice(&[2.0, 0.5, 0.5]);
    assert_ne!(frontal, imagen(&mut gpu, &p));
    for vista in [1.0, 2.0, 3.0] {
        p[7] = vista;
        let vista_imagen = imagen(&mut gpu, &p);
        assert_ne!(frontal, vista_imagen);
    }
    assert_eq!(antes, gpu.inspeccionar_campos().unwrap());
    p[7] = 0.0;
    p[1] = 3.0;
    p[2] = 2.0;
    p[3] = 0.08;
    p[4] = 0.15;
    p[5] = 0.016;
    let fuente = paso(&mut gpu, &vacio, &p);
    let mut referencia = vacio;
    telar_kernel::humo3d::referencia::paso(LADO, 0, &mut referencia, &p[..8].try_into().unwrap()).unwrap();
    assert!(fuente[3 * CELDAS..4 * CELDAS].iter().any(|v| *v > 0.0));
    for (a, b) in fuente[..14 * CELDAS].iter().zip(&referencia) {
        assert!((a - b).abs() < 0.0003);
    }
    gpu.limpiar_campos().unwrap();
    assert!(gpu.inspeccionar_campos().unwrap().iter().all(|v| *v == 0));
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
fn control(gpu: &mut Gpu, estado: &[f32], p: &[f32; 32]) {
    let mut dos = estado.to_vec();
    let mut cuatro = estado.to_vec();
    let parametros = p[..8].try_into().unwrap();
    telar_kernel::humo3d::referencia::paso(LADO, 2, &mut dos, parametros).unwrap();
    telar_kernel::humo3d::referencia::paso(LADO, 4, &mut cuatro, parametros).unwrap();
    let maximo = |v: &[f32]| v[13 * CELDAS..14 * CELDAS].iter().map(|v| v.abs()).fold(0.0, f32::max);
    assert!(maximo(&dos) > maximo(&cuatro));
    let mut p = *p;
    p[12] = f32::midpoint(maximo(&dos), maximo(&cuatro));
    let convergido = paso(gpu, estado, &p);
    assert_eq!(convergido[CONTROL + 1].to_bits(), 4.0f32.to_bits());
    assert_eq!(convergido[CONTROL + 5].to_bits(), 1.0f32.to_bits());
    assert!((convergido[CONTROL + 3] - maximo(&convergido)).abs() < 0.0003);
    for (a, b) in convergido[..14 * CELDAS].iter().zip(&cuatro) {
        assert!((a - b).abs() < 0.0003);
    }
    let mut roto = estado.to_vec();
    roto[12 * CELDAS + 5] = f32::NAN;
    let invalido = paso(gpu, &roto, &p);
    assert_eq!(invalido[CONTROL + 5].to_bits(), 3.0f32.to_bits());
    assert_eq!(invalido[..5 * CELDAS], estado[..5 * CELDAS]);
}
