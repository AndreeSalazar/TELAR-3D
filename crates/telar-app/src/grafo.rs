//! Preparación gráfica al editar; evaluación por vértice exclusivamente en GPU.
use crate::controles::Controles;
use telar_gpu::{DominioProcedural, EtapaProcedural, Gpu, PlanProcedural};

pub(crate) fn preparar(gpu: &mut Gpu, estudio: &telar_ui::Estudio) -> Result<(), String> {
    let codigo = if estudio.red_activa() {
        vec![
            telar_kernel::compilar_con_cache(),
            hilo_spirv::rejilla::generar(),
            crate::grafo_compilar::compilar(estudio)?.compilar(),
        ]
    } else {
        vec![
            telar_kernel::compilar_con_cache(),
            hilo_spirv::rejilla::generar(),
            hilo_spirv::rejilla::deformar(),
            hilo::ejemplos::relieve_dinamico().map_err(|e| e.to_string())?.compilar(),
        ]
    };
    let mut etapas = vec![
        EtapaProcedural { nombre: "coordinador", codigo: &codigo[0], dominio: DominioProcedural::Preparar },
        EtapaProcedural { nombre: "rejilla", codigo: &codigo[1], dominio: DominioProcedural::Generar },
        EtapaProcedural { nombre: "grafo_hilo", codigo: &codigo[2], dominio: DominioProcedural::Puntos },
    ];
    if let Some(relieve) = codigo.get(3) {
        etapas.push(EtapaProcedural { nombre: "relieve", codigo: relieve, dominio: DominioProcedural::Puntos });
    }
    let cabecera = crate::arranque::crear(gpu.perfil(), estudio.rejilla.capacidad, etapas.len())?;
    gpu.preparar_plan_procedural(estudio.rejilla.capacidad, PlanProcedural::nuevo(&etapas)?.con_arranque(&cabecera)?)?;
    println!(
        "telar: red compilada · {} nodos · {} operaciones alcanzables · cálculo GPU",
        estudio.cantidad_nodos(),
        estudio.operaciones_red()?.len()
    );
    Ok(())
}

pub(crate) fn sincronizar(gpu: &mut Gpu, controles: &mut Controles) -> Result<bool, String> {
    if !controles.estudio.tomar_cambio_red() {
        return Ok(false);
    }
    preparar(gpu, &controles.estudio)?;
    controles.sucio = true;
    Ok(true)
}
