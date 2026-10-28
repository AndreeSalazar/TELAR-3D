//! Capturas RGBA8 del historial vivo a 1/16/256 spp, sin ventanas visibles.
#[allow(dead_code, reason = "adaptadores compartidos de geometría y salida")]
#[path = "../src/escena_final.rs"]
mod escena_final;
#[allow(dead_code, reason = "el ejemplo comparte la traducción, no la exportación final")]
#[path = "../src/render_objetos.rs"]
mod render_objetos;
#[allow(dead_code, reason = "el adaptador compartido declara exportación HDR")]
#[path = "../src/salida_hdr.rs"]
mod salida_hdr;
#[path = "../src/tipografia.rs"]
mod tipografia;
use telar_gpu::{CapaInterfaz, Gpu, Opciones, Sombreadores, Superficie};
use telar_ui::{DisenoObjetos, EditorObjetos};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prefijo = std::env::args().nth(1).unwrap_or_else(|| "target/progresivo".into());
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 1280,
        alto: 800,
        visible: false,
        ..telar_ventana::Config::default()
    })?;
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, ..Opciones::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )?;
    let mut editor = EditorObjetos::default();
    editor.escena.seleccionado = 3;
    editor.escena.poner(2, 64.0)?;
    editor.escena.poner(3, 64.0)?;
    editor.escena.poner(1, 256.0)?;
    let diseno = DisenoObjetos::nuevo((1280, 800));
    let area = diseno.area();
    let fuente = tipografia::crear()?;
    render_objetos::preparar_progresivo(&mut gpu, &editor.escena, [area[2], area[3]])?;
    gpu.subir_malla(
        &[[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]],
        &[0, 2, 1],
    )?;
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    #[allow(clippy::cast_precision_loss, reason = "viewport <=1024×768")]
    let dimensiones = [area[2], area[3]].map(|v| v as f32);
    let mut bloque = [0.0; 32];
    bloque[..4].copy_from_slice(&[1.0, 0.0, dimensiones[0], dimensiones[1]]);
    let mut pasos = 0;
    for objetivo in [1, 16, 256] {
        while pasos + 1 < objetivo {
            if gpu.dibujar_bloque_interfaz([0.04, 0.05, 0.07, 1.0], &bloque)? {
                pasos += 1;
            }
        }
        editor.muestras_vista = objetivo;
        let vertices = editor.dibujar(diseno, &fuente)?;
        let mut captura_bloque = bloque;
        captura_bloque[1] = 1.0;
        let captura = gpu.capturar_historial_interfaz(
            (1280, 800),
            [0.04, 0.05, 0.07, 1.0],
            &captura_bloque,
            CapaInterfaz {
                shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                vertices: &vertices,
                area,
                atlas: Some(telar_gpu::AtlasInterfaz {
                    ancho: fuente.ancho,
                    alto: fuente.alto,
                    cobertura: &fuente.cobertura,
                }),
            },
        )?;
        pasos += 1;
        let historial = gpu.inspeccionar_historial()?;
        if !historial.iter().all(|p| p[3] == objetivo) {
            return Err("contador de historial inesperado".into());
        }
        std::fs::write(format!("{prefijo}-{objetivo}.rgba"), captura.pixeles)?;
        println!("historial GPU {objetivo} spp · {} píxeles", historial.len());
    }
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("capturas con avisos".into());
    }
    Ok(())
}
