//! Controles físicos: dt fraccionario, presión par y confirmación transaccional.
use telar_ui::{DisenoObjetos, EditorHumo, EntradaNumero, Humo};
#[test]
fn dominios_y_confirmacion() -> Result<(), &'static str> {
    let mut humo = Humo::default();
    for (i, v) in [(0, 0.0), (0, f32::NAN), (5, 0.0), (5, 1.0), (7, 33.0), (7, 32.5), (7, 256.0), (8, -1.0), (9, 0.2)] {
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
    let valor = d.valor(2);
    editor.pulsar(d, valor.x + 4.0, valor.y + 4.0);
    for c in "0,015".chars() {
        editor.numero(EntradaNumero::Caracter(c));
    }
    editor.numero(EntradaNumero::Confirmar);
    assert!(!editor.editando());
    assert_eq!(editor.humo.0[8].to_bits(), 0.015f32.to_bits());
    editor.humo.validar()
}
#[test]
fn tiempo_y_colision_se_editan_sin_aceptar_estados_invalidos() -> Result<(), &'static str> {
    let mut editor = EditorHumo::default();
    editor.volumen = true;
    editor.pagina = 2;
    let d = DisenoObjetos::nuevo((1280, 800));
    for (fila, texto, esperado) in [(0, "4", 4.0f32), (1, "0", 0.0), (2, "0,6", 0.6), (3, "0.25", 0.25)] {
        let rect = d.valor(fila);
        editor.pulsar(d, rect.x + 4.0, rect.y + 4.0);
        for c in texto.chars() {
            editor.numero(EntradaNumero::Caracter(c));
        }
        editor.numero(EntradaNumero::Confirmar);
        assert!(!editor.editando());
        assert_eq!(editor.volumen_opciones.0[fila].to_bits(), esperado.to_bits());
    }
    for (i, valor) in [(0, 2.5), (0, 0.0), (0, 5.0), (1, -0.1), (1, 0.4), (2, 0.9), (3, 0.0), (3, f32::NAN)] {
        let previo = editor.volumen_opciones;
        assert!(editor.volumen_opciones.poner(i, valor).is_err());
        assert_eq!(editor.volumen_opciones, previo);
    }
    editor.volumen_opciones.validar()?;
    editor.volumen = false;
    assert!(editor.control(0).is_none());
    Ok(())
}
#[test]
fn pulso_se_confirma_y_los_dominios_son_transaccionales() -> Result<(), &'static str> {
    let mut editor = EditorHumo::default();
    editor.volumen = true;
    editor.pagina = 3;
    let d = DisenoObjetos::nuevo((1280, 800));
    for (fila, texto, esperado) in [(0, "0,35", 0.35f32), (1, "12", 12.0), (2, "7", 7.0), (3, "0.5", 0.5)] {
        let rect = d.valor(fila);
        editor.pulsar(d, rect.x + 4.0, rect.y + 4.0);
        for c in texto.chars() {
            editor.numero(EntradaNumero::Caracter(c));
        }
        editor.numero(EntradaNumero::Confirmar);
        assert!(!editor.editando());
        assert_eq!(editor.explosion.0[fila].to_bits(), esperado.to_bits());
    }
    for (i, valor) in [(0, -0.1), (0, 3.0), (1, 21.0), (2, 11.0), (3, 5.0), (1, f32::NAN)] {
        let previo = editor.explosion;
        assert!(editor.explosion.poner(i, valor).is_err());
        assert_eq!(editor.explosion, previo);
    }
    editor.explosion.poner(0, 0.0)?;
    editor.explosion.validar()
}
#[test]
fn inspector_de_particulas_aplica_dominios_propios() -> Result<(), &'static str> {
    let mut editor = EditorHumo::default();
    editor.volumen = true;
    editor.particulas = Some(telar_ui::Particulas::default());
    let d = DisenoObjetos::nuevo((1280, 800));
    for (pagina, fila, texto) in [(0, 0, "37"), (0, 1, "0,5"), (1, 1, "123"), (1, 2, "0")] {
        editor.pagina = pagina;
        let r = d.valor(fila);
        editor.pulsar(d, r.x + 4.0, r.y + 4.0);
        for c in texto.chars() {
            editor.numero(EntradaNumero::Caracter(c));
        }
        editor.numero(EntradaNumero::Confirmar);
        assert!(!editor.editando());
    }
    let mut p = editor.particulas.ok_or("sin parámetros")?;
    assert_eq!(p.0[0].to_bits(), 37.0f32.to_bits());
    assert_eq!(p.0[1].to_bits(), 0.5f32.to_bits());
    assert_eq!(p.0[7].to_bits(), 123.0f32.to_bits());
    assert_eq!(p.0[8].to_bits(), 0.0f32.to_bits());
    for (i, v) in [(0, 37.5), (0, 129.0), (7, 2.5), (1, 0.0), (6, 1.0), (9, f32::NAN)] {
        let previo = p;
        assert!(p.poner(i, v).is_err());
        assert_eq!(p, previo);
    }
    Ok(())
}
