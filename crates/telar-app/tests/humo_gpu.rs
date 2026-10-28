//! MAC residente contra referencia escalar; proyección, persistencia y pausa reales.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "ensayo pequeño con aserciones")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
fn validar(codigo: &[u32], indice: usize) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|v| v.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-humo-{}-{indice}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn dibujar(gpu: &mut Gpu, bloque: &[f32; 32]) {
    for _ in 0..10 {
        if gpu.dibujar_bloque_interfaz([0.0, 0.0, 0.0, 1.0], bloque).unwrap() {
            return;
        }
    }
    panic!("no se pudo dibujar el ensayo de humo");
}
#[test]
fn presion_y_adveccion_coinciden_con_referencia() {
    let lado = 17;
    let n = lado * lado;
    let iteraciones = 64;
    let plan =
        telar_kernel::humo::Configuracion { lado: u32::try_from(lado).unwrap(), iteraciones }.compilar().unwrap();
    let fragmentos = telar_render::humo::fragmentos(u32::try_from(lado).unwrap()).unwrap();
    for (i, codigo) in plan.codigos.iter().chain([&fragmentos]).enumerate() {
        validar(codigo, i);
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
            elementos: u32::try_from(n).unwrap(),
            palabras: 12,
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
    assert!(gpu.inspeccionar_campos().unwrap().iter().all(|v| *v == 0));
    let mut referencia = vec![0.0; n * 12];
    for y in 0..lado {
        for x in 0..lado {
            let pixel = y * lado + x;
            referencia[pixel] = if x + 1 < lado { ((x + y) as f32 * 0.43).sin() * 0.1 } else { 0.0 };
            referencia[n + pixel] = if y + 1 < lado { ((x * 2 + y) as f32 * 0.31).cos() * 0.1 } else { 0.0 };
            referencia[2 * n + pixel] = (x + y) as f32 / 32.0;
            referencia[3 * n + pixel] = (x as f32 * 0.2).sin().abs();
        }
    }
    let inicial = referencia.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    gpu.cargar_campos(&inicial).unwrap();
    assert!(gpu.cargar_campos(&inicial[..20]).is_err());
    let parametros = [0.08, 2.0, 1.5, 0.1, 0.15, 0.016, 1.0, 64.0];
    let mut bloque = [0.0; 32];
    bloque[..8].copy_from_slice(&parametros);
    for _ in 0..4 {
        telar_kernel::humo::referencia::paso(lado, iteraciones, &mut referencia, &parametros).unwrap();
        dibujar(&mut gpu, &bloque);
        let actual = gpu.inspeccionar_campos().unwrap();
        comparar(&actual, &referencia, n);
        let norma = |plano: usize| {
            actual[plano * n..(plano + 1) * n].iter().map(|v| f64::from(f32::from_bits(*v)).powi(2)).sum::<f64>()
        };
        assert!(norma(11) < norma(8) * 0.5, "la proyección no redujo la divergencia");
    }
    gpu.activar_campos(false).unwrap();
    let pausado = gpu.inspeccionar_campos().unwrap();
    dibujar(&mut gpu, &bloque);
    assert_eq!(pausado, gpu.inspeccionar_campos().unwrap());
    gpu.cargar_campos(&inicial).unwrap();
    dibujar(&mut gpu, &bloque);
    assert_eq!(inicial, gpu.inspeccionar_campos().unwrap());
    gpu.activar_campos(true).unwrap();
    dibujar(&mut gpu, &bloque);
    let primera = gpu.inspeccionar_campos().unwrap();
    gpu.cargar_campos(&inicial).unwrap();
    dibujar(&mut gpu, &bloque);
    assert_eq!(primera, gpu.inspeccionar_campos().unwrap());
    let orden = vec![0, 1, 2, 3, 2, 3, 4, 5];
    gpu.secuencia_campos(&orden).unwrap();
    assert!(gpu.secuencia_campos(&[99]).is_err());
    assert!(gpu.secuencia_campos(&[]).is_err());
    let mut cuatro = primera.iter().map(|v| f32::from_bits(*v)).collect::<Vec<_>>();
    telar_kernel::humo::referencia::paso(lado, 4, &mut cuatro, &parametros).unwrap();
    dibujar(&mut gpu, &bloque);
    comparar(&gpu.inspeccionar_campos().unwrap(), &cuatro, n);
    gpu.activar_campos(false).unwrap();
    gpu.limpiar_campos().unwrap();
    dibujar(&mut gpu, &bloque);
    assert!(gpu.inspeccionar_campos().unwrap().iter().all(|v| *v == 0));
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
fn comparar(actual: &[u32], referencia: &[f32], n: usize) {
    for (i, (real, esperado)) in actual.iter().zip(referencia).enumerate() {
        let real = f32::from_bits(*real);
        assert!(real.is_finite());
        assert!((real - esperado).abs() < 0.0003, "canal {} celda {}: GPU={real}, CPU={esperado}", i / n, i % n);
    }
}
