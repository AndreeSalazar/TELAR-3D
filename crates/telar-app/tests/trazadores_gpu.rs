//! Muestreo MAC real y acoplamiento unidireccional sin descargar campos en el producto.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "ensayo GPU acotado a 8³")]
#[allow(dead_code, reason = "subset de E/S compartida en el ensayo")]
#[path = "../src/intercambio_archivo.rs"]
mod intercambio_archivo;
#[allow(dead_code, reason = "exportación independiente cubierta por particulas_gpu")]
#[path = "../src/particulas_geo.rs"]
mod particulas_geo;
#[allow(dead_code, reason = "adaptador compartido para exportación de campos")]
#[path = "../src/puntos_mac.rs"]
mod puntos_mac;
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const LADO: u32 = 8;
const N: usize = 512;
const BASE: usize = 17 * N;
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-trazadores-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}

fn preparar(trazadores: bool) -> (telar_ventana::Ventana, Gpu) {
    let config = telar_kernel::humo3d::Configuracion { lado: LADO, iteraciones: 8 };
    let plan = if trazadores {
        telar_kernel::trazadores::compilar(config, 2).unwrap()
    } else {
        config.compilar_temporal(2).unwrap()
    };
    let fragmentos = if trazadores {
        telar_render::volumen::fragmentos_trazadores(8, 16).unwrap()
    } else {
        telar_render::volumen::fragmentos(8, 16).unwrap()
    };
    if trazadores {
        for (i, c) in plan.codigos.iter().enumerate() {
            validar(c, i);
        }
        validar(&fragmentos, 20);
    }
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
        Sombreadores { vertices: &telar_render::visor_rayos::vertices(), fragmentos: &fragmentos },
        PlanCampos {
            elementos: LADO.pow(3),
            palabras: if trazadores { telar_kernel::trazadores::canales(8) } else { 17 },
            programas: &plan.codigos,
            secuencia: &plan.orden,
        },
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
    p[..8].copy_from_slice(&[0.13, 3.0, 2.0, 0.08, 0.15, 0.016, 1.0, 3.0]);
    p[8] = 0.35;
    p[9] = 1.0;
    p[10] = 8.0;
    p[11] = 0.02;
    p[12] = 0.005;
    p[13] = 2.0;
    p[14] = 0.0;
    p[15] = 0.52;
    p[19] = 0.5;
    p[23] = 2.0;
    p[27] = 4.0;
    p[31] = 1.0;
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

fn leer(gpu: &Gpu) -> Vec<f32> {
    gpu.inspeccionar_campos().unwrap().into_iter().map(f32::from_bits).collect()
}
fn cargar(gpu: &Gpu, s: &[f32]) {
    gpu.cargar_campos(&s.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
}
fn comparar(a: &[f32], b: &[f32]) {
    for (i, (a, b)) in a.iter().zip(b).enumerate() {
        assert!((a - b).abs() < 0.00002, "palabra {i}: {a} != {b}");
    }
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
fn trazadores_mac_referencia_y_no_retroalimentacion() {
    assert!(
        telar_kernel::trazadores::compilar(telar_kernel::humo3d::Configuracion { lado: 4, iteraciones: 8 }, 1).is_err()
    );
    let p = bloque();
    let base = {
        let (ventana, mut gpu) = preparar(false);
        for _ in 0..36 {
            cuadro(&mut gpu, &p);
        }
        let salida = gpu.inspeccionar_campos().unwrap();
        drop(gpu);
        drop(ventana);
        salida
    };
    let (ventana, mut gpu) = preparar(true);
    for _ in 0..36 {
        cuadro(&mut gpu, &p);
    }
    let acoplado = gpu.inspeccionar_campos().unwrap();
    assert_eq!(&base[..], &acoplado[..BASE], "los trazadores no deben modificar el gas");
    let valores = leer(&gpu);
    assert!(valores[BASE + 10 * 128..BASE + 11 * 128].contains(&1.0));
    assert!(valores[BASE + 3 * 128..BASE + 6 * 128].iter().any(|v| v.abs() > 0.00001));
    exportar(&mut gpu, &p);
    fabricados(&mut gpu);
    ciclo(&mut gpu);
    guardianes(&mut gpu);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
fn exportar(gpu: &mut Gpu, p: &[f32; 32]) {
    let antes = gpu.inspeccionar_campos().unwrap();
    let ruta = std::env::temp_dir().join(format!("telar-trazadores-{}.geo", std::process::id()));
    let cantidad = particulas_geo::exportar_trazadores(gpu, 8, &ruta).unwrap();
    let intercambio_archivo::ArchivoGeo::Particulas(puntos) = intercambio_archivo::leer(&ruta).unwrap() else {
        panic!("perfil perdido")
    };
    assert_eq!(cantidad, puntos.len());
    assert!(cantidad > 0);
    assert_eq!(
        puntos,
        particulas_geo::convertir(&antes[BASE..BASE + 11 * 128], f32::from_bits(antes[16 * N + 13])).unwrap()
    );
    assert!(particulas_geo::exportar_trazadores(gpu, 8, &ruta).is_err());
    std::fs::remove_file(ruta).unwrap();
    gpu.activar_campos(false).unwrap();
    let trazadores = imagen(gpu, p);
    let mut otro = *p;
    otro[7] = 0.0;
    assert_ne!(trazadores, imagen(gpu, &otro));
    assert_eq!(antes, gpu.inspeccionar_campos().unwrap());
    gpu.activar_campos(true).unwrap();
}
fn fabricados(gpu: &mut Gpu) {
    gpu.secuencia_campos(&[5]).unwrap();
    let mut p = bloque();
    p[8] = 0.0;
    let mut inicial = vec![0.0; 20 * N];
    // Campo afín en caras: la interpolación debe recuperar P con cada offset MAC.
    for id in 0..N {
        let coords = [id % 8, (id / 8) % 8, id / 64];
        for eje in 0..3 {
            inicial[eje * N + id] = (coords[eje] as f32 + 1.0) / 8.0 * 0.1;
        }
    }
    inicial[16 * N + 10] = 0.01;
    inicial[16 * N + 16] = 0.4;
    inicial[16 * N + 13] = 0.4;
    cargar(gpu, &inicial);
    cuadro(gpu, &p);
    let mut esperado = inicial.clone();
    telar_kernel::trazadores::referencia::paso(8, &mut esperado, p[0], p[8], [0.0, 0.52]).unwrap();
    let resultado = leer(gpu);
    comparar(&resultado[BASE..], &esperado[BASE..]);
    assert_eq!(inicial[..13 * N], resultado[..13 * N]);
    for eje in 0..3 {
        for id in 0..128 {
            let posicion_previa = resultado[BASE + eje * 128 + id] / (1.0 + 0.1 * 0.01);
            assert!((resultado[BASE + (eje + 3) * 128 + id] - posicion_previa * 0.1).abs() < 0.00001);
        }
    }
    // Obstáculo de celdas: una plaza viva en la esfera debe morir, sin rebote ficticio.
    let mut esfera = inicial.clone();
    esfera[BASE] = 0.5;
    esfera[BASE + 128] = 0.5;
    esfera[BASE + 256] = 0.5;
    esfera[BASE + 7 * 128] = 2.0;
    esfera[BASE + 8 * 128] = 1.0;
    esfera[BASE + 9 * 128] = 0.2;
    esfera[BASE + 10 * 128] = 1.0;
    p[14] = 0.3;
    p[15] = 0.5;
    cargar(gpu, &esfera);
    cuadro(gpu, &p);
    telar_kernel::trazadores::referencia::paso(8, &mut esfera, p[0], p[8], [0.3, 0.5]).unwrap();
    comparar(&leer(gpu)[BASE..], &esfera[BASE..]);
    assert_eq!(leer(gpu)[BASE + 10 * 128].to_bits(), 0.0f32.to_bits());
    // NaN propio nunca se convierte a índice; se aísla y bloquea la exportación.
    let mut roto = inicial;
    roto[BASE] = f32::NAN;
    cargar(gpu, &roto);
    cuadro(gpu, &p);
    assert_eq!(leer(gpu)[BASE + 10 * 128].to_bits(), (-1.0f32).to_bits());
    let ruta = std::env::temp_dir().join(format!("telar-trazadores-roto-{}.geo", std::process::id()));
    assert!(particulas_geo::exportar_trazadores(gpu, 8, &ruta).is_err());
    assert!(!ruta.exists());
}
fn ciclo(gpu: &mut Gpu) {
    let mut p = bloque();
    p[8] = 0.0;
    p[14] = 0.0;
    let mut estado = vec![0.0; 20 * N];
    estado[16 * N + 10] = 0.04;
    cargar(gpu, &estado);
    for _ in 0..65 {
        cuadro(gpu, &p);
        telar_kernel::trazadores::referencia::paso(8, &mut estado, p[0], p[8], [0.0, 0.52]).unwrap();
        comparar(&leer(gpu)[BASE..], &estado[BASE..]);
        estado[16 * N + 16] += 0.04;
        let mut real = leer(gpu);
        real[16 * N + 16] = estado[16 * N + 16];
        cargar(gpu, &real);
    }
    assert!(estado[BASE + 8 * 128..BASE + 9 * 128].iter().all(|v| *v >= 2.0));
    // Fuente cerrada: no recicla al morir; dt cero conserva cada plaza.
    estado[16 * N + 16] = 3.0;
    p[8] = 0.35;
    for _ in 0..55 {
        cargar(gpu, &estado);
        cuadro(gpu, &p);
        telar_kernel::trazadores::referencia::paso(8, &mut estado, p[0], p[8], [0.0, 0.52]).unwrap();
        comparar(&leer(gpu)[BASE..], &estado[BASE..]);
    }
    assert!(estado[BASE + 10 * 128..BASE + 11 * 128].iter().all(|v| *v == 0.0));
    estado[16 * N + 10] = 0.0;
    cargar(gpu, &estado);
    cuadro(gpu, &p);
    assert_eq!(leer(gpu)[BASE..], estado[BASE..]);
    gpu.limpiar_campos().unwrap();
    assert!(gpu.inspeccionar_campos().unwrap().iter().all(|v| *v == 0));
}

fn guardianes(gpu: &mut Gpu) {
    let mut p = bloque();
    p[8] = 0.0;
    let mut estado = vec![0.0; 20 * N];
    estado[16 * N + 10] = 0.01;
    estado[16 * N + 16] = 0.5;
    estado[BASE + 8 * 128] = 65535.0;
    cargar(gpu, &estado);
    cuadro(gpu, &p);
    let salida = leer(gpu);
    assert_eq!(salida[BASE + 8 * 128].to_bits(), 65535.0f32.to_bits());
    assert_eq!(salida[BASE + 10 * 128].to_bits(), 0.0f32.to_bits());
    // Con dt cero se conserva también un sistema vivo.
    let mut vivo = salida;
    vivo[16 * N + 10] = 0.0;
    cargar(gpu, &vivo);
    cuadro(gpu, &p);
    assert_eq!(leer(gpu)[BASE..], vivo[BASE..]);
    vivo[16 * N + 10] = 0.01;
    vivo[16 * N + 5] = 3.0;
    cargar(gpu, &vivo);
    cuadro(gpu, &p);
    assert_eq!(leer(gpu)[BASE..], vivo[BASE..]);
    // Una muestra no finita se descarta antes de convertir el destino a índice.
    estado.fill(0.0);
    estado[..N].fill(f32::NAN);
    estado[16 * N + 10] = 0.01;
    cargar(gpu, &estado);
    cuadro(gpu, &p);
    let salida = leer(gpu);
    assert!(salida[BASE + 10 * 128..BASE + 11 * 128].iter().all(|v| v.to_bits() == (-1.0f32).to_bits()));
}
