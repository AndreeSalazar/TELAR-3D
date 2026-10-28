//! Escena independiente, controles, dominios y edición transaccional.
#![allow(clippy::unwrap_used, clippy::float_cmp, reason = "aserciones de estado exacto")]
use telar_ui::{DisenoObjetos, EditorObjetos, EntradaNumero, EscenaObjetos};

#[test]
fn objetos_independientes_y_estado_completo_reproducible() {
    let mut escena = EscenaObjetos::default();
    let azul = escena.valores()[10..20].to_vec();
    escena.poner(0, 1.2).unwrap();
    escena.poner(3, 2.4).unwrap();
    assert_eq!(&escena.valores()[10..20], azul.as_slice());
    escena.pagina = 1;
    escena.poner(0, 0.2).unwrap();
    escena.poner(3, 0.0).unwrap();
    escena.seleccionado = 2;
    escena.pagina = 0;
    escena.poner(4, 40.0).unwrap();
    escena.poner(5, 0.0).unwrap();
    escena.pagina = 1;
    escena.poner(0, 0.1).unwrap();
    let palabras = escena.guardar();
    let mut restaurada = EscenaObjetos::default();
    restaurada.restaurar(&palabras).unwrap();
    assert_eq!(palabras, restaurada.guardar());
    for (campo, invalido) in [(0, f32::NAN.to_bits()), (3, 0), (30, 1.5_f32.to_bits()), (40, 5)] {
        let mut corrupta = palabras;
        corrupta[campo] = invalido;
        assert!(restaurada.restaurar(&corrupta).is_err());
        assert_eq!(palabras, restaurada.guardar());
    }
}

#[test]
fn numeros_enter_escape_y_presupuesto_de_muestras() {
    let mut editor = EditorObjetos::default();
    let diseno = DisenoObjetos::nuevo((1280, 800));
    let campo = diseno.valor(0);
    editor.pulsar(diseno, campo.x + 3.0, campo.y + 3.0);
    for c in "1,25".chars() {
        editor.numero(EntradaNumero::Caracter(c));
    }
    assert_eq!(editor.escena.valores()[0], -0.85);
    editor.numero(EntradaNumero::Confirmar);
    assert_eq!(editor.escena.valores()[0], 1.25);
    editor.pulsar(diseno, campo.x + 3.0, campo.y + 3.0);
    editor.numero(EntradaNumero::Caracter('9'));
    editor.numero(EntradaNumero::Confirmar);
    assert!(editor.editando());
    assert_eq!(editor.escena.valores()[0], 1.25);
    editor.numero(EntradaNumero::Cancelar);
    assert!(!editor.editando());
    editor.escena.seleccionado = 3;
    assert!(editor.escena.poner(1, 256.0).is_err());
    editor.escena.poner(2, 64.0).unwrap();
    editor.escena.poner(3, 64.0).unwrap();
    editor.escena.poner(1, 256.0).unwrap();
    assert_eq!(editor.escena.valores()[30], 256.0);
    assert!(diseno.visor.ancho <= 1024.0 && diseno.visor.alto <= 768.0);
}
