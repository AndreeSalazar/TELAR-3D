//! La escena conserva valores y posiciones, nunca interacción ni capacidades GPU.
#![allow(clippy::unwrap_used, clippy::float_cmp, reason = "comparación exacta de estados de prueba")]
use telar_ui::{AccionProyecto, Estudio};

#[test]
fn restaurar_conserva_parametros_posiciones_y_estilos_y_termina_arrastres() {
    let mut origen = Estudio::default();
    origen.configurar_capacidad(512);
    origen.rejilla.lado = 73;
    origen.onda.amplitud = 0.7;
    origen.onda.velocidad = 0.4;
    origen.relieve.factor = 2.5;
    origen.salida.exposicion = 3.0;
    origen.salida.rugosidad = 0.8;
    origen.salida.material = 2;
    origen.salida.muestras = 2;
    origen.seleccionado = 2;
    origen.pausado = true;
    let diseno = origen.distribucion((1280, 720));
    let rect = origen.nodo(diseno, 2);
    origen.pulsar(diseno, rect.x + 10.0, rect.y + 10.0);
    origen.mover_en(diseno, diseno.grafo.x + 140.0, diseno.grafo.y + 70.0);
    let estado = origen.guardar_estado();
    let mut destino = Estudio::default();
    destino.cargar_estilos(".visor { background-color: #123456; }").unwrap();
    let fondo = destino.fondo_visor();
    destino.salida.muestras_reales = 1;
    destino.restaurar_estado(&estado).unwrap();
    assert_eq!(destino.guardar_estado(), estado);
    assert_eq!(destino.fondo_visor(), fondo);
    assert_eq!(destino.salida.muestras_reales, 1, "depende del backend destino");
    assert!(!destino.mover_en(diseno, 400.0, 400.0), "no restaura arrastres");
    assert_eq!(destino.guardar_estado(), estado);
}

#[test]
fn datos_invalidos_no_modifican_el_estado_anterior() {
    let mut estudio = Estudio::default();
    let original = estudio.guardar_estado();
    for (indice, invalido) in [
        (0, 1),
        (0, 1025),
        (1, 0),
        (1, 1025),
        (2, f32::NAN.to_bits()),
        (3, f32::INFINITY.to_bits()),
        (4, (-1.0f32).to_bits()),
        (5, 0),
        (6, 0),
        (7, 3),
        (8, 256),
        (9, 4),
        (10, 2),
        (11, 2.0f32.to_bits()),
        (18, f32::NEG_INFINITY.to_bits()),
    ] {
        let mut malo = original;
        malo[indice] = invalido;
        assert!(estudio.restaurar_estado(&malo).is_err(), "palabra {indice}");
        assert_eq!(estudio.guardar_estado(), original);
    }
}

#[test]
fn acciones_de_archivo_son_explicitas_y_se_consumen_una_vez() {
    let mut estudio = Estudio::default();
    let estado = estudio.guardar_estado();
    let diseno = estudio.distribucion((800, 600));
    for (i, accion) in [AccionProyecto::Guardar, AccionProyecto::Abrir].into_iter().enumerate() {
        let boton = diseno.proyecto(i);
        assert!(estudio.pulsar(diseno, boton.x + 10.0, boton.y + 10.0));
        assert_eq!(estudio.tomar_accion_proyecto(), Some(accion));
        assert_eq!(estudio.tomar_accion_proyecto(), None);
        assert_eq!(estudio.guardar_estado(), estado);
    }
    let oculto = estudio.distribucion((640, 480));
    let boton = oculto.proyecto(0);
    assert!(!estudio.pulsar(oculto, boton.x + 10.0, boton.y + 10.0));
    assert_eq!(estudio.tomar_accion_proyecto(), None);
}
