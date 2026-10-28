//! Edición del volumen: transacciones y confirmación del número completo.
use telar_ui::{DisenoObjetos, EditorFuego, EntradaNumero, Fuego};
#[test]
fn enter_dominios_y_pagina() -> Result<(), &'static str> {
    let mut fuego = Fuego::default();
    for (campo, valor) in [(0, f32::NAN), (0, 0.0), (7, 32.5), (7, 256.0), (8, 1.0)] {
        let previo = fuego.0.map(f32::to_bits);
        assert!(fuego.poner(campo, valor).is_err());
        assert_eq!(previo, fuego.0.map(f32::to_bits));
    }
    let mut editor = EditorFuego::default();
    editor.pagina = 1;
    let d = DisenoObjetos::nuevo((1280, 800));
    let valor = d.valor(1);
    editor.pulsar(d, valor.x + 5.0, valor.y + 5.0);
    editor.numero(EntradaNumero::Caracter('3'));
    editor.numero(EntradaNumero::Caracter('2'));
    assert_eq!(editor.fuego.0[7].to_bits(), 64.0f32.to_bits());
    editor.numero(EntradaNumero::Confirmar);
    assert!(!editor.editando());
    assert_eq!(editor.fuego.0[7].to_bits(), 32.0f32.to_bits());
    editor.pulsar(d, valor.x + 5.0, valor.y + 5.0);
    editor.numero(EntradaNumero::Caracter('0'));
    editor.numero(EntradaNumero::Confirmar);
    assert!(editor.editando());
    assert_eq!(editor.fuego.0[7].to_bits(), 32.0f32.to_bits());
    editor.numero(EntradaNumero::Cancelar);
    editor.fuego.validar()
}
