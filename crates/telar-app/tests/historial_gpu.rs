//! Historial real entre fotogramas: media, límite y reinicio, sin fragment stores.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "pruebas pequeñas con aserciones"
)]
#[path = "../src/escena_final.rs"]
mod escena_final;
use telar_gpu::{Gpu, Opciones, Sombreadores, Superficie};
use telar_render::escena_rayos::Trabajo;

fn trabajo() -> Trabajo {
    Trabajo {
        tamano: [17, 13],
        muestras: 8,
        semilla: 1,
        camara: escena_final::camara([4.0, 3.0, 6.0], [0.0, 0.5, 0.0], 17.0 / 13.0).unwrap(),
        triangulos: escena_final::triangulos(0.0),
        luz: [-1.0, 4.0, 1.0],
        radio_luz: 0.65,
        energia: 25.0,
        color_luz: [1.0; 3],
        exposicion: 1.0,
        sombras: true,
    }
}
fn validar(codigo: &[u32], nombre: &str) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|v| std::path::PathBuf::from(v).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-historial-{nombre}-{}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
fn dibujar(gpu: &mut Gpu, bloque: &[f32; 32], cantidad: usize) {
    let mut hechos = 0;
    for _ in 0..cantidad + 10 {
        if gpu.dibujar_bloque_interfaz([0.0; 4], bloque).unwrap() {
            hechos += 1;
            if hechos == cantidad {
                break;
            }
        }
    }
    assert_eq!(hechos, cantidad);
}
#[test]
fn historial_progresivo_gpu_coincide_con_media_y_se_detiene_sin_borrar_por_exposicion() {
    let trabajo = trabajo();
    let compute = telar_render::historial_rayos::acumular(&trabajo, [17, 13]).unwrap();
    let vertices = telar_render::visor_rayos::vertices();
    let fragmentos = telar_render::historial_rayos::resolver();
    for (codigo, nombre) in [(&compute, "compute"), (&vertices, "vertices"), (&fragmentos, "resolver")] {
        validar(codigo, nombre);
    }
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        visible: false,
        ancho: 128,
        alto: 96,
        ..telar_ventana::Config::default()
    })
    .unwrap();
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, ..Opciones::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )
    .unwrap();
    let shaders = Sombreadores { vertices: &vertices, fragmentos: &fragmentos };
    gpu.preparar_historial(shaders, &compute, [17, 13]).unwrap();
    gpu.subir_malla(
        &[[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]],
        &[0, 2, 1],
    )
    .unwrap();
    assert!(gpu.inspeccionar_historial().unwrap().iter().all(|p| *p == [0; 4]));
    let mut bloque = [0.0; 32];
    bloque[..4].copy_from_slice(&[1.0, 1.0, 17.0, 13.0]);
    dibujar(&mut gpu, &bloque, 1);
    assert!(gpu.inspeccionar_historial().unwrap().iter().all(|p| p[3] == 1));
    dibujar(&mut gpu, &bloque, 7);
    let finalizado = gpu.inspeccionar_historial().unwrap();
    assert!(finalizado.iter().all(|p| p[3] == 8));
    dibujar(&mut gpu, &bloque, 3);
    assert_eq!(finalizado, gpu.inspeccionar_historial().unwrap());
    bloque[0] = 2.0;
    dibujar(&mut gpu, &bloque, 1);
    assert_eq!(finalizado, gpu.inspeccionar_historial().unwrap());
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let capa = telar_gpu::CapaInterfaz {
        shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
        vertices: &[],
        area: [0, 0, 64, 64],
        atlas: None,
    };
    bloque[0] = 1.0;
    let normal = gpu.capturar_historial_interfaz((64, 64), [0.0; 4], &bloque, capa).unwrap();
    bloque[0] = 2.0;
    let expuesto = gpu.capturar_historial_interfaz((64, 64), [0.0; 4], &bloque, capa).unwrap();
    assert_ne!(normal.pixeles, expuesto.pixeles, "exposición cambia visualización");
    assert_eq!(finalizado, gpu.inspeccionar_historial().unwrap(), "capturas convergidas conservan HDR");
    let calculo = telar_gpu::Calculo::nuevo(&Opciones::default()).unwrap();
    let referencia = calculo.ejecutar(&trabajo.compilar().unwrap(), 17 * 13 * 5, 64).unwrap();
    for (suma, esperado) in finalizado.iter().zip(referencia.datos.as_chunks::<5>().0) {
        for i in 0..3 {
            let media = f32::from_bits(suma[i]) / 8.0;
            assert!((media - f32::from_bits(esperado[i])).abs() < 2e-5);
        }
    }
    gpu.preparar_historial(shaders, &compute, [17, 13]).unwrap();
    let mut invalido = bloque;
    invalido[2] = 0.0;
    assert!(gpu.dibujar_bloque_interfaz([0.0; 4], &invalido).is_err());
    dibujar(&mut gpu, &bloque, 1);
    assert!(gpu.inspeccionar_historial().unwrap().iter().all(|p| p[3] == 1));
    drop(calculo);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
