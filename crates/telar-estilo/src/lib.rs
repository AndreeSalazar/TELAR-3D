//! Subconjunto CSS explícito: variables, clases, estados, cascada y cajas.
//! Sin Windows, Vulkan, HTML ni dependencias. Se resuelve al cambiar la vista.
#![no_std]
extern crate alloc;

mod parser;
mod propiedades;
mod tipos;
use alloc::{string::String, vec::Vec};
pub use tipos::{Caja, Color, Estado, Estilo, Sombra};

/// Hoja distribuida con TELAR; editable sin tocar el compositor de triángulos.
pub const TEMA_TELAR: &str = include_str!("../temas/telar.css");

#[derive(Default)]
struct Regla {
    clase: String,
    estado: u8,
    estilo: propiedades::Declaraciones,
}

/// Reglas analizadas. No conserva referencias al texto de entrada.
#[derive(Default)]
pub struct Hoja {
    reglas: Vec<Regla>,
}

impl Hoja {
    /// Analiza la hoja completa antes de reemplazar la vista anterior.
    ///
    /// # Errors
    /// Rechaza sintaxis, propiedades y unidades no implementadas, valores no
    /// finitos y variables ausentes o cíclicas. Límite: 1 MiB y 512 reglas.
    pub fn analizar(css: &str) -> Result<Self, String> {
        parser::analizar(css)
    }

    /// Aplica herencia de texto y cascada de clase/estados, en orden de origen.
    /// Los estados añaden especificidad; a igual especificidad gana la última regla.
    #[must_use]
    pub fn resolver(&self, clase: &str, estado: Estado, padre: Option<&Estilo>) -> Estilo {
        let mut estilo = Estilo::default();
        if let Some(padre) = padre {
            estilo.color = padre.color;
            estilo.fuente = padre.fuente;
            estilo.linea = padre.linea;
        }
        for especificidad in 0..=4 {
            for regla in &self.reglas {
                if regla.clase == clase
                    && regla.estado.count_ones() == especificidad
                    && estado.0 & regla.estado == regla.estado
                {
                    regla.estilo.aplicar(&mut estilo);
                }
            }
        }
        estilo
    }
}
