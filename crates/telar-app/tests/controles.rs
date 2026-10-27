//! Los controles del inspector no deben mover la cámara ni la fase pausada.
#![allow(clippy::float_cmp, clippy::expect_used, reason = "estado exacto y mensajes de prueba conocidos")]
#[path = "../src/controles.rs"]
mod controles;

use controles::Controles;
use telar_ui::Distribucion;
use telar_ventana::{Boton, Evento, Tecla};

#[test]
fn enter_confirma_despues_del_texto_pendiente_de_windows() {
    use telar_ventana::evento::{WM_CHAR, WM_KEYDOWN, traducir};
    let mut control = Controles::nuevo(true);
    let d = control.estudio.distribucion((1280, 720));
    let r = d.valor(0);
    control.estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
    // TranslateMessage puede dejar WM_CHAR detrás de teclas ya encoladas.
    control.evento(traducir(WM_KEYDOWN, 13, 0).expect("Enter traducido"), d);
    for c in ['0', '.', '4', '2', '\r'] {
        control.evento(traducir(WM_CHAR, c as usize, 0).expect("carácter traducido"), d);
    }
    assert_eq!(control.estudio.onda.amplitud, 0.42);
    assert!(!control.estudio.editando_numero());
}

#[test]
fn enter_numerico_y_retroceso_respetan_el_orden_y_no_aplican_valores_invalidos() {
    use telar_ventana::evento::{WM_CHAR, WM_KEYDOWN, traducir};
    for extendido in [0, 1 << 24] {
        let mut control = Controles::nuevo(true);
        let d = control.estudio.distribucion((1280, 720));
        let r = d.valor(0);
        control.estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
        control.evento(traducir(WM_KEYDOWN, 8, 0).expect("retroceso"), d);
        control.evento(traducir(WM_KEYDOWN, 13, extendido).expect("Enter principal/numérico"), d);
        for c in ['0', '.', '4', '2', '\u{8}', '1', '\r'] {
            control.evento(traducir(WM_CHAR, c as usize, 0).expect("carácter"), d);
        }
        assert_eq!(control.estudio.onda.amplitud, 0.41);
        assert!(!control.estudio.editando_numero());
        control.estudio.seleccionado = 3;
        let r = d.numero_muestras();
        control.estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
        control.evento(Evento::Texto('3'), d);
        control.evento(Evento::Tecla(Tecla::Otra(13)), d);
        control.evento(Evento::Texto('\r'), d);
        assert_eq!(control.estudio.salida.muestras, 2);
        assert!(control.estudio.editando_numero());
        control.evento(Evento::Tecla(Tecla::Otra(46)), d);
        control.evento(Evento::Texto('4'), d);
        control.evento(Evento::Texto('\r'), d);
        assert_eq!(control.estudio.salida.muestras, 4);
        assert!(!control.estudio.editando_numero());
    }
}

#[test]
fn escribir_no_activa_atajos_y_escape_no_cambia_parametros() {
    let mut control = Controles::nuevo(true);
    let d = control.estudio.distribucion((1280, 720));
    let r = d.valor(0);
    control.estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
    for c in "0.42".chars() {
        control.evento(Evento::Texto(c), d);
    }
    control.evento(Evento::Tecla(Tecla::Otra(13)), d);
    control.evento(Evento::Texto('\r'), d);
    assert_eq!(control.estudio.onda.amplitud, 0.42);
    control.estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
    control.evento(Evento::Texto('0'), d);
    control.evento(Evento::Tecla(Tecla::Escape), d);
    assert!(!control.estudio.editando_numero());
    assert_eq!(control.estudio.onda.amplitud, 0.42);
}

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
