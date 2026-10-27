//! Captura RGBA8 del estudio con Vulkan, sin abrir una ventana.
use telar_gpu::{
    Calculo, CapaInterfaz, Constantes, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones,
    ParametrosProcedurales, PlanProcedural, Sombreadores,
};
use telar_ui::{Distribucion, Estudio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = Calculo::nuevo(&Opciones::default())?;
    let codigo = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let relieve = hilo::ejemplos::relieve(1.3)?.compilar();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "relieve", codigo: &relieve, dominio: DominioProcedural::Puntos });
    let mut residente = gpu.procedural_con_plan(256, PlanProcedural::nuevo(&etapas)?)?;
    let estudio = Estudio::default();
    let diseno = Distribucion::nueva((1280, 720));
    residente.paso(ParametrosProcedurales { lado: 256, fase: 0.6, amplitud: estudio.amplitud })?;
    let escena = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let vertices = estudio.dibujar(diseno, 65_536);
    let constantes = Constantes {
        matriz: nudo::Camara { distancia: 7.0, ..nudo::Camara::default() }
            .matriz(diseno.visor.ancho / diseno.visor.alto),
        luz: nudo::mate::normalizar([0.45, 0.8, 0.4]),
        color: estudio.color(),
        cielo: telar_gpu::eleccion::hex(0x4a_5a78),
        suelo: telar_gpu::eleccion::hex(0x20_1812),
    };
    let pixeles = residente.capturar_estudio(
        Sombreadores { vertices: &escena[0], fragmentos: &escena[1] },
        (1280, 720),
        telar_gpu::eleccion::hex(0x0b_1118),
        &constantes,
        CapaInterfaz {
            shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
            vertices: &vertices,
            area: diseno.area(),
        },
    )?;
    let ruta = std::env::args().nth(1).unwrap_or_else(|| "target/estudio.rgba".to_owned());
    std::fs::write(&ruta, pixeles)?;
    println!("captura RGBA8 1280x720: {ruta}");
    Ok(())
}
