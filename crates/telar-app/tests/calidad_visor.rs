//! El selector real cambia recursos gráficos y conserva el plan compute residente.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    reason = "verificación explícita y coordenadas de ventana pequeñas"
)]
#[path = "../src/calidad.rs"]
mod calidad;
#[path = "../src/controles.rs"]
mod controles;
use telar_gpu::{Constantes, Gpu, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores, Superficie};
use telar_ventana::{Boton, Config, Evento, Ventana};

#[test]
fn selector_de_salida_cambia_msaa_en_vivo_con_redimensionado() {
    let mut ventana =
        Ventana::nueva(&Config { titulo: "Prueba calidad".to_owned(), ancho: 1280, alto: 720, visible: false })
            .unwrap();
    let manejadores = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones::default(),
        Superficie { instancia: manejadores.instancia, ventana: manejadores.ventana },
        ventana.tamano(),
    )
    .unwrap();
    let escena = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    gpu.preparar_visor(Sombreadores { vertices: &escena[0], fragmentos: &escena[1] }).unwrap();
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    gpu.preparar_procedural(
        33,
        KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] },
    )
    .unwrap();
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    gpu.preparar_interfaz(Sombreadores { vertices: &ui[0], fragmentos: &ui[1] }).unwrap();
    let inicial = gpu.actividad_procedural();
    let mut control = controles::Controles::nuevo(true);
    control.estudio.seleccionado = 3;
    control.estudio.salida.muestras_reales = gpu.muestras().unwrap();
    control.estudio.configurar_capacidad(33);
    let mut aplicadas = 4;
    for indice in [2, 1, 0, 1, 2, 2, 0] {
        let diseno = control.estudio.distribucion(gpu.imagenes().1);
        let boton = diseno.calidad(indice);
        control.evento(Evento::Raton { x: (boton.x + 12.0) as i32, y: (boton.y + 12.0) as i32 }, diseno);
        control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
        control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, diseno);
        control.avanzar(0.0);
        calidad::sincronizar(&mut gpu, &mut control, &mut aplicadas).unwrap();
        let solicitadas = [1, 2, 4][indice];
        assert_eq!(aplicadas, solicitadas);
        assert!(gpu.muestras().unwrap() <= solicitadas);
        assert_eq!(control.estudio.salida.muestras_reales, gpu.muestras().unwrap());
        let vertices = control.estudio.dibujar(diseno, 33 * 33);
        gpu.actualizar_interfaz(&vertices, diseno.area()).unwrap();
        let constantes = Constantes {
            matriz: control.camara.matriz(1.0),
            luz: [0.0, 1.0, 0.0],
            color: [1.0; 4],
            cielo: [0.1; 4],
            suelo: [0.0; 4],
        };
        for _ in 0..3 {
            assert!(
                gpu.dibujar_procedural(
                    [0.0, 0.0, 0.0, 1.0],
                    &constantes,
                    ParametrosProcedurales { lado: 33, fase: 0.4, amplitud: 0.35 }
                )
                .unwrap()
            );
        }
        ventana.cambiar_tamano(960, 640);
        for evento in ventana.eventos() {
            if let Evento::Tamano { ancho, alto } = evento {
                gpu.redimensionar(ancho, alto);
            }
        }
    }
    let antes = gpu.muestras().unwrap();
    assert!(gpu.configurar_muestras(3).is_err());
    assert_eq!(gpu.muestras().unwrap(), antes);
    let finalizado = gpu.actividad_procedural();
    assert_eq!(
        (finalizado.buffers, finalizado.pipelines, finalizado.grabaciones, finalizado.mapeos),
        (inicial.buffers, inicial.pipelines, inicial.grabaciones, inicial.mapeos)
    );
    assert_eq!(finalizado.geometria_subida, 0);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
