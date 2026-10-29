//! Captura reproducible del humo con tolerancias y estado GPU visibles.
#[allow(dead_code, reason = "cámara compartida, sin objetos en este ejemplo")]
#[path = "../src/escena_final.rs"]
mod escena_final;
#[allow(dead_code, reason = "el ejemplo captura un presupuesto fijo, sin edición de secuencia")]
#[path = "../src/humo_vista.rs"]
mod humo_vista;
#[path = "../src/tipografia.rs"]
mod tipografia;
#[allow(dead_code, reason = "adaptador 3D compartido por la vista de humo")]
#[path = "../src/volumen_vista.rs"]
mod volumen_vista;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ruta = std::env::args().nth(1).unwrap_or_else(|| "target/presion-adaptativa.ppm".into());
    let mut editor = telar_ui::EditorHumo::default();
    editor.pagina = 1;
    let fuente = tipografia::crear()?;
    let (ventana, mut gpu) = humo_vista::preparar(&editor, &fuente, false, true)?;
    let d = humo_vista::diseno(gpu.imagenes().1);
    gpu.actualizar_interfaz(&editor.dibujar(d, &fuente)?, d.area())?;
    let srgb = gpu.imagenes().0 != telar_gpu::eleccion::FORMATO_SRGB;
    let bloque = humo_vista::bloque(&editor, srgb);
    let mut pasos = 0;
    while pasos < 240 {
        if gpu.dibujar_bloque_interfaz([0.03, 0.04, 0.05, 1.0], &bloque)? {
            pasos += 1;
        }
    }
    editor.mensaje = humo_vista::datos(&gpu)?;
    println!("{} · {pasos} pasos · {}", editor.mensaje, gpu.dispositivo());
    humo_vista::capturar_bloque(
        &mut gpu,
        &editor,
        &fuente,
        std::path::Path::new(&ruta),
        &humo_vista::bloque(&editor, true),
    )?;
    drop(gpu);
    drop(ventana);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("captura con avisos Vulkan".into());
    }
    Ok(())
}

#[allow(dead_code, reason = "adaptador compartido por el panel físico; este ejemplo sigue siendo humo")]
#[path = "../src/particulas_vista.rs"]
mod particulas_vista;
