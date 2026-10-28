//! Controles físicos: dt fraccionario, presión par y confirmación transaccional.
use telar_ui::{DisenoObjetos, EditorHumo, EntradaNumero, Humo};
#[test]
fn dominios_y_confirmacion() -> Result<(), &'static str> {
    let mut humo = Humo::default();
    for (i, v) in [(0, 0.0), (0, f32::NAN), (5, 0.0), (5, 1.0), (7, 33.0), (7, 32.5), (7, 256.0)] {
        let previo = humo.0.map(f32::to_bits);
        assert!(humo.poner(i, v).is_err());
        assert_eq!(previo, humo.0.map(f32::to_bits));
    }
    let mut editor = EditorHumo::default();
    let d = DisenoObjetos::nuevo((1280, 800));
    let valor = d.valor(5);
    editor.pulsar(d, valor.x + 4.0, valor.y + 4.0);
    for c in "0,025".chars() {
        editor.numero(EntradaNumero::Caracter(c));
    }
    editor.numero(EntradaNumero::Confirmar);
    assert!(!editor.editando());
    assert_eq!(editor.humo.0[5].to_bits(), 0.025f32.to_bits());
    editor.pagina = 1;
    let valor = d.valor(1);
    editor.pulsar(d, valor.x + 4.0, valor.y + 4.0);
    for c in "33".chars() {
        editor.numero(EntradaNumero::Caracter(c));
    }
    editor.numero(EntradaNumero::Confirmar);
    assert!(editor.editando());
    assert_eq!(editor.humo.0[7].to_bits(), 64.0f32.to_bits());
    editor.numero(EntradaNumero::Borrar);
    for c in "32".chars() {
        editor.numero(EntradaNumero::Caracter(c));
    }
    editor.numero(EntradaNumero::Confirmar);
    assert!(!editor.editando());
    assert_eq!(editor.humo.0[7].to_bits(), 32.0f32.to_bits());
    editor.humo.validar()
}
