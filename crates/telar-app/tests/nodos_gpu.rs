//! Los valores editables llegan al mismo plan residente y a sus claves GPU.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
#[path = "../src/arranque.rs"]
mod arranque;
#[path = "../src/render.rs"]
mod render;
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosNodos,
    ParametrosProcedurales, PlanProcedural,
};

#[test]
fn relieve_vivo_corrige_alturas_normales_y_cache_sin_recompilar() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let programa = hilo::ejemplos::relieve_dinamico().unwrap();
    let relieve = programa.compilar();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "relieve", codigo: &relieve, dominio: DominioProcedural::Puntos });
    let cabecera = arranque::crear(gpu.perfil(), 33, etapas.len()).unwrap();
    let mut residente =
        gpu.procedural_con_plan(33, PlanProcedural::nuevo(&etapas).unwrap().con_arranque(&cabecera).unwrap()).unwrap();
    let mut estudio = telar_ui::Estudio::default();
    estudio.configurar_capacidad(33);
    for (factor, recalculadas, omitidas) in [(0.0, 1, 0), (1.3, 2, 0), (1.3, 2, 1), (2.0, 3, 1), (0.0, 4, 1)] {
        estudio.relieve.factor = factor;
        let parametros = ParametrosNodos {
            superficie: ParametrosProcedurales {
                lado: estudio.rejilla.lado,
                fase: 0.4,
                amplitud: estudio.onda.amplitud,
            },
            valores: [factor, 0.0, 0.0, 0.0],
        };
        residente.paso_nodos(parametros).unwrap();
        let datos = residente.inspeccionar().unwrap();
        assert_eq!(datos.control[36], recalculadas);
        assert_eq!(datos.control[37], omitidas);
        for vertice in &datos.vertices {
            let (x, z) = (vertice[0], vertice[2]);
            let angulo = 2.0 * x + 0.4;
            let altura = 0.35 * angulo.sin() * (2.0 * z).cos();
            let normal = nudo::mate::normalizar([
                -0.7 * angulo.cos() * (2.0 * z).cos(),
                1.0,
                0.7 * angulo.sin() * (2.0 * z).sin(),
            ]);
            let esperado = programa
                .interpretar_con_parametros(
                    hilo::Entrada { posicion: [x, altura, z], normal, fase: 0.4, amplitud: 0.35 },
                    parametros.valores,
                )
                .unwrap();
            assert!((vertice[1] - esperado.posicion[1]).abs() < 0.0001);
            for i in 0..3 {
                assert!((vertice[3 + i] - esperado.normal[i]).abs() < 0.0001);
            }
        }
    }
    let actividad = residente.actividad();
    assert_eq!(
        (actividad.buffers, actividad.pipelines, actividad.grabaciones, actividad.geometria_subida),
        (5, 4, 1, 0)
    );
    let invalido = ParametrosNodos {
        superficie: ParametrosProcedurales { lado: 33, fase: 0.4, amplitud: 0.35 },
        valores: [f32::NAN, 0.0, 0.0, 0.0],
    };
    assert!(residente.paso_nodos(invalido).is_err());
    assert_eq!(residente.actividad(), actividad);
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn ajustes_de_salida_se_empaquetan_sin_modificar_geometria() {
    let mut estudio = telar_ui::Estudio::default();
    let camara = nudo::Camara::default();
    estudio.salida.exposicion = 2.5;
    estudio.salida.rugosidad = 0.7;
    let bloque = render::bloque(&camara, 1.0, estudio.color(), &estudio.salida, true).unwrap();
    assert_eq!(bloque[19].to_bits(), 2.5f32.to_bits());
    assert_eq!(bloque[27].to_bits(), 0.7f32.to_bits());
    assert_eq!(estudio.relieve.factor.to_bits(), 1.3f32.to_bits());
}
