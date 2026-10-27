//! Un encabezado rechazado no habilita geometría ni productores de servicios.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
#[path = "../src/arranque.rs"]
mod arranque;
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural,
};
use telar_kernel::abi;

#[test]
fn cabecera_valida_conserva_geometria_bit_a_bit_frente_al_coordinador_anterior() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let generar = hilo_spirv::rejilla::generar();
    let deformar = hilo_spirv::rejilla::deformar();
    let viejo = telar_kernel::compilar();
    let nuevo = telar_kernel::compilar_con_arranque();
    let base = |preparar| KernelsProcedurales { preparar, generar: &generar, deformar: &deformar }.etapas();
    let etapas_viejas = base(&viejo);
    let etapas_nuevas = base(&nuevo);
    let header = arranque::crear(gpu.perfil(), 17, 3).unwrap();
    let mut referencia = gpu.procedural_con_plan(17, PlanProcedural::nuevo(&etapas_viejas).unwrap()).unwrap();
    let mut protegida = gpu
        .procedural_con_plan(17, PlanProcedural::nuevo(&etapas_nuevas).unwrap().con_arranque(&header).unwrap())
        .unwrap();
    for lado in [17, 9, 9, 0, 18, 17] {
        let parametros = ParametrosProcedurales { lado, fase: 0.4, amplitud: 0.3 };
        referencia.paso(parametros).unwrap();
        protegida.paso(parametros).unwrap();
        let a = referencia.inspeccionar().unwrap();
        let b = protegida.inspeccionar().unwrap();
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.vertices.len(), b.vertices.len());
        assert!(a.vertices.iter().map(|v| v.map(f32::to_bits)).eq(b.vertices.iter().map(|v| v.map(f32::to_bits))));
        assert_eq!(&a.control[..25], &b.control[..25]);
    }
    drop(referencia);
    drop(protegida);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn guardian_gpu_rechaza_cabeceras_y_solo_admite_el_contrato_completo() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    assert!(gpu.validacion());
    let perfil = gpu.perfil();
    assert!(perfil.grupos_x > 0 && perfil.almacenes >= 5 && perfil.memoria_local > 0);
    let shaders =
        [telar_kernel::compilar_con_arranque(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let productor = telar_kernel::solicitar_en(1, 0); // Se activaría tras un fallo si el dominio Servicio fuera directo.
    let mut etapas =
        KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural {
        nombre: "canario-servicio",
        codigo: &productor,
        dominio: DominioProcedural::Servicio,
    });
    let cabecera = arranque::crear(perfil, 9, etapas.len()).unwrap();
    let mut casos = vec![
        ("válida", cabecera.to_vec(), 0),
        ("truncada", cabecera[..31].to_vec(), 1),
        ("una palabra", cabecera[..1].to_vec(), 1),
    ];
    for (indice, valor, error) in [
        (0, 0, 2),
        (1, 999, 2),
        (2, 31, 2),
        (16, u32::MAX, 3),
        (17, 3, 3),
        (18, u32::MAX, 3),
        (19, 0, 3),
        (20, 33, 3),
        (13, 0, 3),
        (3, 0, 4),
        (4, 7, 4),
        (8, 0, 4),
        (10, 63, 4),
        (11, 63, 4),
        (21, 128, 4),
        (25, 4, 4),
        (9, 0, 5),
        (23, 0, 5),
        (22, 1, 5),
        (22, 33, 5),
        (24, 0, 5),
        (27, 1, 2),
    ] {
        let mut invalida = cabecera;
        invalida[indice] = valor;
        casos.push(("campo inválido", invalida.to_vec(), error));
    }
    let plan = PlanProcedural::nuevo(&etapas).unwrap();
    assert!(plan.con_arranque(&[]).is_err());
    assert!(plan.con_arranque(&[0; 257]).is_err());
    for (nombre, header, esperado) in casos {
        let mut residente = gpu.procedural_con_plan(9, plan.con_arranque(&header).unwrap()).unwrap();
        residente.paso(ParametrosProcedurales { lado: 9, fase: 0.4, amplitud: 0.3 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        assert_eq!(datos.control[abi::ERROR_ARRANQUE as usize], esperado, "{nombre}: {header:?}");
        assert_eq!(datos.control[abi::LISTO as usize], u32::from(esperado == 0));
        assert_eq!(datos.control[abi::SOLICITUD as usize], 0, "canario del servicio no debe escribir");
        if esperado == 0 {
            assert_eq!(datos.vertices.len(), 81);
            assert_eq!(datos.indices.len(), 384);
            assert_eq!(datos.control[abi::FABRICANTE as usize], perfil.fabricante);
            assert_eq!(datos.control[abi::GENERACION_PLAN as usize], 1);
        } else {
            assert!(datos.vertices.is_empty() && datos.indices.is_empty());
            for indice in [0, 3, 9, 12, 15, 27] {
                assert_eq!(datos.control[indice], 0);
            }
        }
        assert_eq!(residente.actividad().buffers, 5);
        assert_eq!(residente.actividad().geometria_subida, 0);
    }
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn arranque_admitido_conserva_suspension_y_reanudacion_en_gpu() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo =
        [telar_kernel::compilar_con_arranque(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let suspender = telar_kernel::solicitar_en(2, 1);
    let reanudar = telar_kernel::solicitar_en(3, 3);
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "suspender", codigo: &suspender, dominio: DominioProcedural::Servicio });
    etapas.push(EtapaProcedural { nombre: "reanudar", codigo: &reanudar, dominio: DominioProcedural::Servicio });
    let header = arranque::crear(gpu.perfil(), 9, etapas.len()).unwrap();
    let plan = PlanProcedural::nuevo(&etapas).unwrap().con_arranque(&header).unwrap();
    let mut residente = gpu.procedural_con_plan(9, plan).unwrap();
    for (cantidad, pendiente) in [(81, 2), (0, 0), (0, 3), (81, 0)] {
        residente.paso(ParametrosProcedurales { lado: 9, fase: 0.4, amplitud: 0.3 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        assert_eq!(datos.vertices.len(), cantidad);
        assert_eq!(datos.control[16], pendiente);
        assert_eq!(&datos.control[27..30], &[1, 1, 1], "los servicios siguen habilitados durante suspensión");
        assert_eq!(datos.control[25], 0);
    }
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
