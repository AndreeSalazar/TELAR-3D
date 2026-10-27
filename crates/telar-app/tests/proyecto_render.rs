//! Una escena guardada reconstruye la misma imagen en recursos GPU nuevos.
#![allow(clippy::unwrap_used, reason = "comparación explícita de capturas reales")]
#[path = "../src/arranque.rs"]
mod arranque;
#[allow(dead_code, reason = "el codec compartido incluye E/S y argumentos probados en proyecto.rs")]
#[path = "../src/proyecto.rs"]
mod proyecto;
#[path = "../src/render.rs"]
mod render;
use proyecto::Proyecto;
use telar_gpu::{
    Calculo, CapaInterfaz, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosNodos,
    ParametrosProcedurales, PlanProcedural, Sombreadores,
};
use telar_ui::Estudio;

fn capturar(gpu: &Calculo, proyecto: &Proyecto) -> Vec<u8> {
    let mut estudio = Estudio::default();
    estudio.restaurar_estado(&proyecto.estudio).unwrap();
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let relieve = hilo::ejemplos::relieve_dinamico().unwrap().compilar();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "relieve", codigo: &relieve, dominio: DominioProcedural::Puntos });
    let cabecera = arranque::crear(gpu.perfil(), estudio.rejilla.capacidad, etapas.len()).unwrap();
    let mut residente = gpu
        .procedural_con_plan(
            estudio.rejilla.capacidad,
            PlanProcedural::nuevo(&etapas).unwrap().con_arranque(&cabecera).unwrap(),
        )
        .unwrap();
    residente
        .paso_nodos(ParametrosNodos {
            superficie: ParametrosProcedurales {
                lado: estudio.rejilla.lado,
                fase: proyecto.fase,
                amplitud: estudio.onda.amplitud,
            },
            valores: [estudio.relieve.factor, 0.0, 0.0, 0.0],
        })
        .unwrap();
    let shaders =
        [telar_render::vertices(), telar_render::fragmentos_rejilla(telar_render::LuzArea::default()).unwrap()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let bloque = render::bloque(&proyecto.camara, 1.0, estudio.color(), &estudio.salida, true).unwrap();
    let pixeles = residente
        .capturar_estudio_bloque(
            Sombreadores { vertices: &shaders[0], fragmentos: &shaders[1] },
            (192, 192),
            [0.0, 0.0, 0.0, 1.0],
            &bloque,
            CapaInterfaz {
                shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                vertices: &[],
                area: [0, 0, 192, 192],
                atlas: None,
            },
        )
        .unwrap();
    assert_eq!(residente.actividad().geometria_subida, 0);
    pixeles
}

#[test]
fn reapertura_reconstruye_pixeles_identicos_y_responde_a_ediciones() {
    let gpu = Calculo::nuevo(&Opciones { muestras: 2, ..Opciones::default() }).unwrap();
    let mut estudio = Estudio::default();
    estudio.configurar_capacidad(33);
    estudio.rejilla.lado = 29;
    estudio.onda.amplitud = 0.6;
    estudio.relieve.factor = 1.7;
    estudio.salida.material = 2;
    estudio.salida.exposicion = 2.0;
    estudio.salida.muestras = 2;
    estudio.pausado = true;
    let original = Proyecto::capturar(&estudio, &nudo::Camara::default(), 0.6);
    let a = capturar(&gpu, &original);
    let reabierto = Proyecto::decodificar(&original.codificar().unwrap()).unwrap();
    let b = capturar(&gpu, &reabierto);
    assert!(a.as_chunks::<4>().0.iter().filter(|p| p[0] > 0 || p[1] > 0 || p[2] > 0).count() > 1000);
    assert_eq!(a, b, "mismo archivo, misma imagen; recursos GPU recreados");
    estudio.relieve.factor = 0.2;
    let editado = Proyecto::capturar(&estudio, &nudo::Camara::default(), 0.6);
    let c = capturar(&gpu, &editado);
    assert!(b.as_chunks::<4>().0.iter().zip(c.as_chunks::<4>().0).filter(|(b, c)| b != c).count() > 100);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
