//! Programas de referencia construidos con el IR, sin sintaxis textual todavía.
use crate::{Error, Id, Operacion as Op, Programa, Salida};
use alloc::vec;

/// Escala altura y corrige la normal, conservando x/z e índices de la superficie.
///
/// # Errors
/// Rechaza factores no finitos, como cualquier constante del IR.
pub fn relieve(factor: f32) -> Result<Programa, Error> {
    relieve_con(Op::Escalar(factor))
}

/// Relieve cuyo factor procede del primer escalar vivo del nodo, sin recompilar.
///
/// # Errors
/// Si la construcción del programa deja de cumplir el contrato del IR.
pub fn relieve_dinamico() -> Result<Programa, Error> {
    relieve_con(Op::Parametro(0))
}

fn relieve_con(factor: Op) -> Result<Programa, Error> {
    Programa::nuevo(
        vec![
            Op::Posicion,
            Op::Normal,
            factor,
            Op::Extraer(Id(0), 1),
            Op::Multiplicar(Id(3), Id(2)),
            Op::Extraer(Id(1), 0),
            Op::Multiplicar(Id(5), Id(2)),
            Op::Extraer(Id(1), 1),
            Op::Extraer(Id(1), 2),
            Op::Multiplicar(Id(8), Id(2)),
            Op::Componer([Id(6), Id(7), Id(9)]),
            Op::Normalizar(Id(10)),
        ],
        Salida { altura: Id(4), normal: Id(11) },
    )
}
