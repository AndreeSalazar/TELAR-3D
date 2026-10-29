//! Ciclo de vida real, referencia escalar y splats GPU de partículas.
#![allow(clippy::unwrap_used, reason = "ensayo GPU con fallos explícitos")]
#[allow(dead_code, reason = "se comprueba exportación; el resto de comandos pertenece a la app")]
#[path = "../src/intercambio_archivo.rs"]
mod intercambio_archivo;
#[allow(dead_code, reason = "exportador MAC se prueba en trazadores_gpu")]
#[path = "../src/particulas_geo.rs"]
mod particulas_geo;
#[allow(dead_code, reason = "adaptador de campos enlazado por la E/S compartida")]
#[path = "../src/puntos_mac.rs"]
mod puntos_mac;
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const N: usize = 128;
const CONTROL: usize = 11 * N;
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-particulas-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn preparar() -> (telar_ventana::Ventana, Gpu) {
    let codigos = telar_kernel::particulas::compilar();
    for (i, codigo) in codigos.iter().enumerate() {
        validar(codigo, i);
    }
    validar(&telar_render::particulas::fragmentos(), 2);
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
            fragmentos: &telar_render::particulas::fragmentos(),
        },
        PlanCampos { elementos: 128, palabras: 12, programas: &codigos, secuencia: &[0, 1] },
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

