//! Identidades independientes, puertos, límites y guardado del grafo real.
#![allow(clippy::unwrap_used, clippy::float_cmp, reason = "aserciones de estados exactos")]
use telar_ui::{EntradaNumero as N, Estudio, OperacionRed as Op, TipoNodo};

fn valor(estudio: &mut Estudio, indice: usize, texto: &str) {
    let d = estudio.distribucion((1280, 720));
    let r = d.valor(indice);
    estudio.pulsar(d, r.x + 4.0, r.y + 4.0);
    for c in texto.chars() {
        estudio.editar_numero(N::Caracter(c));
    }
    estudio.editar_numero(N::Confirmar);
}

#[test]
fn dos_ondas_conservan_sus_valores_y_conexiones_cambian_el_plan() {
    let mut e = Estudio::default();
    let onda = e.crear_nodo(TipoNodo::Onda).unwrap();
    assert_eq!(onda, 4);
    valor(&mut e, 0, "0.17");
    valor(&mut e, 1, "0.3");
    assert_eq!(e.onda.amplitud, 0.35);
    assert_eq!(e.onda.velocidad, 1.0);
    assert_eq!(
        e.operaciones_red().unwrap(),
        vec![
            Op::Onda { amplitud: 0.35, velocidad: 1.0 },
            Op::Relieve { factor: 1.3 },
            Op::Onda { amplitud: 0.17, velocidad: 0.3 }
        ]
    );
    e.conectar_nodos(1, onda).unwrap();
    e.conectar_nodos(onda, 2).unwrap();
    e.conectar_nodos(2, 3).unwrap();
    assert!(matches!(e.operaciones_red().unwrap().last(), Some(Op::Relieve { .. })));
    e.habilitar_nodo(1, false).unwrap();
    assert_eq!(
        e.operaciones_red().unwrap(),
        vec![Op::Onda { amplitud: 0.17, velocidad: 0.3 }, Op::Relieve { factor: 1.3 }]
    );
    let antes = e.guardar_red();
    for (a, b) in [(2, onda), (onda, onda), (3, 1), (1, 0), (99, 2), (2, 99)] {
        assert!(e.conectar_nodos(a, b).is_err());
        assert_eq!(e.guardar_red(), antes);
    }
}

#[test]
fn extension_v3_conserva_identidades_y_rechaza_datos_invalidos_sin_mutar() {
    let mut e = Estudio::default();
    e.crear_nodo(TipoNodo::Onda).unwrap();
    e.crear_nodo(TipoNodo::Relieve).unwrap();
    e.habilitar_nodo(2, false).unwrap();
    let base = e.guardar_estado();
    let red = e.guardar_red();
    let mut copia = Estudio::default();
    copia.restaurar_estado(&base).unwrap();
    copia.restaurar_red(&red).unwrap();
    assert_eq!(copia.guardar_red(), red);
    assert_eq!(copia.operaciones_red().unwrap(), e.operaciones_red().unwrap());
    for (indice, malo) in [
        (0, 11),
        (1, 99),
        (2 + 7, 2),
        (2 + 7 + 1, 2),
        (2 + 7 + 2, f32::NAN.to_bits()),
        (2 + 4 * 7 + 5, f32::INFINITY.to_bits()),
        (2 + 2 * 7 + 4, 3),
    ] {
        let mut alterada = red.clone();
        alterada[indice] = malo;
        assert!(copia.restaurar_red(&alterada).is_err(), "palabra {indice}");
        assert_eq!(copia.guardar_red(), red);
    }
    assert!(copia.restaurar_red(&red[..red.len() - 1]).is_err());
    assert_eq!(copia.guardar_red(), red);
}

#[test]
fn limite_de_operaciones_y_puertos_se_reflejan_en_la_interfaz() {
    let mut e = Estudio::default();
    let d = e.distribucion((1280, 720));
    let b = d.accion_red(0);
    assert!(e.pulsar(d, b.x + 5.0, b.y + 5.0));
    assert_eq!(e.cantidad_nodos(), 5);
    e.desplazar_red(d, 100.0);
    let puerto = e.puerto(d, 0, false).unwrap();
    e.pulsar(d, puerto.x + 7.0, puerto.y + 7.0);
    assert!(e.conectando());
    let entrada = e.puerto(d, 1, true).unwrap();
    e.pulsar(d, entrada.x + 7.0, entrada.y + 7.0);
    assert!(!e.conectando());
    assert_eq!(e.entrada_nodo(1), Some(0));
    while e.cantidad_nodos() < 10 {
        e.crear_nodo(TipoNodo::Onda).unwrap();
    }
    assert!(e.crear_nodo(TipoNodo::Onda).is_err());
    assert!(e.crear_nodo(TipoNodo::Salida).is_err());
    assert!(e.dibujar(d, 256 * 256).iter().flatten().all(|v| v.is_finite()));
}
