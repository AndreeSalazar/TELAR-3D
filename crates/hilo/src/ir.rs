//! IR lineal en SSA: cada instrucción define un valor y solo usa valores anteriores.

use alloc::{format, string::String, vec::Vec};
use core::fmt;

/// Tipo de un valor del programa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo {
    /// Número f32.
    Escalar,
    /// Tres componentes f32.
    Vector,
}

/// Índice de instrucción, validado dentro del programa que lo contiene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(pub usize);

/// Operaciones puras por punto: sin accesos arbitrarios a memoria ni atómicos.
#[derive(Clone, Copy, Debug)]
pub enum Operacion {
    /// Constante finita f32.
    Escalar(f32),
    /// Constante vectorial finita.
    Vector([f32; 3]),
    /// Posición de entrada del punto actual.
    Posicion,
    /// Normal de entrada del punto actual.
    Normal,
    /// Fase del buffer de parámetros, sin recompilar.
    Fase,
    /// Amplitud del buffer de parámetros, sin recompilar.
    Amplitud,
    /// Escalar propio de nodo, índice 0..3 de la extensión de parámetros.
    Parametro(u32),
    /// Suma de dos valores del mismo tipo.
    Sumar(Id, Id),
    /// Resta de dos valores del mismo tipo.
    Restar(Id, Id),
    /// Producto escalar, componente a componente o vector por escalar.
    Multiplicar(Id, Id),
    /// Extrae una componente 0, 1 o 2 de un vector.
    Extraer(Id, u32),
    /// Compone un vector a partir de tres escalares.
    Componer([Id; 3]),
    /// Producto escalar de dos vectores.
    ProductoEscalar(Id, Id),
    /// Vector unitario; el vector nulo produce [0, 1, 0].
    Normalizar(Id),
}

/// Salidas restringidas al ABI de rejilla: conserva posición x/z e índices.
#[derive(Clone, Copy, Debug)]
pub struct Salida {
    /// Valor escalar que reemplaza posición.y.
    pub altura: Id,
    /// Valor vectorial que reemplaza la normal.
    pub normal: Id,
}

/// Diagnóstico con índice exacto; en la salida, el índice es la longitud del IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// Instrucción o salida que no cumple el contrato.
    pub instruccion: usize,
    /// Motivo legible.
    pub mensaje: String,
}

impl Error {
    pub(crate) fn nuevo(instruccion: usize, mensaje: impl Into<String>) -> Self {
        Self { instruccion, mensaje: mensaje.into() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formato: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formato, "HILO, instrucción {}: {}", self.instruccion, self.mensaje)
    }
}
impl core::error::Error for Error {}

/// Programa validado e inmutable, con un máximo de 256 instrucciones.
pub struct Programa {
    pub(crate) operaciones: Vec<Operacion>,
    pub(crate) tipos: Vec<Tipo>,
    pub(crate) salida: Salida,
}

impl Programa {
    /// Valida referencias anteriores, tipos, constantes, componentes y salidas.
    ///
    /// # Errors
    /// Devuelve el índice y motivo de cada construcción inválida. No demuestra
    /// ausencia de desbordamientos numéricos para todos los valores de entrada.
    pub fn nuevo(operaciones: Vec<Operacion>, salida: Salida) -> Result<Self, Error> {
        if operaciones.is_empty() || operaciones.len() > 256 {
            return Err(Error::nuevo(0, "se requieren de 1 a 256 instrucciones"));
        }
        let mut tipos = Vec::with_capacity(operaciones.len());
        for (indice, operacion) in operaciones.iter().enumerate() {
            tipos.push(tipo_operacion(indice, *operacion, &tipos)?);
        }
        let indice = operaciones.len();
        exigir(indice, tipo_referencia(indice, salida.altura, &tipos)?, Tipo::Escalar)?;
        exigir(indice, tipo_referencia(indice, salida.normal, &tipos)?, Tipo::Vector)?;
        Ok(Self { operaciones, tipos, salida })
    }

    /// Instrucciones en orden de dependencia.
    #[must_use]
    pub fn operaciones(&self) -> &[Operacion] {
        &self.operaciones
    }

    /// Tipo de una referencia, o None si no pertenece al intervalo del programa.
    #[must_use]
    pub fn tipo(&self, id: Id) -> Option<Tipo> {
        self.tipos.get(id.0).copied()
    }
}

fn tipo_referencia(indice: usize, id: Id, tipos: &[Tipo]) -> Result<Tipo, Error> {
    tipos.get(id.0).copied().ok_or_else(|| {
        Error::nuevo(indice, format!("la referencia {} no está definida antes de esta instrucción", id.0))
    })
}

fn exigir(indice: usize, actual: Tipo, esperado: Tipo) -> Result<(), Error> {
    if actual == esperado {
        Ok(())
    } else {
        Err(Error::nuevo(indice, format!("se esperaba {esperado:?}, se recibió {actual:?}")))
    }
}

fn tipo_operacion(indice: usize, operacion: Operacion, tipos: &[Tipo]) -> Result<Tipo, Error> {
    let tipo = |id| tipo_referencia(indice, id, tipos);
    match operacion {
        Operacion::Escalar(valor) => {
            if !valor.is_finite() {
                return Err(Error::nuevo(indice, "constante no finita"));
            }
            Ok(Tipo::Escalar)
        }
        Operacion::Vector(valores) => {
            if !valores.iter().all(|v| v.is_finite()) {
                return Err(Error::nuevo(indice, "vector no finito"));
            }
            Ok(Tipo::Vector)
        }
        Operacion::Posicion | Operacion::Normal => Ok(Tipo::Vector),
        Operacion::Fase | Operacion::Amplitud => Ok(Tipo::Escalar),
        Operacion::Parametro(indice_parametro) => {
            if indice_parametro > 3 {
                return Err(Error::nuevo(indice, "parámetro fuera de 0..=3"));
            }
            Ok(Tipo::Escalar)
        }
        Operacion::Sumar(a, b) | Operacion::Restar(a, b) => {
            let (a, b) = (tipo(a)?, tipo(b)?);
            exigir(indice, b, a)?;
            Ok(a)
        }
        Operacion::Multiplicar(a, b) => {
            let (a, b) = (tipo(a)?, tipo(b)?);
            Ok(if a == Tipo::Vector || b == Tipo::Vector { Tipo::Vector } else { Tipo::Escalar })
        }
        Operacion::Extraer(valor, componente) => {
            exigir(indice, tipo(valor)?, Tipo::Vector)?;
            if componente > 2 {
                return Err(Error::nuevo(indice, "componente fuera de 0..=2"));
            }
            Ok(Tipo::Escalar)
        }
        Operacion::Componer(ids) => {
            for id in ids {
                exigir(indice, tipo(id)?, Tipo::Escalar)?;
            }
            Ok(Tipo::Vector)
        }
        Operacion::ProductoEscalar(a, b) => {
            exigir(indice, tipo(a)?, Tipo::Vector)?;
            exigir(indice, tipo(b)?, Tipo::Vector)?;
            Ok(Tipo::Escalar)
        }
        Operacion::Normalizar(id) => {
            exigir(indice, tipo(id)?, Tipo::Vector)?;
            Ok(Tipo::Vector)
        }
    }
}
