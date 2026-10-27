//! Solicitudes nacidas en GPU, atendidas sin que el host decida sus consecuencias.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, KernelsProcedurales, Opciones, ParametrosProcedurales, PlanProcedural,
};
use telar_kernel::abi::{self, Servicio};

fn productor_por_cantidad() -> Vec<u32> {
    use hilo_spirv::{
        Modulo,
        op::{self, Clase},
    };
    let mut modulo = Modulo::nuevo();
    let entero = modulo.tipo_u32();
    let booleano = modulo.tipo_bool();
    let array = modulo.tipo_array_dinamico(entero, 4);
    let bloque = modulo.tipo_bloque(&[array], &[0]);
    let puntero_bloque = modulo.tipo_puntero(Clase::Almacen, bloque);
    let control = modulo.variable(puntero_bloque, Clase::Almacen);
    modulo.decorar(control, op::DECORACION_CONJUNTO, &[0]);
    modulo.decorar(control, op::DECORACION_ENLACE, &[3]);
    let puntero = modulo.tipo_puntero(Clase::Almacen, entero);
    let buzon = telar_kernel::Buzon::nuevo(&mut modulo, control);
    let [cero, puntos, umbral, servicio] =
        [0, 9, 50, Servicio::RehacerTopologia as u32].map(|v| modulo.constante_u32(v));
    let vacio = modulo.tipo_void();
    let firma = modulo.tipo_funcion(vacio, &[]);
    let mut funcion = modulo.funcion(vacio, firma);
    let entrada = funcion.etiqueta();
    funcion.bloque(entrada);
    let direccion = funcion.acceso(puntero, control, &[cero, puntos]);
    let cantidad = funcion.cargar(entero, direccion);
    let suficiente = funcion.binaria(op::U_LESS_THAN, booleano, umbral, cantidad);
    let peticion = funcion.elegir(entero, suficiente, servicio, cero);
    // GPU: una invocación solicita según el recuento residente; ninguna lectura host.
    buzon.solicitar(&mut funcion, peticion);
    funcion.volver();
    let main = funcion.terminar();
    modulo.entrada_calculo(main, "main", &[], [1, 1, 1]);
    modulo.ensamblar()
}

#[test]
fn un_trabajador_pide_servicio_segun_datos_residentes_y_no_un_calendario_host() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let productor = productor_por_cantidad();
    let mut etapas =
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas().to_vec();
    etapas.push(EtapaProcedural {
        nombre: "servicio-por-datos",
        codigo: &productor,
        dominio: DominioProcedural::Servicio,
    });
    let mut residente = gpu.procedural_con_plan(9, PlanProcedural::nuevo(&etapas).unwrap()).unwrap();
    for (lado, generaciones, peticion) in [(5, 1, 0), (9, 2, 1), (9, 3, 1), (5, 4, 0), (5, 4, 0)] {
        residente.paso(ParametrosProcedurales { lado, fase: 0.4, amplitud: 0.3 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        assert_eq!(datos.control[15], generaciones);
        assert_eq!(datos.control[abi::SOLICITUD as usize], peticion);
        assert_eq!(datos.vertices.len(), (lado * lado) as usize);
    }
    assert_eq!(residente.actividad().geometria_subida, 0);
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn gpu_solicita_rehacer_suspender_reanudar_y_rechaza_servicios_invalidos() {
    assert_eq!(abi::PALABRAS, 32, "el prefijo ABI v1 permanece intacto");
    assert_eq!(telar_kernel::cache::PALABRAS, telar_gpu::PALABRAS_CONTROL_PROCEDURAL, "extensión de caché explícita");
    let gpu = Calculo::nuevo(&Opciones::default()).expect("Vulkan real");
    assert!(gpu.validacion());
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let base = KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] }.etapas();
    let pedidos = [
        ("rehacer", Servicio::RehacerTopologia as u32, 2),
        ("suspender", Servicio::Suspender as u32, 4),
        ("reanudar", Servicio::Reanudar as u32, 6),
        ("desconocido", 999, 8),
        ("buzon-ocupado", Servicio::RehacerTopologia as u32, 8),
    ];
    let productores: Vec<_> =
        pedidos.iter().map(|(_, codigo, vuelta)| telar_kernel::solicitar_en(*codigo, *vuelta)).collect();
    let mut etapas = base.to_vec();
    etapas.extend(pedidos.iter().zip(&productores).map(|((nombre, _, _), codigo)| EtapaProcedural {
        nombre,
        codigo,
        dominio: DominioProcedural::Servicio,
    }));
    let mut residente = gpu.procedural_con_plan(9, PlanProcedural::nuevo(&etapas).unwrap()).unwrap();
    // Expectativas declaradas de antemano; ninguna descarga decide el paso siguiente.
    let casos = [
        (1, 0, 0, 1, 0, 0),
        (2, 1, 0, 1, 0, 0),
        (3, 0, 0, 2, 1, 1),
        (4, 2, 0, 2, 1, 1),
        (5, 0, 1, 2, 2, 2),
        (6, 3, 1, 2, 2, 2),
        (7, 0, 0, 2, 3, 3),
        (8, 999, 0, 2, 3, 3),
        (9, 0, 0, 2, 3, 999),
    ];
    for (vuelta, pendiente, suspendido, generaciones, atendidas, ultima) in casos {
        residente.paso(ParametrosProcedurales { lado: 9, fase: 0.4, amplitud: 0.3 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        let control = datos.control;
        let leer = |indice: u32| control[indice as usize];
        assert_eq!(leer(abi::VERSION_CONTROL), abi::VERSION);
        assert_eq!(leer(abi::VUELTA), vuelta);
        assert_eq!(leer(abi::SOLICITUD), pendiente, "vuelta {vuelta}");
        assert_eq!(leer(abi::SUSPENDIDO), suspendido);
        assert_eq!(control[15], generaciones, "la GPU fuerza regeneración sin cambio de lado");
        assert_eq!(leer(abi::ATENDIDAS), atendidas);
        assert_eq!(leer(abi::ULTIMA), ultima);
        assert_eq!(leer(abi::OCUPADAS), u32::from(vuelta >= 8));
        assert_eq!(leer(abi::RESULTADO), u32::from(vuelta >= 9));
        assert_eq!(leer(abi::DESCONOCIDAS), u32::from(vuelta >= 9));
        let activo = suspendido == 0;
        assert_eq!(control[0], if activo { 2 } else { 0 });
        assert_eq!(control[3], if activo { 384 } else { 0 });
        assert_eq!(datos.vertices.len(), if activo { 81 } else { 0 });
        if activo {
            assert!(datos.vertices.iter().any(|v| v[1].abs() > 0.01), "reanudar conserva una superficie operativa");
            assert!(datos.indices.iter().all(|i| *i < 81));
        }
    }
    for lado in [u32::MAX, 1, 9] {
        residente.paso(ParametrosProcedurales { lado, fase: 0.4, amplitud: 0.3 }).unwrap();
        let datos = residente.inspeccionar().unwrap();
        assert_eq!(datos.control[10], u32::from(lado > 9));
        assert_eq!(datos.control[15], 2, "vacío/error no destruyen la caché");
        assert_eq!(datos.vertices.len(), if lado == 9 { 81 } else { 0 });
    }
    let actividad = residente.actividad();
    assert_eq!((actividad.buffers, actividad.grabaciones, actividad.mapeos), (4, 1, 1));
    assert_eq!(actividad.parametros, 12 * 16);
    assert_eq!(actividad.geometria_subida, 0);
    drop(residente);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
