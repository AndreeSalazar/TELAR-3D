//! `transformar`: escala, gira y traslada los puntos de su entrada. Ver `libreto.md`.

use alloc::string::String;
use alloc::vec::Vec;

use nudo::mate::{Vec3, multiplicar, rotar, sumar};
use nudo::{Geometria, Nodo, Valor};

use crate::parametros::{desconocido, real, vector};

/// Escala, rotación y traslación, en ese orden (SRT); la rotación, en X, luego Y, luego Z.
#[derive(Debug, Clone, PartialEq)]
pub struct Transformar {
    /// Cuánto se mueve.
    pub traslacion: Vec3,
    /// Cuántos grados gira alrededor de X, Y y Z.
    pub rotacion: Vec3,
    /// Cuánto se estira en cada eje.
    pub escala: Vec3,
    /// Una escala más, igual en los tres ejes.
    pub escala_uniforme: f32,
}

impl Default for Transformar {
    fn default() -> Self {
        Self { traslacion: [0.0; 3], rotacion: [0.0; 3], escala: [1.0; 3], escala_uniforme: 1.0 }
    }
}

impl Transformar {
    /// Transforma un punto: escala, gira (X, Y, Z) y traslada.
    #[must_use]
    pub fn aplicar(&self, p: Vec3) -> Vec3 {
        let e = self.escala_uniforme;
        let mut q = multiplicar(p, [self.escala[0] * e, self.escala[1] * e, self.escala[2] * e]);
        for eje in 0..3 {
            if self.rotacion[eje] != 0.0 {
                q = rotar(q, eje, self.rotacion[eje]);
            }
        }
        sumar(q, self.traslacion)
    }
}

impl Nodo for Transformar {
    fn tipo(&self) -> &'static str {
        "transformar"
    }

    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        alloc::vec![
            ("traslacion", Valor::Vector(self.traslacion)),
            ("rotacion", Valor::Vector(self.rotacion)),
            ("escala", Valor::Vector(self.escala)),
            ("escala_uniforme", Valor::Real(self.escala_uniforme)),
        ]
    }

    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        match nombre {
            "traslacion" => self.traslacion = vector(nombre, valor)?,
            "rotacion" => self.rotacion = vector(nombre, valor)?,
            "escala" => self.escala = vector(nombre, valor)?,
            "escala_uniforme" => self.escala_uniforme = real(nombre, valor)?,
            _ => return Err(desconocido(self.tipo(), nombre)),
        }
        Ok(())
    }

    fn cocinar(&self, entradas: &[&Geometria]) -> Result<Geometria, String> {
        let [entrada] = entradas else {
            return Err(alloc::format!("transformar necesita 1 entrada y tiene {}", entradas.len()));
        };
        let mut g = (*entrada).clone();
        for p in &mut g.puntos {
            *p = self.aplicar(*p);
        }
        Ok(g)
    }
}
