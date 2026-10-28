//! Reutilización y cancelación por lotes contra el kernel hash de referencia.
#![allow(clippy::unwrap_used, reason = "fallos explícitos en pruebas Vulkan")]
use telar_gpu::{Calculo, Opciones, PlanLotes};
#[test]
fn lotes_reutilizan_salida_y_preservan_indices_globales_y_cancelacion() {
    let gpu = Calculo::nuevo(&Opciones::default()).unwrap();
    let codigo = hilo_spirv::ejemplos::llenar_con_hash(64);
    let plan = PlanLotes { elementos: 1003, invocaciones: 1003, grupo: 64, grupos_por_lote: 3 };
    let mut progreso = Vec::new();
    let datos = gpu
        .ejecutar_lotes(&codigo, plan, |p| {
            progreso.push(p.completadas);
            true
        })
        .unwrap()
        .unwrap();
    assert_eq!(progreso, vec![0, 192, 384, 576, 768, 960, 1003]);
    assert_eq!(datos, (0..1003).map(hilo_spirv::ejemplos::hash_pcg).collect::<Vec<_>>());
    for corte in [0, 192, 1003] {
        let salida = gpu.ejecutar_lotes(&codigo, plan, |p| p.completadas < corte).unwrap();
        assert!(salida.is_none());
    }
    assert!(gpu.ejecutar_lotes(&[], PlanLotes { grupo: 0, ..plan }, |_| true).is_err());
    assert!(gpu.ejecutar_lotes(&codigo, PlanLotes { invocaciones: u32::MAX, ..plan }, |_| true).is_err());
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
