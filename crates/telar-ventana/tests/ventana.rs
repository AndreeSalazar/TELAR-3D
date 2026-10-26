//! La ventana: la traducción de mensajes (pura) y una ventana de verdad, oculta.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use telar_ventana::evento::{WM_CLOSE, WM_KEYDOWN, WM_MOUSEMOVE, WM_SIZE, traducir, utf16};
use telar_ventana::{Config, Evento, Tecla, Ventana};

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
