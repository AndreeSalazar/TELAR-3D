//! Programas de referencia construidos con el IR, sin sintaxis textual todavía.
use crate::{Error, Id, Operacion as Op, Programa, Salida};
use alloc::vec;

/// Escala altura y corrige la normal, conservando x/z e índices de la superficie.
///
/// # Errors
/// Rechaza factores no finitos, como cualquier constante del IR.
pub fn relieve(factor: f32) -> Result<Programa, Error> {
    Programa::nuevo(
        vec![
            Op::Posicion,
            Op::Normal,
            Op::Escalar(factor),
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
