//! Comportamiento observable de cascada, cajas, errores y tema distribuido.
#![allow(clippy::unwrap_used, clippy::float_cmp, reason = "valores exactos y fallos de verificación")]
use telar_estilo::{Caja, Estado, Hoja, TEMA_TELAR};

#[test]
fn cascada_respeta_especificidad_orden_y_propiedades_independientes() {
    let hoja = Hoja::analizar(
        "
        .nodo:hover { color: #ff0000; padding: 8px; }
        .nodo { color: #ffffff; background-color: #123456; padding: 2px; }
        .nodo:checked { border-width: 2px; }
        .nodo:checked:hover { color: #00ff00; }
        .nodo:hover { border-radius: 6px; }
    ",
    )
    .unwrap();
    let normal = hoja.resolver("nodo", Estado::default(), None);
    let hover = hoja.resolver("nodo", Estado::nuevo(true, false), None);
    let ambos = hoja.resolver("nodo", Estado::nuevo(true, true), None);
    assert_eq!(normal.color, [1.0; 3]);
    assert_eq!(normal.relleno, 2.0);
    assert_eq!(hover.color, [1.0, 0.0, 0.0]);
    assert_eq!(hover.relleno, 8.0);
    assert_eq!(hover.radio, 6.0);
    assert_eq!(ambos.color, [0.0, 1.0, 0.0]);
    assert_eq!(ambos.grosor, 2.0);
    assert_eq!(normal.fondo, hover.fondo);
}

#[test]
fn variables_resueltas_antes_de_reglas_y_texto_heredado() {
    let hoja = Hoja::analizar(
        "
        .padre { color: var(--tinta); font-size: 16px; padding: 12px; }
        :root { --tinta: #123456; --radio: var(--paso); --paso: 4px; }
        .hijo { border-radius: var(--radio); }
        :root { --tinta: #abcdef; }
    ",
    )
    .unwrap();
    let padre = hoja.resolver("padre", Estado::default(), None);
    let hijo = hoja.resolver("hijo", Estado::default(), Some(&padre));
    assert_eq!(hijo.color, padre.color);
    assert_eq!(hijo.fuente, 16.0);
    assert_eq!(hijo.radio, 4.0);
    assert_eq!(hijo.relleno, 0.0, "el padding no se hereda");
}

#[test]
fn cajas_descuentan_borde_y_padding_sin_dimensiones_negativas() {
    let hoja = Hoja::analizar(".panel { border-width: 2px; padding: 12px; }").unwrap();
    let estilo = hoja.resolver("panel", Estado::default(), None);
    let caja = Caja { x: 10.0, y: 20.0, ancho: 100.0, alto: 60.0 };
    assert_eq!(caja.contenido(&estilo), Caja { x: 24.0, y: 34.0, ancho: 72.0, alto: 32.0 });
    assert_eq!(caja.interior(1000.0).ancho, 0.0);
    assert!(caja.contiene(10.0, 20.0));
    assert!(!caja.contiene(110.0, 80.0));
}

#[test]
fn sintaxis_ajena_incompleta_y_valores_peligrosos_producen_error() {
    for css in [
        ".nodo { color: red; }",
        ".nodo { padding: -1px; }",
        ".nodo { padding: NaNpx; }",
        ".nodo { width: infpx; }",
        ".nodo { font-size: 0; }",
        ".nodo { gap: 5em; }",
        ".nodo { color: #abc; }",
        ".nodo { display: flex; }",
        ".nodo { padding 2px; }",
        ".nodo { padding: ; }",
        ".nodo { color: var(--ausente); }",
        "/* sin cerrar",
        ".nodo { padding: 2px;",
        ".padre .hijo { padding: 2px; }",
        ".nodo:visited { color: #ffffff; }",
        ":root { --a: var(--b); --b: var(--a); }",
        ".nodo { box-shadow: 0px 2px 4px #000000; }",
        ".nodo { .hijo { padding: 0; } }",
        ".nodo { padding: 9000px; }",
    ] {
        assert!(Hoja::analizar(css).is_err(), "debe rechazar: {css}");
    }
}

#[test]
fn tema_distribuido_y_estados_de_foco_son_validos() {
    let tema = Hoja::analizar(TEMA_TELAR).unwrap();
    let normal = tema.resolver("nodo", Estado::default(), None);
    let seleccionado = tema.resolver("nodo", Estado::nuevo(false, true), None);
    assert_ne!(normal.fondo, seleccionado.fondo);
    assert!(normal.sombra.is_some());
    let hoja = Hoja::analizar(".boton:focus:active { border-width: 3px; }").unwrap();
    let estado = Estado::default().con_foco(true).con_pulsacion(true);
    assert_eq!(hoja.resolver("boton", estado, None).grosor, 3.0);
    assert_eq!(hoja.resolver("boton", estado.con_foco(false), None).grosor, 0.0);
}