fn bloque() -> [f32; 32] {
    let mut p = [0.0; 32];
    p[..10].copy_from_slice(&[128.0, 1.5, 0.8, 1.5, 0.2, 0.016, 0.012, 7.0, 0.35, 0.6]);
    p[10] = 1.0;
    p[11] = 1.0;
    p[16..19].copy_from_slice(&[0.0, 0.0, -1.0]);
    p[20..23].copy_from_slice(&[0.5, 0.0, 0.0]);
    p[24..27].copy_from_slice(&[0.0, 0.5, 0.0]);
    p[28..31].copy_from_slice(&[0.5, 0.5, 2.0]);
    p
}
fn cuadro(gpu: &mut Gpu, p: &[f32; 32]) {
    for _ in 0..10 {
        if gpu.dibujar_bloque_interfaz([0.0, 0.0, 0.0, 1.0], p).unwrap() {
            return;
        }
    }
    panic!("sin cuadro de partículas");
}
#[test]
fn ciclo_gpu_coincide_con_referencia_y_termina_el_pulso() {
    let (ventana, mut gpu) = preparar();
    let p = bloque();
    let parametros = p[..10].try_into().unwrap();
    let mut referencia = vec![0.0; 12 * N];
    gpu.cargar_campos(&vec![0; 12 * N]).unwrap();
    for paso in 0..160 {
        cuadro(&mut gpu, &p);
        telar_kernel::particulas::referencia::paso(&mut referencia, &parametros).unwrap();
        let actual: Vec<_> = gpu.inspeccionar_campos().unwrap().into_iter().map(f32::from_bits).collect();
        for (i, (a, b)) in actual.iter().zip(&referencia).enumerate() {
            assert!(a.is_finite() && (a - b).abs() < 0.0002, "paso {paso} campo {i}: GPU {a}, CPU {b}");
        }
    }
    let control = gpu.inspeccionar_rango_campos(u32::try_from(CONTROL).unwrap(), 8).unwrap();
    assert_eq!(control[1], 0.0f32.to_bits());
    assert_eq!(control[2], 128.0f32.to_bits());
    reciclar(&mut gpu);
    suelo_y_cuarentena(&mut gpu);
    limites(&mut gpu);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

fn comparar(gpu: &Gpu, referencia: &[f32]) {
    let actual: Vec<_> = gpu.inspeccionar_campos().unwrap().into_iter().map(f32::from_bits).collect();
    for (i, (a, b)) in actual.iter().zip(referencia).enumerate() {
        assert!(a.is_finite() && (a - b).abs() < 0.0002, "reciclaje {i}: GPU {a}, CPU {b}");
    }
}
fn reciclar(gpu: &mut Gpu) {
    let mut p = bloque();
    p[0] = 37.0;
    p[1] = 0.1;
    p[2] = 0.3;
    p[3] = 2.0;
    p[5] = 0.04;
    p[8] = 0.0;
    let mut referencia = vec![0.0; 12 * N];
    gpu.cargar_campos(&vec![0; 12 * N]).unwrap();
    for paso in 0..120 {
        if paso == 50 {
            p[7] = 123.0;
        }
        if paso == 70 {
            p[0] = 8.0;
        }
        cuadro(gpu, &p);
        telar_kernel::particulas::referencia::paso(&mut referencia, &p[..10].try_into().unwrap()).unwrap();
        comparar(gpu, &referencia);
        if paso == 10 {
            exportar_y_vistas(gpu, &mut p);
        }
    }
    assert!(referencia[8 * N..9 * N].iter().any(|v| *v > 1.0));
    assert!(referencia[10 * N + 8..11 * N].iter().all(|v| *v == 0.0));
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
fn exportar_y_vistas(gpu: &mut Gpu, p: &mut [f32; 32]) {
    let antes = gpu.inspeccionar_campos().unwrap();
    let ruta = std::env::temp_dir().join(format!("telar-particulas-{}.geo", std::process::id()));
    let cantidad = particulas_geo::exportar(gpu, &ruta).unwrap();
    let intercambio_archivo::ArchivoGeo::Particulas(puntos) = intercambio_archivo::leer(&ruta).unwrap() else {
        panic!("perfil de vida perdido");
    };
    assert_eq!(cantidad, puntos.len());
    assert!(cantidad > 0 && cantidad <= 37);
    let esperado = particulas_geo::convertir(&antes[..11 * N], f32::from_bits(antes[CONTROL])).unwrap();
    assert_eq!(puntos, esperado);
    assert!(puntos.iter().any(|p| p.punto.id >= 256));
    assert!(particulas_geo::exportar(gpu, &ruta).is_err());
    std::fs::remove_file(ruta).unwrap();
    gpu.activar_campos(false).unwrap();
    let belleza = imagen(gpu, p);
    for vista in [1.0, 2.0, 3.0] {
        p[12] = vista;
        assert_ne!(belleza, imagen(gpu, p));
    }
    p[12] = 0.0;
    p[16..19].copy_from_slice(&[-1.0, 0.0, 0.0]);
    p[20..23].copy_from_slice(&[0.0, 0.0, -0.5]);
    p[28..31].copy_from_slice(&[2.0, 0.5, 0.5]);
    assert_ne!(belleza, imagen(gpu, p));
    assert_eq!(antes, gpu.inspeccionar_campos().unwrap());
    gpu.activar_campos(true).unwrap();
}
fn suelo_y_cuarentena(gpu: &mut Gpu) {
    let p = bloque();
    let mut estado = vec![0.0f32; 12 * N];
    estado[0] = 0.5;
    estado[N] = 0.001;
    estado[2 * N] = 0.5;
    estado[4 * N] = -1.0;
    estado[6 * N] = 0.2;
    estado[7 * N] = 1.0;
    estado[8 * N] = 1.0;
    estado[9 * N] = 0.8;
    estado[10 * N] = 1.0;
    estado[CONTROL] = 1.0;
    gpu.cargar_campos(&estado.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
    cuadro(gpu, &p);
    telar_kernel::particulas::referencia::paso(&mut estado, &p[..10].try_into().unwrap()).unwrap();
    comparar(gpu, &estado);
    assert_eq!(estado[N].to_bits(), 0.0f32.to_bits());
    assert!(estado[4 * N] > 0.0);
    let mut roto = gpu.inspeccionar_campos().unwrap();
    roto[3 * N] = f32::NAN.to_bits();
    gpu.cargar_campos(&roto).unwrap();
    cuadro(gpu, &p);
    let resultado = gpu.inspeccionar_campos().unwrap();
    assert_eq!(resultado[10 * N], (-1.0f32).to_bits());
    assert_eq!(resultado[CONTROL + 3], 1.0f32.to_bits());
    let ruta = std::env::temp_dir().join(format!("telar-particulas-invalido-{}.geo", std::process::id()));
    assert!(particulas_geo::exportar(gpu, &ruta).is_err());
    assert!(!ruta.exists());
    gpu.limpiar_campos().unwrap();
    assert!(gpu.inspeccionar_campos().unwrap().iter().all(|v| *v == 0));
}

fn limites(gpu: &mut Gpu) {
    let mut p = bloque();
    p[0] = 1.0;
    p[8] = 0.0;
    let mut inicial = vec![0.0f32; 12 * N];
    inicial[8 * N] = 65535.0;
    gpu.cargar_campos(&inicial.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
    cuadro(gpu, &p);
    let limite = gpu.inspeccionar_campos().unwrap();
    assert_eq!(limite[8 * N], 65535.0f32.to_bits());
    assert_eq!(limite[10 * N], 0.0f32.to_bits());
    inicial.fill(0.0);
    inicial[CONTROL] = 3599.99;
    gpu.cargar_campos(&inicial.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
    cuadro(gpu, &p);
    telar_kernel::particulas::referencia::paso(&mut inicial, &p[..10].try_into().unwrap()).unwrap();
    comparar(gpu, &inicial);
    assert_eq!(inicial[CONTROL].to_bits(), 3600.0f32.to_bits());
    let antes = gpu.inspeccionar_campos().unwrap();
    cuadro(gpu, &p);
    let despues = gpu.inspeccionar_campos().unwrap();
    assert_eq!(antes[..11 * N], despues[..11 * N]);
    assert_eq!(despues[CONTROL + 6], 0.0f32.to_bits());
    assert_eq!(despues[CONTROL + 7], 1.0f32.to_bits());
}
