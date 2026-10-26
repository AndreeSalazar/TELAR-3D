//! Leer el valor de un parámetro con el tipo que espera cada nodo.

use alloc::string::String;

use nudo::Valor;

/// El valor como vector, o un error que dice qué parámetro esperaba qué.
///
/// # Errors
///
/// Si el valor no es un vector.
pub fn vector(nombre: &str, valor: Valor) -> Result<[f32; 3], String> {
    match valor {
        Valor::Vector(v) => Ok(v),
        otro => Err(alloc::format!("`{nombre}` espera un vector y llegó {otro:?}")),
    }
}

/// El valor como real (un entero también vale).
///
/// # Errors
///
/// Si el valor es un vector.
pub fn real(nombre: &str, valor: Valor) -> Result<f32, String> {
    match valor {
        Valor::Real(v) => Ok(v),
        #[allow(clippy::cast_precision_loss, reason = "un parámetro entero cabe de sobra en f32")]
        Valor::Entero(v) => Ok(v as f32),
        Valor::Vector(_) => Err(alloc::format!("`{nombre}` espera un número y llegó un vector")),
    }
}

/// El valor como entero, al menos `minimo`.
///
/// # Errors
///
/// Si no es un entero o es menor que `minimo`.
pub fn entero(nombre: &str, valor: Valor, minimo: i32) -> Result<i32, String> {
    match valor {
        Valor::Entero(v) if v >= minimo => Ok(v),
        Valor::Entero(v) => Err(alloc::format!("`{nombre}` tiene que ser al menos {minimo} (llegó {v})")),
        otro => Err(alloc::format!("`{nombre}` espera un entero y llegó {otro:?}")),
    }
}

/// El error de un parámetro que el nodo no tiene.
#[must_use]
pub fn desconocido(tipo: &str, nombre: &str) -> String {
    alloc::format!("el nodo {tipo} no tiene el parámetro `{nombre}`")
}
