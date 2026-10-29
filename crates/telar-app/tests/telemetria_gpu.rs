//! Consultas de tiempo y snapshot no alteran campos ni exigen envíos de lectura separados.
#![allow(clippy::unwrap_used, reason = "ensayo de integración con fallos explícitos")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
fn preparar() -> (telar_ventana::Ventana, Gpu, [f32; 32]) {
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 96,
        alto: 96,
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
    let plan = telar_kernel::humo3d::Configuracion { lado: 8, iteraciones: 8 }.compilar_temporal(1).unwrap();
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::volumen::fragmentos(8, 16).unwrap(),
        },
        PlanCampos { elementos: 512, palabras: 17, programas: &plan.codigos, secuencia: &plan.orden },
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
    gpu.activar_campos(false).unwrap();
    let mut bloque = [0.0; 32];
    bloque[6] = 1.0;
    bloque[19] = 0.5;
    bloque[27] = 1.0;
    bloque[31] = 1.0;
    bloque[16..19].copy_from_slice(&[0.0, 0.0, -1.0]);
    bloque[20..23].copy_from_slice(&[0.5, 0.0, 0.0]);
    bloque[24..27].copy_from_slice(&[0.0, 0.5, 0.0]);
    bloque[28..31].copy_from_slice(&[0.5, 0.5, 2.0]);
    (ventana, gpu, bloque)
}
fn cuadros(gpu: &mut Gpu, bloque: &[f32; 32], cantidad: u32) {
    let mut dibujados = 0;
    for _ in 0..cantidad + 8 {
        if gpu.dibujar_bloque_interfaz([0.0, 0.0, 0.0, 1.0], bloque).unwrap() {
            dibujados += 1;
        }
        if dibujados == cantidad {
            return;
        }
    }
    panic!("no terminó la presentación");
}
#[test]
fn observa_sin_modificar_y_reutiliza_juegos_al_redimensionar() {
    let (ventana, mut gpu, bloque) = preparar();
    assert!(gpu.activar_telemetria(true, Some(u32::MAX)).is_err());
    let mut estado = vec![0u32; 17 * 512];
    let control = [0.25f32, 2.0, 3.0, 0.01, 0.02, 1.0, 0.5, 2.0, 0.0, 0.0, 0.008, 0.2, 0.5, 1.0, 0.0, 0.0];
    estado[16 * 512..16 * 512 + 16].copy_from_slice(&control.map(f32::to_bits));
    gpu.cargar_campos(&estado).unwrap();
    gpu.activar_telemetria(true, Some(16 * 512)).unwrap();
    cuadros(&mut gpu, &bloque, 8);
    let muestra = gpu.telemetria();
    assert!(muestra.muestras >= 6);
    assert_eq!(muestra.control.unwrap().map(f32::to_bits), control.map(f32::to_bits));
    if let Some(ms) = muestra.milisegundos {
        assert!(ms.iter().all(|v| v.is_finite() && *v >= 0.0));
        assert!(ms.iter().sum::<f64>() > 0.0);
    }
    assert_eq!(gpu.inspeccionar_campos().unwrap(), estado);
    let carga = gpu.carga_campos();
    assert_eq!(carga.bytes, 17 * 512 * 4 + 4);
    assert_eq!(carga.grupos, 8);
    assert_eq!(carga.programas, 16);
    let heaps = gpu.memoria_heaps();
    assert!(!heaps.is_empty());
    assert_eq!(heaps.iter().filter(|h| h.local).map(|h| h.capacidad).sum::<u64>(), gpu.perfil().memoria_local);
    for h in heaps {
        assert_eq!(h.uso.is_some(), h.presupuesto.is_some());
    }
    gpu.redimensionar(120, 80);
    cuadros(&mut gpu, &bloque, 8);
    assert_eq!(gpu.telemetria().control.unwrap().map(f32::to_bits), control.map(f32::to_bits));
    gpu.activar_telemetria(false, None).unwrap();
    cuadros(&mut gpu, &bloque, 4);
    let detenida = gpu.telemetria().muestras;
    cuadros(&mut gpu, &bloque, 4);
    assert_eq!(gpu.telemetria().muestras, detenida);
    gpu.activar_telemetria(true, Some(16 * 512)).unwrap();
    cuadros(&mut gpu, &bloque, 6);
    assert!(gpu.telemetria().muestras > detenida);
    assert_eq!(gpu.inspeccionar_campos().unwrap(), estado);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
#[test]
fn ejecutable_exporta_csv_y_rechaza_reemplazo() {
    let carpeta = std::env::temp_dir().join(format!("telar-monitor-{}", std::process::id()));
    std::fs::create_dir(&carpeta).unwrap();
    let ruta = carpeta.join("medicion.csv");
    let ejecutar = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
            .args(["--plano-vfx", "--prueba", "--benchmark-gpu"])
            .arg(&ruta)
            .output()
            .unwrap()
    };
    let salida = ejecutar();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
    let bytes = std::fs::read(&ruta).unwrap();
    let csv = String::from_utf8(bytes.clone()).unwrap();
    assert!(csv.lines().count() >= 3);
    let fila: Vec<_> = csv.lines().last().unwrap().split(',').collect();
    assert_eq!(fila.len(), 10);
    assert!(fila[1].parse::<f64>().unwrap() > 0.0);
    assert!(ruta.with_extension("txt").exists());
    assert!(!ejecutar().status.success());
    assert_eq!(std::fs::read(&ruta).unwrap(), bytes);
    std::fs::remove_file(&ruta).unwrap();
    std::fs::remove_file(ruta.with_extension("txt")).unwrap();
    std::fs::remove_dir(carpeta).unwrap();
}
