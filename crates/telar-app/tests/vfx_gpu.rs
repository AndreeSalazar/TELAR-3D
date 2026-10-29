//! Combustible/reacción y secundarios: contratos numéricos en Vulkan real.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "ensayo acotado a 8³")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const N: usize = 512;
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-vfx-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}

fn preparar() -> (telar_ventana::Ventana, Gpu) {
    let plan = telar_kernel::vfx::compilar(telar_kernel::humo3d::Configuracion { lado: 8, iteraciones: 8 }, 1).unwrap();
    for (i, c) in plan.codigos.iter().enumerate() {
        validar(c, i);
    }
    let frag = telar_render::volumen::fragmentos_vfx(8, 16).unwrap();
    validar(&frag, 18);
    validar(&telar_render::vfx_final::programa(32, false).unwrap(), 19);
    validar(&telar_render::vfx_final::programa(32, true).unwrap(), 20);
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
        Sombreadores { vertices: &telar_render::visor_rayos::vertices(), fragmentos: &frag },
        PlanCampos {
            elementos: 512,
            palabras: telar_kernel::vfx::canales(8),
            programas: &plan.codigos,
            secuencia: &plan.orden,
        },
    )
    .unwrap();
    gpu.subir_malla(
        &[[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]],
        &[0, 2, 1],
    )
    .unwrap();
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
    p[0] = 0.13;
    p[5] = 0.01;
    p[6] = 1.0;
    p[10] = 8.0;
    p[11] = 0.02;
    p[12] = 0.005;
    p[13] = 1.0;
    p[15] = 0.52;
    p[19] = 0.5;
    p[27] = 1.0;
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

