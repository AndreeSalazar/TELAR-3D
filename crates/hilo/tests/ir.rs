//! Contratos de tipado y semántica del intérprete, sin Vulkan.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "asserts de tests y valores exactamente representables"
)]

use hilo::{Entrada, Id, Operacion as Op, Programa, Salida, Tipo};

fn programa(operaciones: Vec<Op>, altura: usize, normal: usize) -> Programa {
    Programa::nuevo(operaciones, Salida { altura: Id(altura), normal: Id(normal) }).unwrap()
}

#[test]
fn diagnosticos_situan_el_error_sin_aceptar_referencias_futuras() {
    let salida = Salida { altura: Id(0), normal: Id(1) };
    let error = |ops: Vec<Op>| Programa::nuevo(ops, salida).err().unwrap();
    assert!(error(vec![]).mensaje.contains("256"));
    assert!(error(vec![Op::Escalar(f32::NAN)]).mensaje.contains("finita"));
    assert_eq!(error(vec![Op::Extraer(Id(1), 0), Op::Normal]).instruccion, 0);
    assert_eq!(error(vec![Op::Sumar(Id(0), Id(0)), Op::Normal]).instruccion, 0);
    let e = error(vec![Op::Escalar(1.0), Op::Sumar(Id(0), Id(9))]);
    assert_eq!(e.instruccion, 1);
    assert!(e.mensaje.contains("referencia 9"));
    assert!(error(vec![Op::Escalar(1.0), Op::Normal, Op::Sumar(Id(0), Id(1))]).mensaje.contains("Escalar"));
    assert!(error(vec![Op::Escalar(1.0), Op::Normal, Op::Extraer(Id(1), 3)]).mensaje.contains("componente"));
    assert!(error(vec![Op::Escalar(1.0), Op::Normal, Op::Normalizar(Id(0))]).mensaje.contains("Vector"));
    assert!(error(vec![Op::Escalar(1.0), Op::Normal, Op::Componer([Id(0), Id(1), Id(0)])]).mensaje.contains("Escalar"));
    assert!(error(vec![Op::Vector([f32::INFINITY, 0.0, 0.0])]).mensaje.contains("finito"));
    assert!(Programa::nuevo(vec![Op::Normal], Salida { altura: Id(0), normal: Id(0) }).is_err());
    assert!(Programa::nuevo(vec![Op::Escalar(0.0)], salida).is_err());
    assert!(error(vec![Op::Escalar(0.0); 257]).mensaje.contains("256"));
}

#[test]
fn interpreta_vectores_escalares_y_orden_de_operandos() {
    let p = programa(
        vec![
            Op::Vector([1.0, 2.0, 3.0]),
            Op::Escalar(2.0),
            Op::Multiplicar(Id(0), Id(1)),
            Op::Multiplicar(Id(1), Id(0)),
            Op::Sumar(Id(2), Id(3)),
            Op::Vector([0.5, 0.25, 0.5]),
            Op::Multiplicar(Id(4), Id(5)),
            Op::Restar(Id(6), Id(0)),
            Op::ProductoEscalar(Id(7), Id(0)),
            Op::Escalar(2.0),
            Op::Restar(Id(8), Id(9)),
        ],
        10,
        7,
    );
    assert_eq!(p.tipo(Id(10)), Some(Tipo::Escalar));
    assert_eq!(p.tipo(Id(7)), Some(Tipo::Vector));
    assert_eq!(p.tipo(Id(11)), None);
    let resultado =
        p.interpretar(Entrada { posicion: [4.0, 8.0, 16.0], normal: [0.0; 3], fase: 0.0, amplitud: 0.0 }).unwrap();
    assert_eq!(resultado.normal, [1.0, 0.0, 3.0]);
    assert_eq!(resultado.posicion, [4.0, 8.0, 16.0]);
}

#[test]
fn normalizacion_segura_y_parametros_sin_recompilacion() {
    let p = programa(vec![Op::Fase, Op::Amplitud, Op::Sumar(Id(0), Id(1)), Op::Normal, Op::Normalizar(Id(3))], 2, 4);
    let entrada = Entrada { posicion: [2.0, 0.0, -2.0], normal: [0.0; 3], fase: 0.25, amplitud: 0.5 };
    let nulo = p.interpretar(entrada).unwrap();
    assert_eq!(nulo.posicion, [2.0, 0.75, -2.0]);
    assert_eq!(nulo.normal, [0.0, 1.0, 0.0]);
    let grande = p.interpretar(Entrada { normal: [f32::MAX, 0.0, 0.0], ..entrada }).unwrap();
    assert_eq!(grande.normal, [1.0, 0.0, 0.0]);
    assert!(p.interpretar(Entrada { fase: f32::NAN, ..entrada }).is_err());
    let desborda = programa(vec![Op::Escalar(f32::MAX), Op::Multiplicar(Id(0), Id(0)), Op::Normal], 1, 2);
    assert_eq!(desborda.interpretar(entrada).err().unwrap().instruccion, 1);
}

#[test]
fn relieve_coincide_con_la_normal_geometrica() {
    let entrada = Entrada { posicion: [1.0, 2.0, 3.0], normal: [0.6, 0.8, 0.0], fase: 0.0, amplitud: 0.0 };
    let p = hilo::ejemplos::relieve(-2.0).unwrap();
    let resultado = p.interpretar(entrada).unwrap();
    assert_eq!(resultado.posicion, [1.0, -4.0, 3.0]);
    let longitud = (1.2f32 * 1.2 + 0.8 * 0.8).sqrt();
    assert!((resultado.normal[0] + 1.2 / longitud).abs() < 0.000_001);
    assert!((resultado.normal[1] - 0.8 / longitud).abs() < 0.000_001);
}
