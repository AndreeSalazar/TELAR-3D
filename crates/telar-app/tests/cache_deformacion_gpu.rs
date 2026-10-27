//! La GPU invalida u omite deformación sin consultar resultados desde el host.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
#[path = "../src/arranque.rs"]
mod arranque;
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural,
};
use telar_kernel::cache;

#[test]
fn cache_exacta_reutiliza_e_invalida_toda_la_cadena_de_puntos() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let relieve = hilo::ejemplos::relieve(1.3).unwrap().compilar();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural { nombre: "relieve", codigo: &relieve, dominio: DominioProcedural::Puntos });
    let cabecera = arranque::crear(gpu.perfil(), 33, etapas.len()).unwrap();
    let mut residente =
        gpu.procedural_con_plan(33, PlanProcedural::nuevo(&etapas).unwrap().con_arranque(&cabecera).unwrap()).unwrap();
    let sin_cache = telar_kernel::compilar_con_arranque();
    let mut referencia_etapas = etapas.clone();
    referencia_etapas[0].codigo = &sin_cache;
    let mut referencia = gpu
        .procedural_con_plan(33, PlanProcedural::nuevo(&referencia_etapas).unwrap().con_arranque(&cabecera).unwrap())
        .unwrap();
    let casos = [
        (33, 0.6, 0.35, 1, 0),
        (33, 0.6, 0.35, 1, 1),
        (33, 0.7, 0.35, 2, 1),
        (33, 0.7, 0.5, 3, 1),
        (17, 0.7, 0.5, 4, 1),
        (17, 0.7, 0.5, 4, 2),
        (0, 0.8, 0.5, 4, 2),
        (u32::MAX, 0.8, 0.5, 4, 2),
        (17, 0.7, 0.5, 4, 3),
        (33, 0.7, 0.5, 5, 3),
    ];
    let mut anterior = None;
    for (lado, fase, amplitud, calculadas, omitidas) in casos {
        residente.paso(ParametrosProcedurales { lado, fase, amplitud }).unwrap();
        referencia.paso(ParametrosProcedurales { lado, fase, amplitud }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        let esperados = referencia.inspeccionar().unwrap();
        assert_eq!(datos.indices, esperados.indices);
        assert!(
            datos
                .vertices
                .iter()
                .map(|v| v.map(f32::to_bits))
                .eq(esperados.vertices.iter().map(|v| v.map(f32::to_bits)))
        );
        assert_eq!(datos.control[cache::RECALCULADAS as usize], calculadas);
        assert_eq!(datos.control[cache::OMITIDAS as usize], omitidas);
        let bits: Vec<_> = datos.vertices.iter().map(|v| v.map(f32::to_bits)).collect();
        if datos.control[0] == 0 && lado == 33 && fase.to_bits() == 0.6f32.to_bits() {
            assert_eq!(Some(&bits), anterior.as_ref());
        }
        anterior = Some(bits);
    }
    assert_eq!(residente.actividad().geometria_subida, 0);
    assert_eq!(residente.actividad().grabaciones, 1);
    drop(residente);
    drop(referencia);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
