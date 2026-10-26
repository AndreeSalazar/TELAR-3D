//! `caja`: un cubo de 8 puntos y 6 caras cuadradas. Ver `libreto.md`.

use alloc::string::String;
use alloc::vec::Vec;

use nudo::mate::Vec3;
use nudo::{Geometria, Nodo, Valor};

use crate::parametros::{desconocido, vector};

/// Las 6 caras: cada una con sus 4 esquinas en orden antihorario visto desde
/// fuera (la normal, por la regla de la mano derecha, mira hacia fuera). Las
/// esquinas se numeran con un bit por eje: `x + 2y + 4z` (0 = lado negativo).
const CARAS: [[u32; 4]; 6] = [
    [0, 4, 6, 2], // −X
    [1, 3, 7, 5], // +X
    [0, 1, 5, 4], // −Y
    [2, 6, 7, 3], // +Y
    [0, 2, 3, 1], // −Z
    [4, 5, 7, 6], // +Z
];

/// Una caja alineada con los ejes.
#[derive(Debug, Clone, PartialEq)]
pub struct Caja {
    /// Lado en X, Y y Z.
    pub tamano: Vec3,
    /// Dónde queda su centro.
    pub centro: Vec3,
}

impl Default for Caja {
    fn default() -> Self {
        Self { tamano: [1.0; 3], centro: [0.0; 3] }
    }
}

impl Nodo for Caja {
    fn tipo(&self) -> &'static str {
        "caja"
    }

    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        alloc::vec![("tamano", Valor::Vector(self.tamano)), ("centro", Valor::Vector(self.centro))]
    }

    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        match nombre {
            "tamano" => self.tamano = vector(nombre, valor)?,
            "centro" => self.centro = vector(nombre, valor)?,
            _ => return Err(desconocido(self.tipo(), nombre)),
        }
        Ok(())
    }

    fn cocinar(&self, _entradas: &[&Geometria]) -> Result<Geometria, String> {
        let mut g = Geometria::nueva();
        for esquina in 0..8u8 {
            let lado = |eje: u8| if esquina & (1 << eje) == 0 { -0.5 } else { 0.5 };
            let i = usize::from;
            g.punto([0u8, 1, 2].map(|eje| self.centro[i(eje)] + lado(eje) * self.tamano[i(eje)]));
        }
        for cara in CARAS {
            g.poligono(&cara);
        }
        Ok(g)
    }
}
