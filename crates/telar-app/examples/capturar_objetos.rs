//! Captura reproducible del editor de objetos para inspección visual.
#[allow(dead_code, reason = "el ejemplo solo usa los constructores de geometría compartidos")]
#[path = "../src/escena_final.rs"]
mod escena_final;
#[allow(dead_code, reason = "la captura reutiliza la traducción de escena, no el bucle nativo")]
#[path = "../src/render_objetos.rs"]
mod render_objetos;
#[allow(dead_code, reason = "el adaptador compartido declara exportación que el ejemplo no invoca")]
#[path = "../src/salida_hdr.rs"]
mod salida_hdr;
#[path = "../src/tipografia.rs"]
mod tipografia;
use telar_gpu::{Calculo, CapaInterfaz, Opciones, Sombreadores};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let ruta = args.first().map_or("target/editor-objetos.rgba", String::as_str);
    let mut editor = telar_ui::EditorObjetos::default();
    if let Some(v) = args.get(1) {
        editor.escena.seleccionado = v.parse()?;
    }
    let diseno = telar_ui::DisenoObjetos::nuevo((1280, 800));
    let trabajo = render_objetos::trabajo(&editor.escena, Some(diseno.visor.ancho / diseno.visor.alto))?;
    let shaders = [telar_render::visor_rayos::vertices(), telar_render::visor_rayos::fragmentos(&trabajo, true)?];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let fuente = tipografia::crear()?;
    let vertices = editor.dibujar(diseno, &fuente)?;
    let gpu = Calculo::nuevo(&Opciones { muestras: 1, ..Opciones::default() })?;
    let malla = [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
    let captura = gpu.capturar_malla_interfaz(
        Sombreadores { vertices: &shaders[0], fragmentos: &shaders[1] },
        (&malla, &[0, 2, 1]),
        (1280, 800),
        [0.04, 0.05, 0.07, 1.0],
        &[0.0; 32],
        CapaInterfaz {
            shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
            vertices: &vertices,
            area: diseno.area(),
            atlas: Some(telar_gpu::AtlasInterfaz {
                ancho: fuente.ancho,
                alto: fuente.alto,
                cobertura: &fuente.cobertura,
            }),
        },
    )?;
    std::fs::write(ruta, captura.pixeles)?;
    drop(gpu);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("captura con avisos".into());
    }
    Ok(())
}
