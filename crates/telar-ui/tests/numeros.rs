//! Edición por texto con confirmación, dominios y estado de escena estable.
#![allow(clippy::float_cmp, reason = "valores exactos solicitados por el usuario")]
use telar_ui::{EntradaNumero as N, Estudio};

fn escribir(estudio: &mut Estudio, indice: usize, texto: &str) {
    let diseno = estudio.distribucion((1280, 720));
    let caja = diseno.valor(indice);
    estudio.pulsar(diseno, caja.x + 5.0, caja.y + 5.0);
    for c in texto.chars() {
        estudio.editar_numero(N::Caracter(c));
    }
    estudio.editar_numero(N::Confirmar);
}

#[test]
fn parametros_admiten_precision_coma_y_enteros_sin_redondear_entrada_invalida() {
    let mut estudio = Estudio::default();
    escribir(&mut estudio, 0, "0,12345");
    assert_eq!(estudio.onda.amplitud, 0.12345);
    assert!(!estudio.editando_numero());
    estudio.seleccionado = 3;
    escribir(&mut estudio, 0, "2.375");
    assert_eq!(estudio.salida.exposicion, 2.375);
    estudio.seleccionado = 0;
    escribir(&mut estudio, 0, "47");
    assert_eq!(estudio.rejilla.lado, 47);
    escribir(&mut estudio, 0, "47.5");
    assert_eq!(estudio.rejilla.lado, 47);
    assert!(estudio.editando_numero());
}

#[test]
fn texto_parcial_y_fuera_de_rango_no_cambia_la_escena_y_escape_cancela() {
    let mut estudio = Estudio::default();
    let original = estudio.guardar_estado();
    for texto in ["-", "1e", "1e99", "-0.5", "0.81"] {
        escribir(&mut estudio, 0, texto);
        assert!(estudio.editando_numero(), "{texto}");
        assert_eq!(estudio.guardar_estado(), original);
        estudio.editar_numero(N::Cancelar);
        assert!(!estudio.editando_numero());
    }
    let caja = estudio.distribucion((1280, 720)).valor(0);
    estudio.pulsar(estudio.distribucion((1280, 720)), caja.x + 5.0, caja.y + 5.0);
    estudio.editar_numero(N::Caracter('0'));
    assert_eq!(estudio.guardar_estado(), original, "no aplica al teclear");
    estudio.editar_numero(N::Cancelar);
    assert_eq!(estudio.guardar_estado(), original);
}

#[test]
fn msaa_es_editable_pero_no_confunde_muestras_de_trazado_con_cobertura() {
    let mut estudio = Estudio::default();
    estudio.seleccionado = 3;
    let diseno = estudio.distribucion((1280, 720));
    let caja = diseno.numero_muestras();
    for muestras in ["2", "256", "3", "16", "1", "4"] {
        estudio.pulsar(diseno, caja.x + 5.0, caja.y + 5.0);
        for c in muestras.chars() {
            estudio.editar_numero(N::Caracter(c));
        }
        let anterior = estudio.salida.muestras;
        estudio.editar_numero(N::Confirmar);
        if let Ok(valido @ (1 | 2 | 4)) = muestras.parse::<u32>() {
            assert_eq!(estudio.salida.muestras, valido);
        } else {
            assert_eq!(estudio.salida.muestras, anterior);
            assert!(estudio.editando_numero());
            estudio.editar_numero(N::Cancelar);
        }
    }
}

#[test]
fn borrar_corregir_y_confirmar_al_salir_respetan_los_valores() {
    let mut estudio = Estudio::default();
    let diseno = estudio.distribucion((800, 600));
    let caja = diseno.valor(0);
    estudio.pulsar(diseno, caja.x + 5.0, caja.y + 5.0);
    estudio.editar_numero(N::Retroceso);
    for c in "0.62".chars() {
        estudio.editar_numero(N::Caracter(c));
    }
    estudio.editar_numero(N::Retroceso);
    estudio.editar_numero(N::Caracter('5'));
    estudio.pulsar(diseno, 200.0, 200.0);
    assert_eq!(estudio.onda.amplitud, 0.65);
    assert!(!estudio.editando_numero());
    estudio.pulsar(diseno, caja.x + 5.0, caja.y + 5.0);
    estudio.editar_numero(N::Borrar);
    estudio.editar_numero(N::Caracter('2'));
    estudio.editar_numero(N::Confirmar);
    assert!(estudio.editando_numero());
    assert_eq!(estudio.onda.amplitud, 0.65);
}
