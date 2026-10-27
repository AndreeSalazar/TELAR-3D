//! Diagnóstico reproducible: tiempos GPU del dibujo, sin confundirlos con FPS.
use telar_gpu::{
    Calculo, CapaInterfaz, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural, Sombreadores,
};
#[path = "../src/arranque.rs"]
mod arranque;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let antialias = !std::env::args().any(|arg| arg == "--sin-msaa");
    let gpu = Calculo::nuevo(&Opciones { antialias, ..Opciones::default() })?;
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let etapas = KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas();
    let cabecera = arranque::crear(gpu.perfil(), 256, etapas.len())?;
    let mut residente = gpu.procedural_con_plan(256, PlanProcedural::nuevo(&etapas)?.con_arranque(&cabecera)?)?;
    residente.paso(ParametrosProcedurales { lado: 256, fase: 0.6, amplitud: 0.65 })?;
    let vertices = telar_render::vertices();
    let luz = telar_render::LuzArea::default();
    let variantes = [
        ("area-4-sin-sombra", telar_render::LuzArea { sombras: false, ..luz }),
        ("area-1-sombra-32", telar_render::LuzArea { muestras: 1, ..luz }),
        ("area-4-sombra-32", luz),
    ];
    let shaders =
        variantes.map(|(nombre, luz)| Ok::<_, &'static str>((nombre, telar_render::fragmentos_rejilla(luz)?)));
    let shaders = shaders.into_iter().collect::<Result<Vec<_>, _>>()?;
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let camara = nudo::Camara { distancia: 7.0, ..Default::default() };
    let bloque = telar_render::Parametros {
        matriz: camara.matriz(1280.0 / 720.0),
        ojo: camara.posicion(),
        luz: [0.0, 1.0, 0.0],
        material: telar_render::Material { color: [0.5, 0.2, 0.1], rugosidad: 0.3, metal: 0.8 },
        exposicion: 1.4,
        intensidad: 16.0,
        codificar_srgb: true,
    }
    .bloque()?;
    println!("GPU {} · Vulkan validación {} · 1280x720 · 256x256 puntos", gpu.dispositivo(), gpu.validacion());
    println!("MSAA solicitado: {antialias} (capacidad real devuelta por las capturas)");
    println!(
        "Timestamps de dibujo+limpieza+barreras, sin overlay visible; excluye creación, compute y descarga. 2 calentamientos y 9 muestras intercaladas."
    );
    let mut tiempos = [Vec::new(), Vec::new(), Vec::new()];
    for vuelta in 0..11 {
        for (i, (_, fragmentos)) in shaders.iter().enumerate() {
            let captura = residente.medir_estudio(
                Sombreadores { vertices: &vertices, fragmentos },
                (1280, 720),
                [0.0, 0.0, 0.0, 1.0],
                &bloque,
                CapaInterfaz {
                    shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                    vertices: &[],
                    area: [0, 0, 1280, 720],
                    atlas: None,
                },
            )?;
            if vuelta >= 2 {
                if vuelta == 2 && i == 0 {
                    println!("MSAA real {}x", captura.muestras);
                }
                tiempos[i].push(captura.nanosegundos.ok_or("la cola no admite timestamps")? / 1_000_000.0);
            }
        }
    }
    for ((nombre, _), tiempos) in shaders.iter().zip(&mut tiempos) {
        tiempos.sort_by(f64::total_cmp);
        println!("{nombre}: mediana {:.4} ms · mínimo {:.4} · máximo {:.4}", tiempos[4], tiempos[0], tiempos[8]);
    }
    for (nombre, fase) in [("dinamico", 0.7), ("estatico", 0.7)] {
        residente.paso(ParametrosProcedurales { lado: 256, fase, amplitud: 0.65 })?;
        let diagnostico = residente.inspeccionar()?;
        println!(
            "{nombre}: grupos {} · recalculadas {} · omitidas {} · tiempos compute ns {:?}",
            diagnostico.control[0], diagnostico.control[36], diagnostico.control[37], diagnostico.nanosegundos
        );
    }
    println!(
        "{} bytes de geometría subidos; las lecturas anteriores son diagnósticas explícitas.",
        residente.actividad().geometria_subida
    );
    drop(residente);
    drop(gpu);
    if telar_gpu::avisos_de_validacion() != 0 {
        return Err("avisos Vulkan durante medición".into());
    }
    Ok(())
}
