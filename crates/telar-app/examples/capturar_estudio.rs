//! Captura RGBA8 del estudio con Vulkan, sin abrir una ventana.
use telar_gpu::{
    Calculo, CapaInterfaz, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales,
    PlanProcedural, Sombreadores,
};
use telar_ui::Estudio;
#[path = "../src/arranque.rs"]
mod arranque;
#[path = "../src/render.rs"]
mod render;
#[path = "../src/tipografia.rs"]
mod tipografia;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut argumentos = std::env::args().skip(1);
    let ruta = argumentos.next().unwrap_or_else(|| "target/estudio.rgba".to_owned());
    let ancho = argumentos.next().map(|v| v.parse::<u32>()).transpose()?.unwrap_or(1280);
    let alto = argumentos.next().map(|v| v.parse::<u32>()).transpose()?.unwrap_or(720);
    let seleccionado = argumentos.next().map(|v| v.parse::<usize>()).transpose()?.unwrap_or(1).min(3);
    let muestreo = argumentos.next().unwrap_or_else(|| "4".to_owned());
    let muestras = if muestreo == "sin-msaa" { 1 } else { muestreo.parse::<u32>()? };
    let tamano = (ancho, alto);
    let gpu = Calculo::nuevo(&Opciones { muestras, ..Opciones::default() })?;
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let relieve = hilo::ejemplos::relieve_dinamico()?.compilar();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "relieve", codigo: &relieve, dominio: DominioProcedural::Puntos });
    let cabecera = arranque::crear(gpu.perfil(), 256, etapas.len())?;
    let mut residente = gpu.procedural_con_plan(256, PlanProcedural::nuevo(&etapas)?.con_arranque(&cabecera)?)?;
    let mut estudio = Estudio::default();
    estudio.seleccionado = seleccionado;
    estudio.salida.muestras = muestras;
    estudio.salida.muestras_reales = gpu.muestras_de_captura()?;
    let diseno = estudio.distribucion(tamano);
    residente.paso(ParametrosProcedurales { lado: 256, fase: 0.6, amplitud: estudio.onda.amplitud })?;
    let escena = [telar_render::vertices(), telar_render::fragmentos_rejilla(telar_render::LuzArea::default())?];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let fuente = tipografia::crear()?;
    let vertices = estudio.dibujar_con_fuente(diseno, 65_536, &fuente)?;
    let camara = nudo::Camara { distancia: 7.0, ..nudo::Camara::default() };
    let constantes =
        render::bloque(&camara, diseno.visor.ancho / diseno.visor.alto, estudio.color(), &estudio.salida, true)?;
    let captura = residente.medir_estudio(
        Sombreadores { vertices: &escena[0], fragmentos: &escena[1] },
        tamano,
        estudio.fondo_visor(),
        &constantes,
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
    std::fs::write(&ruta, captura.pixeles)?;
    println!("captura RGBA8 {ancho}x{alto} · MSAA {}x: {ruta}", captura.muestras);
    Ok(())
}
