//! Planes variables: validación previa y composición GPU contra matemática CPU.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_precision_loss,
    reason = "tests pequeños, con asserts y referencias numéricas independientes"
)]

use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural,
    avisos_de_validacion,
};

fn etapa<'a>(nombre: &'a str, codigo: &'a [u32]) -> EtapaProcedural<'a> {
    EtapaProcedural { nombre, codigo, dominio: DominioProcedural::Puntos }
}

#[test]
fn rechaza_planes_incoherentes_antes_de_crear_recursos() {
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let base = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] }.etapas();
    assert!(PlanProcedural::nuevo(&[]).is_err());
    assert!(PlanProcedural::nuevo(&base[..1]).is_err());
    assert!(PlanProcedural::nuevo(&base[..2]).is_ok(), "generar sin deformar también es válido");
    let mut mal = base;
    mal.swap(0, 1);
    assert!(PlanProcedural::nuevo(&mal).err().unwrap().contains("etapa 0"));
    mal = base;
    mal[2].nombre = "generar";
    assert!(PlanProcedural::nuevo(&mal).err().unwrap().contains("único"));
    mal[2].nombre = "  ";
    assert!(PlanProcedural::nuevo(&mal).is_err());
    mal = base;
    mal[2].dominio = DominioProcedural::Preparar;
    assert!(PlanProcedural::nuevo(&mal).err().unwrap().contains("etapa 2"));
    mal = base;
    mal[2].codigo = &[0, 0, 0, 0, 0];
    assert!(PlanProcedural::nuevo(&mal).err().unwrap().contains("SPIR-V"));
    let nombres: Vec<String> = (0..31).map(|i| format!("operacion-{i}")).collect();
    let mut limite = base[..2].to_vec();
    for nombre in &nombres[..30] {
        limite.push(etapa(nombre, &shaders[2]));
    }
    assert_eq!(PlanProcedural::nuevo(&limite).unwrap().etapas().len(), 32);
    limite.push(etapa(&nombres[30], &shaders[2]));
    assert!(PlanProcedural::nuevo(&limite).is_err());
    assert!(hilo_spirv::rejilla::escalar_altura(f32::INFINITY).is_err());
    assert!(hilo_spirv::rejilla::escalar_altura(17.0).is_err());
}

#[test]
fn operaciones_componibles_conservan_datos_y_tiempos_por_etapa() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("esta prueba requiere Vulkan real");
    assert!(gpu.validacion());
    let shaders = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let base = KernelsProcedurales { preparar: &shaders[0], generar: &shaders[1], deformar: &shaders[2] }.etapas();
    let escalar = hilo_spirv::rejilla::escalar_altura(1.3).unwrap();
    let invertir = hilo_spirv::rejilla::escalar_altura(-0.5).unwrap();
    let aplanar = hilo_spirv::rejilla::escalar_altura(0.0).unwrap();
    let casos = [
        (vec![], 0.0),
        (vec![base[2]], 1.0),
        (vec![base[2], etapa("relieve", &escalar)], 1.3),
        (vec![base[2], etapa("relieve", &escalar), etapa("invertir", &invertir)], -0.65),
        (vec![base[2], etapa("aplanar", &aplanar)], 0.0),
        // La última onda reemplaza la altura escalada: prueba el orden real de dependencia.
        (vec![etapa("relieve", &escalar), base[2]], 1.0),
    ];
    for (operaciones, factor) in casos {
        let mut etapas = base[..2].to_vec();
        etapas.extend(operaciones);
        let plan = PlanProcedural::nuevo(&etapas).unwrap();
        let mut residente = gpu.procedural_con_plan(9, plan).unwrap();
        for cuadro in 0..12 {
            let fase = cuadro as f32 * 0.11;
            residente.paso(ParametrosProcedurales { lado: 9, fase, amplitud: 0.4 }).unwrap();
            let datos = residente.inspeccionar().unwrap();
            let nombres: Vec<&str> = datos.etapas.iter().map(String::as_str).collect();
            assert_eq!(nombres, etapas.iter().map(|e| e.nombre).collect::<Vec<_>>());
            if gpu.mide_tiempos() {
                let tiempos = datos.nanosegundos.as_ref().unwrap();
                assert_eq!(tiempos.len(), etapas.len());
                assert!(tiempos.iter().all(|t| t.is_finite() && *t >= 0.0));
            }
            assert_eq!(datos.control[15], 1, "las operaciones no invalidan x/z ni los índices");
            for vertice in &datos.vertices {
                let sx = (2.0 * vertice[0] + fase).sin();
                let cx = (2.0 * vertice[0] + fase).cos();
                let sz = (2.0 * vertice[2]).sin();
                let cz = (2.0 * vertice[2]).cos();
                let normal = [-0.8 * factor * cx * cz, 1.0, 0.8 * factor * sx * sz];
                let longitud = normal.iter().map(|n| n * n).sum::<f32>().sqrt();
                let esperado =
                    [0.4 * factor * sx * cz, normal[0] / longitud, normal[1] / longitud, normal[2] / longitud];
                for (valor, referencia) in [vertice[1], vertice[3], vertice[4], vertice[5]].iter().zip(esperado) {
                    assert!((valor - referencia).abs() < 0.00002, "{valor} != {referencia}");
                }
            }
        }
        residente.paso(ParametrosProcedurales { lado: 0, fase: 0.0, amplitud: 0.4 }).unwrap();
        let vacio = residente.inspeccionar().unwrap();
        assert!(vacio.vertices.is_empty() && vacio.indices.is_empty());
        let actividad = residente.actividad();
        assert_eq!(actividad.pipelines, etapas.len() as u64);
        assert_eq!((actividad.buffers, actividad.grabaciones, actividad.mapeos), (4, 1, 1));
        assert_eq!(actividad.geometria_subida, 0);
    }
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}
