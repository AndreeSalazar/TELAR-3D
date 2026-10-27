//! Los controles del inspector no deben mover la cámara ni la fase pausada.
#![allow(clippy::float_cmp, reason = "estado exacto y ausencia de cambios tras eventos")]
#[path = "../src/controles.rs"]
mod controles;

use controles::Controles;
use telar_ui::Distribucion;
use telar_ventana::{Boton, Evento, Tecla};

#[test]
fn restaurar_interaccion_corta_arrastre_y_atajos_no_modifican_la_escena() {
    let mut control = Controles::nuevo(true);
    let diseno = control.estudio.distribucion((1280, 720));
    control.evento(Evento::Raton { x: 300, y: 300 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
    control.restaurar_interaccion();
    let camara = control.camara.clone();
    control.evento(Evento::Raton { x: 400, y: 350 }, diseno);
    assert_eq!(camara, control.camara);
    let estado = control.estudio.guardar_estado();
    for (tecla, accion) in [(116, telar_ui::AccionProyecto::Guardar), (120, telar_ui::AccionProyecto::Abrir)] {
        control.evento(Evento::Tecla(Tecla::Otra(tecla)), diseno);
        assert_eq!(control.estudio.tomar_accion_proyecto(), Some(accion));
        assert_eq!(control.estudio.guardar_estado(), estado);
    }
}

#[test]
#[allow(clippy::cast_possible_truncation, reason = "coordenadas de controles dentro de una ventana de 1280×720")]
fn inspector_camara_y_pausa_tienen_eventos_separados() {
    let diseno = Distribucion::nueva((1280, 720));
    let mut control = Controles::nuevo(true);
    let inicial = control.camara.clone();
    let pista = diseno.control(0);
    let (x, y) = ((pista.x + pista.ancho * 0.5) as i32, (pista.y + pista.alto * 0.5) as i32);
    control.evento(Evento::Raton { x, y }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
    control.evento(Evento::Raton { x: 1300, y }, diseno);
    assert_eq!(control.estudio.onda.amplitud, 0.8);
    assert_eq!(control.camara, inicial);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, diseno);
    control.evento(Evento::Rueda(1.0), diseno);
    assert_eq!(control.camara, inicial, "la rueda sobre el inspector no hace zoom");
    control.avanzar(0.25);
    let fase = control.fase;
    control.evento(Evento::Tecla(Tecla::Otra(32)), diseno);
    control.avanzar(0.25);
    assert_eq!(control.fase, fase);
    control.evento(Evento::Raton { x: 600, y: 300 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
    control.evento(Evento::Raton { x: 650, y: 325 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, diseno);
    assert_ne!(control.camara, inicial, "el arrastre dentro del visor sí orbita");
    let distancia = control.camara.distancia;
    control.evento(Evento::Rueda(1.0), diseno);
    assert!(control.camara.distancia < distancia);
    let antes_del_grafo = control.camara.clone();
    control.evento(Evento::Raton { x: 1150, y: 480 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
    control.evento(Evento::Raton { x: 1050, y: 600 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: false }, diseno);
    assert_eq!(control.camara, antes_del_grafo, "arrastrar un nudo no orbita la escena");
}
