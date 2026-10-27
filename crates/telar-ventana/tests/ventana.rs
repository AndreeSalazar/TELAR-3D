//! La ventana: la traducción de mensajes (pura) y una ventana de verdad, oculta.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use telar_ventana::evento::{
    WM_CLOSE, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, traducir, utf16,
};
use telar_ventana::{Boton, Config, Evento, Tecla, Ventana};

#[test]
fn traduce_los_mensajes_que_importan() {
    assert_eq!(traducir(WM_CLOSE, 0, 0), Some(Evento::Cerrar));
    // WM_SIZE: ancho en los 16 bits bajos, alto en los siguientes.
    assert_eq!(traducir(WM_SIZE, 0, (0x2D0 << 16) | 0x500), Some(Evento::Tamano { ancho: 1280, alto: 720 }));
    // Minimizada (SIZE_MINIMIZED): 0 × 0, para no dibujar.
    assert_eq!(traducir(WM_SIZE, 1, 0), Some(Evento::Tamano { ancho: 0, alto: 0 }));
    assert_eq!(traducir(WM_KEYDOWN, 0x1B, 0), Some(Evento::Tecla(Tecla::Escape)));
    assert_eq!(traducir(WM_KEYDOWN, 0x41, 0), Some(Evento::Tecla(Tecla::Otra(0x41))));
    assert_eq!(traducir(0x000F, 0, 0), None); // WM_PAINT: no es asunto de TELAR
}

#[test]
fn el_raton_puede_estar_en_negativo() {
    // x = -5 (0xFFFB), y = 10.
    assert_eq!(traducir(WM_MOUSEMOVE, 0, (10 << 16) | 0xFFFB), Some(Evento::Raton { x: -5, y: 10 }));
}

#[test]
fn botones_del_raton() {
    let boton = |boton, pulsado| Some(Evento::Boton { boton, pulsado });
    assert_eq!(traducir(WM_LBUTTONDOWN, 0, 0), boton(Boton::Izquierdo, true));
    assert_eq!(traducir(WM_LBUTTONUP, 0, 0), boton(Boton::Izquierdo, false));
    assert_eq!(traducir(WM_RBUTTONDOWN, 0, 0), boton(Boton::Derecho, true));
    assert_eq!(traducir(WM_RBUTTONUP, 0, 0), boton(Boton::Derecho, false));
    assert_eq!(traducir(WM_MBUTTONDOWN, 0, 0), boton(Boton::Medio, true));
    assert_eq!(traducir(WM_MBUTTONUP, 0, 0), boton(Boton::Medio, false));
    assert_eq!(traducir(0x0203, 0, 0), None); // WM_LBUTTONDBLCLK: la clase no lo pide
}

#[test]
fn la_rueda_en_muescas_con_signo() {
    // La palabra alta del wParam: +120 hacia delante, −120 (0xFF88) hacia atrás; la
    // baja lleva las teclas pulsadas (aquí, MK_CONTROL) y no cuenta.
    assert_eq!(traducir(WM_MOUSEWHEEL, (120 << 16) | 0x8, 0), Some(Evento::Rueda(1.0)));
    assert_eq!(traducir(WM_MOUSEWHEEL, 0xFF88 << 16, 0), Some(Evento::Rueda(-1.0)));
    assert_eq!(traducir(WM_MOUSEWHEEL, 30 << 16, 0), Some(Evento::Rueda(0.25)));
}

#[test]
fn utf16_terminado_en_cero() {
    assert_eq!(utf16("Añ"), [0x41, 0xF1, 0]);
}

#[test]
fn abre_una_ventana_de_verdad_y_la_cierra() {
    let config = Config { titulo: "TELAR 3D · test".to_owned(), ancho: 320, alto: 200, visible: false };
    let mut ventana = Ventana::nueva(&config).unwrap();
    let _ = ventana.eventos();
    let (ancho, alto) = ventana.tamano();
    assert!(ancho >= 320 && alto >= 200, "el área de dibujo mide {ancho} × {alto}");
    let m = ventana.manejadores();
    assert!(m.instancia != 0 && m.ventana != 0);
    ventana.poner_titulo("otro título");
    ventana.cambiar_tamano(400, 300);
    assert!(ventana.eventos().contains(&Evento::Tamano { ancho: 400, alto: 300 }));
    assert_eq!(ventana.tamano(), (400, 300));
    // Una segunda ventana en el mismo proceso reutiliza la clase registrada.
    let otra = Ventana::nueva(&config).unwrap();
    assert_ne!(otra.manejadores().ventana, m.ventana);
}
