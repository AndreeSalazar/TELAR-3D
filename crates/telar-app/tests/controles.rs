//! Los controles del inspector no deben mover la cámara ni la fase pausada.
#![allow(clippy::float_cmp, reason = "estado exacto y ausencia de cambios tras eventos")]
#[path = "../src/controles.rs"]
mod controles;

use controles::Controles;
use telar_ui::Distribucion;
use telar_ventana::{Boton, Evento, Tecla};

#[test]
fn inspector_camara_y_pausa_tienen_eventos_separados() {
    let diseno = Distribucion::nueva((1280, 720));
    let mut control = Controles::nuevo(true);
    let inicial = control.camara.clone();
    control.evento(Evento::Raton { x: 1100, y: 248 }, diseno);
    control.evento(Evento::Boton { boton: Boton::Izquierdo, pulsado: true }, diseno);
    control.evento(Evento::Raton { x: 1300, y: 248 }, diseno);
    assert_eq!(control.estudio.amplitud, 0.8);
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
}
