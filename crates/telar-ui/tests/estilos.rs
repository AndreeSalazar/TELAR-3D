//! Una hoja cambia dibujo e interacción sin modificar la escena procedural.
#![allow(clippy::unwrap_used, clippy::float_cmp, reason = "verificaciones exactas")]
use telar_ui::Estudio;

#[test]
fn personalizacion_cambia_cajas_y_pixeles_y_un_error_conserva_el_tema() {
    let mut estudio = Estudio::default();
    let anterior = estudio.distribucion((1280, 720));
    let dibujo = estudio.dibujar(anterior, 4096);
    estudio
        .cargar_estilos(".inspector { width: 310px; } .nodo { width: 116px; } .cabecera { background-color: #552244; }")
        .unwrap();
    let diseno = estudio.distribucion((1280, 720));
    assert!(diseno.visor.ancho > anterior.visor.ancho);
    assert!(estudio.nodo(diseno, 0).ancho < estudio.nodo(anterior, 0).ancho);
    let personalizado = estudio.dibujar(diseno, 4096);
    assert_ne!(dibujo, personalizado);
    assert!(estudio.cargar_estilos(".nodo { padding: oops; }").is_err());
    assert_eq!(personalizado, estudio.dibujar(diseno, 4096));
    let nodo = estudio.nodo(diseno, 0);
    assert!(estudio.pulsar(diseno, nodo.x + 10.0, nodo.y + 10.0));
    assert_eq!(estudio.seleccionado, 0);
    assert_eq!(estudio.amplitud, 0.35);
}
