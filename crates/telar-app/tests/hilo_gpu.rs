//! El mismo IR se interpreta en CPU y se ejecuta como SPIR-V sobre datos residentes.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "tests pequeños con asserts y tolerancias declaradas"
)]

use hilo::{Entrada, Id, Operacion as Op, Programa, Salida};
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural,
    avisos_de_validacion,
};

fn operaciones_compuestas() -> Programa {
    Programa::nuevo(
        vec![
            Op::Posicion,
            Op::Normal,
            Op::Fase,
            Op::Amplitud,
            Op::Escalar(2.0),
            Op::Vector([0.25, 0.5, 0.75]),
            Op::Multiplicar(Id(0), Id(4)),
            Op::Multiplicar(Id(4), Id(1)),
            Op::Multiplicar(Id(6), Id(5)),
            Op::Sumar(Id(7), Id(8)),
            Op::Restar(Id(9), Id(5)),
            Op::ProductoEscalar(Id(0), Id(5)),
            Op::Sumar(Id(2), Id(3)),
            Op::Multiplicar(Id(11), Id(12)),
            Op::Restar(Id(13), Id(4)),
            Op::Extraer(Id(10), 1),
            Op::Componer([Id(14), Id(15), Id(4)]),
            Op::Normalizar(Id(16)),
        ],
        Salida { altura: Id(14), normal: Id(17) },
    )
    .unwrap()
}

#[test]
fn kernels_del_ir_coinciden_con_su_interprete_y_reutilizan_la_cache() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("esta prueba requiere Vulkan real");
    assert!(gpu.validacion());
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let kernels = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] };
    let cero = Programa::nuevo(
        vec![Op::Escalar(0.0), Op::Vector([0.0; 3]), Op::Normalizar(Id(1))],
        Salida { altura: Id(0), normal: Id(2) },
    )
    .unwrap();
    let grande = Programa::nuevo(
        vec![Op::Escalar(0.0), Op::Vector([f32::MAX, 0.0, 0.0]), Op::Normalizar(Id(1))],
        Salida { altura: Id(0), normal: Id(2) },
    )
    .unwrap();
    let programas = [
        hilo::ejemplos::relieve(1.3).unwrap(),
        hilo::ejemplos::relieve(-0.5).unwrap(),
        hilo::ejemplos::relieve(0.0).unwrap(),
        operaciones_compuestas(),
        cero,
        grande,
    ];
    for programa in programas {
        let codigo = programa.compilar();
        let mut etapas = kernels.etapas().to_vec();
        etapas.push(EtapaProcedural { nombre: "hilo-ir", codigo: &codigo, dominio: DominioProcedural::Puntos });
        let mut referencia = gpu.procedural(9, kernels).unwrap();
        let mut compilado = gpu.procedural_con_plan(9, PlanProcedural::nuevo(&etapas).unwrap()).unwrap();
        for paso in 0..16 {
            let lado = [0, 1, 2, 3, 9, 9, 18, 9][paso % 8];
            let parametros = ParametrosProcedurales { lado, fase: paso as f32 * 0.13, amplitud: 0.37 };
            referencia.paso(parametros).unwrap();
            compilado.paso(parametros).unwrap();
            let antes = referencia.inspeccionar().unwrap();
            let despues = compilado.inspeccionar().unwrap();
            assert_eq!(antes.control, despues.control, "HILO no modifica los argumentos ni la caché");
            assert_eq!(antes.indices, despues.indices);
            assert_eq!(antes.vertices.len(), despues.vertices.len());
            for (entrada, actual) in antes.vertices.iter().zip(&despues.vertices) {
                let esperado = programa
                    .interpretar(Entrada {
                        posicion: [entrada[0], entrada[1], entrada[2]],
                        normal: [entrada[3], entrada[4], entrada[5]],
                        fase: parametros.fase,
                        amplitud: parametros.amplitud,
                    })
                    .unwrap();
                let valores = [
                    esperado.posicion[0],
                    esperado.posicion[1],
                    esperado.posicion[2],
                    esperado.normal[0],
                    esperado.normal[1],
                    esperado.normal[2],
                ];
                for (cpu, gpu) in valores.iter().zip(actual) {
                    assert!((cpu - gpu).abs() < 0.00002, "CPU {cpu} != GPU {gpu}");
                }
            }
        }
        let parametros = ParametrosProcedurales { lado: 9, fase: 0.75, amplitud: 0.4 };
        compilado.paso(parametros).unwrap();
        let primera = compilado.inspeccionar().unwrap();
        compilado.paso(parametros).unwrap();
        let segunda = compilado.inspeccionar().unwrap();
        let bits = |v: &Vec<[f32; 6]>| v.iter().map(|v| v.map(f32::to_bits)).collect::<Vec<_>>();
        assert_eq!(bits(&primera.vertices), bits(&segunda.vertices));
        assert_eq!(compilado.actividad().grabaciones, 1);
        assert_eq!(compilado.actividad().geometria_subida, 0);
    }
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}
