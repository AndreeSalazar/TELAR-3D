//! El selector real cambia recursos gráficos y conserva el plan compute residente.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    reason = "verificación explícita y coordenadas de ventana pequeñas"
)]
#[path = "../src/arranque.rs"]
mod arranque;
#[path = "../src/calidad.rs"]
mod calidad;
#[path = "../src/controles.rs"]
mod controles;
#[path = "../src/grafo.rs"]
mod grafo;
#[path = "../src/grafo_compilar.rs"]
mod grafo_compilar;
#[allow(dead_code, reason = "comparte codec con la CLI; sus argumentos se prueban en proyecto.rs")]
#[path = "../src/proyecto.rs"]
mod proyecto;
#[path = "../src/sesion.rs"]
mod sesion;
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
    control.restaurar_interaccion();
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
    comprobar_sesion(&mut control, &mut gpu);
    comprobar_red(&mut control, &mut gpu);
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

fn comprobar_red(control: &mut controles::Controles, gpu: &mut Gpu) {
    let ruta = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/red-visor-{}.tlar", std::process::id()));
    let legado = ruta.with_file_name(format!("legado-visor-{}.tlar", std::process::id()));
    sesion::guardar(&legado, control).unwrap();
    let d = control.estudio.distribucion(gpu.imagenes().1);
    let r = d.accion_red(0);
    control.evento(Evento::Raton { x: (r.x + 5.0) as i32, y: (r.y + 5.0) as i32 }, d);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, d);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, d);
    assert_eq!(control.estudio.cantidad_nodos(), 5);
    assert!(grafo::sincronizar(gpu, control).unwrap());
    assert_eq!(gpu.actividad_procedural().pipelines, 6);
    let campo = d.valor(0);
    control.evento(Evento::Raton { x: (campo.x + 4.0) as i32, y: (campo.y + 4.0) as i32 }, d);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, d);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, d);
    for c in "0.31".chars() {
        control.evento(Evento::Texto(c), d);
    }
    control.evento(Evento::Tecla(telar_ventana::Tecla::Otra(13)), d);
    assert!(grafo::sincronizar(gpu, control).unwrap());
    assert!(!grafo::sincronizar(gpu, control).unwrap(), "no recompila en cuadros estables");
    sesion::guardar(&ruta, control).unwrap();
    let red_guardada = control.estudio.guardar_red();
    control.estudio.habilitar_nodo(4, false).unwrap();
    assert!(grafo::sincronizar(gpu, control).unwrap());
    for archivo in [&ruta, &legado] {
        control.estudio.solicitar_proyecto(telar_ui::AccionProyecto::Abrir);
        sesion::procesar(archivo, control, gpu);
        assert!(grafo::sincronizar(gpu, control).unwrap());
        if archivo == &ruta {
            assert_eq!(control.estudio.guardar_red(), red_guardada);
            assert_eq!(gpu.actividad_procedural().pipelines, 6);
        } else {
            assert!(!control.estudio.red_activa());
            assert_eq!(gpu.actividad_procedural().pipelines, 8);
        }
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
                    ParametrosProcedurales { lado: 19, fase: 0.6, amplitud: 0.35 }
                )
                .unwrap()
            );
        }
        assert_eq!(gpu.actividad_procedural().geometria_subida, 0);
        std::fs::remove_file(archivo).unwrap();
    }
}

fn comprobar_sesion(control: &mut controles::Controles, gpu: &mut Gpu) {
    use telar_ui::AccionProyecto;
    let ruta = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/sesion-gpu-{}.tlar", std::process::id()));
    control.estudio.salida.muestras = 2;
    control.estudio.rejilla.lado = 19;
    control.estudio.onda.amplitud = 0.7;
    control.estudio.relieve.factor = 2.2;
    control.estudio.pausado = true;
    control.fase = 0.6;
    let original = proyecto::Proyecto::capturar(&control.estudio, &control.camara, control.fase).codificar().unwrap();
    // Botones de cabecera por la misma ruta de entrada que usa la ventana.
    let diseno = control.estudio.distribucion(gpu.imagenes().1);
    for indice in [0, 1] {
        let rect = diseno.proyecto(indice);
        control.evento(Evento::Raton { x: (rect.x + 10.0) as i32, y: (rect.y + 10.0) as i32 }, diseno);
        control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
        control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, diseno);
        sesion::procesar(&ruta, control, gpu);
        if indice == 0 {
            control.estudio.onda.amplitud = 0.1;
            control.estudio.salida.muestras = 4;
            control.fase = 3.0;
            control.camara.orbitar(15.0, 10.0);
        }
    }
    let reabierto = proyecto::Proyecto::capturar(&control.estudio, &control.camara, control.fase).codificar().unwrap();
    assert_eq!(reabierto, original);
    assert_eq!(control.estudio.salida.muestras_reales, gpu.muestras().unwrap());
    let comprobar_error = |control: &mut controles::Controles, gpu: &mut Gpu| {
        control.estudio.solicitar_proyecto(AccionProyecto::Abrir);
        sesion::procesar(&ruta, control, gpu);
        assert_eq!(
            proyecto::Proyecto::capturar(&control.estudio, &control.camara, control.fase).codificar().unwrap(),
            original
        );
    };
    let mut otra = proyecto::leer(&ruta).unwrap();
    otra.estudio[1] = 64;
    proyecto::guardar(&ruta, &otra).unwrap();
    comprobar_error(control, gpu);
    std::fs::write(&ruta, b"corrupto").unwrap();
    comprobar_error(control, gpu);
    std::fs::remove_file(ruta).unwrap();
}
