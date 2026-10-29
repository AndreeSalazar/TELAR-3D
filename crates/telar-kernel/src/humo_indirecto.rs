//! Argumentos de presión en binding 0, separados del estado físico de binding 1.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
/// Programas pesados que pueden omitir grupos tras converger.
pub const PROGRAMAS: [usize; 4] = [9, 10, 11, 12];
/// Dos comandos u32 x/y/z: Jacobi/residual completo y reducción parcial.
pub const OFFSETS: [u64; 4] = [0, 0, 0, 12];
pub(super) fn publicar(f: &mut Funcion, m: &Memoria, activo: Id) {
    let cero = Memoria::c(f, 0.0);
    let activo = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, activo, cero);
    let cero_u = Memoria::u(f, 0);
    let uno_u = Memoria::u(f, 1);
    let grupos = m.celdas().div_ceil(64);
    for (comando, cantidad) in [grupos, grupos.div_ceil(64)].into_iter().enumerate() {
        let maximo = Memoria::u(f, cantidad);
        let x = f.elegir(m.entero, activo, maximo, cero_u);
        for (eje, valor) in [x, uno_u, uno_u].into_iter().enumerate() {
            m.guardar_argumento(f, u32::try_from(comando * 3 + eje).unwrap_or(0), valor);
        }
    }
}