fn cargar(gpu: &Gpu, estado: &[f32]) {
    gpu.cargar_campos(&estado.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
}
fn leer(gpu: &Gpu) -> Vec<f32> {
    gpu.inspeccionar_campos().unwrap().into_iter().map(f32::from_bits).collect()
}
fn base(vfx: telar_ui::Vfx) -> Vec<f32> {
    let mut s = vec![0.0; 26 * N];
    s[19 * N..19 * N + 28].copy_from_slice(&vfx.0);
    s
}
fn cerca(a: f32, b: f32) {
    assert!((a - b).abs() < 0.00001, "{a} != {b}");
}
#[test]
fn reaccion_acotada_y_controles_en_gpu() {
    let (ventana, mut gpu) = preparar();
    let p = bloque();
    let mut vfx = telar_ui::Vfx::default();
    vfx.0[0] = 0.0;
    vfx.0[24] = 0.0;
    vfx.0[26] = 0.0;
    let mut s = base(vfx);
    s[4 * N..5 * N].fill(1.0);
    s[17 * N..18 * N].fill(1.0);
    gpu.secuencia_campos(&[14, 15, 0]).unwrap();
    cargar(&gpu, &s);
    cuadro(&mut gpu, &p);
    let q = leer(&gpu);
    for id in 0..N {
        cerca(q[18 * N + id], 0.97);
        cerca(q[8 * N + id], 0.024);
        cerca(q[9 * N + id], (0.998 + 0.12) * 0.996);
    }
    assert_eq!(&s[..5 * N], &q[..5 * N]);
    assert_eq!(&s[19 * N..], &q[19 * N..]);
    // Sin ignición no hay reacción; dt/campos son los mismos.
    vfx.0[2] = 10.0;
    s[19 * N..19 * N + 28].copy_from_slice(&vfx.0);
    cargar(&gpu, &s);
    cuadro(&mut gpu, &p);
    let q = leer(&gpu);
    for id in 0..N {
        cerca(q[18 * N + id], 1.0);
        cerca(q[8 * N + id], 0.0);
    }
    // Agotamiento: nunca consume más combustible disponible.
    vfx.0[2] = 0.0;
    vfx.0[1] = 20.0;
    s[19 * N..19 * N + 28].copy_from_slice(&vfx.0);
    s[17 * N..18 * N].fill(0.001);
    cargar(&gpu, &s);
    cuadro(&mut gpu, &p);
    let q = leer(&gpu);
    assert!(q[18 * N..19 * N].iter().all(|v| *v >= 0.0 && *v < 1e-6));
    // Actualización parcial de parámetros: no toca ningún otro byte y rechaza exceso.
    let antes = gpu.inspeccionar_campos().unwrap();
    gpu.cargar_rango_campos(u32::try_from(19 * N + 7).unwrap(), &[17.0f32.to_bits()]).unwrap();
    let despues = gpu.inspeccionar_campos().unwrap();
    assert_eq!(&antes[..19 * N + 7], &despues[..19 * N + 7]);
    assert_eq!(&antes[19 * N + 8..], &despues[19 * N + 8..]);
    assert!(gpu.cargar_rango_campos(u32::MAX, &[0]).is_err());
    secundarios(&mut gpu, p);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
fn secundarios(gpu: &mut Gpu, p: [f32; 32]) {
    let mut v = telar_ui::Vfx::default();
    v.0[0] = 0.0;
    v.0[7] = 17.0;
    v.0[11] = 0.0;
    v.0[23] = 0.0;
    v.0[24] = 0.0;
    let s = base(v);
    cargar(gpu, &s);
    gpu.secuencia_campos(&telar_kernel::humo3d::orden_temporal(8, 1).unwrap()).unwrap();
    cuadro(gpu, &p);
    let primero = leer(gpu);
    assert_eq!(primero[20 * N + 10 * 128..20 * N + 11 * 128].iter().filter(|v| **v > 0.0).count(), 17);
    for id in 0..17 {
        cerca(primero[20 * N + 6 * 128 + id], 0.01);
        cerca(primero[20 * N + 7 * 128 + id], 2.0);
        for frame in 0..4 {
            for eje in 0..3 {
                cerca(primero[20 * N + 1536 + (frame * 3 + eje) * 128 + id], primero[20 * N + eje * 128 + id]);
            }
        }
    }
    v.0[20] += 0.1;
    let s = base(v);
    cargar(gpu, &s);
    cuadro(gpu, &p);
    let cambiado = leer(gpu);
    for id in 0..17 {
        cerca(cambiado[20 * N + id] - primero[20 * N + id], 0.1);
    }
    v.0[8] += 1.0;
    cargar(gpu, &base(v));
    cuadro(gpu, &p);
    assert_ne!(cambiado[20 * N..20 * N + 128], leer(gpu)[20 * N..20 * N + 128]);
}

#[test]
fn hdr_cumple_transmision_y_no_avanza_la_simulacion() {
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 64,
        alto: 64,
        visible: false,
        ..Default::default()
    })
    .unwrap();
    let manejadores = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, ..Default::default() },
        Superficie { instancia: manejadores.instancia, ventana: manejadores.ventana },
        ventana.tamano(),
    )
    .unwrap();
    let codigos =
        [telar_render::vfx_final::programa(32, false).unwrap(), telar_render::vfx_final::programa(32, true).unwrap()];
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::volumen::fragmentos_vfx(32, 16).unwrap(),
        },
        PlanCampos { elementos: 32768, palabras: 31, programas: &codigos, secuencia: &[1] },
    )
    .unwrap();
    let celdas = 32768usize;
    let base = telar_render::vfx_final::base(32) as usize;
    let mut v = telar_ui::Vfx::default();
    v.0[15] = 0.0;
    v.0[16] = 1.0;
    v.0[18] = 0.0;
    let mut estado = vec![0.0; 31 * celdas];
    estado[3 * celdas..4 * celdas].fill(0.5);
    estado[19 * celdas..19 * celdas + 28].copy_from_slice(&v.0);
    cargar(&gpu, &estado);
    let p = bloque();
    gpu.activar_campos(true).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    gpu.secuencia_campos(&[0]).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    let salida = gpu.inspeccionar_campos().unwrap();
    let centro = base + (128 * 256 + 128) * 5;
    assert_eq!(salida[centro + 3], 1);
    assert_eq!(salida[..base], estado[..base].iter().map(|v| v.to_bits()).collect::<Vec<_>>());
    let rojo = f32::from_bits(salida[centro]);
    let esperado = 0.008 * (-0.5f32).exp();
    assert!((rojo / esperado - 1.0).abs() < 0.02);
    gpu.ejecutar_campos(&p).unwrap();
    let segunda = gpu.inspeccionar_campos().unwrap();
    assert_eq!(segunda[centro + 3], 2);
    gpu.cargar_rango_campos(u32::try_from(19 * celdas + 16).unwrap(), &[2.0f32.to_bits()]).unwrap();
    gpu.secuencia_campos(&[1]).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    gpu.secuencia_campos(&[0]).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    let absorbido = gpu.inspeccionar_campos().unwrap();
    assert!(f32::from_bits(absorbido[centro]) < rojo);
    gpu.cargar_rango_campos(u32::try_from(19 * celdas + 15).unwrap(), &[12.0f32.to_bits()]).unwrap();
    gpu.secuencia_campos(&[1]).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    gpu.secuencia_campos(&[0]).unwrap();
    gpu.ejecutar_campos(&p).unwrap();
    let iluminado = gpu.inspeccionar_campos().unwrap();
    assert!(f32::from_bits(iluminado[centro]) > f32::from_bits(absorbido[centro]));
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
