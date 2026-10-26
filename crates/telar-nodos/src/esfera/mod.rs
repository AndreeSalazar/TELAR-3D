//! `esfera`: una esfera de meridianos y paralelos (como un globo terráqueo),
//! con un punto en cada polo. Ver `libreto.md`.

use alloc::string::String;
use alloc::vec::Vec;

use nudo::mate::{Vec3, sin_cos};
use nudo::{Geometria, Nodo, Valor};

use crate::parametros::{desconocido, entero, real, vector};

/// Una esfera de polígonos.
#[derive(Debug, Clone, PartialEq)]
pub struct Esfera {
    /// El radio.
    pub radio: f32,
    /// Filas de puntos de polo a polo, polos incluidos (al menos 3).
    pub filas: i32,
    /// Puntos en cada paralelo (al menos 3).
    pub columnas: i32,
    /// Dónde queda su centro.
    pub centro: Vec3,
}

impl Default for Esfera {
    fn default() -> Self {
        Self { radio: 1.0, filas: 13, columnas: 24, centro: [0.0; 3] }
    }
}

impl Nodo for Esfera {
    fn tipo(&self) -> &'static str {
        "esfera"
    }

    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        alloc::vec![
            ("radio", Valor::Real(self.radio)),
            ("filas", Valor::Entero(self.filas)),
            ("columnas", Valor::Entero(self.columnas)),
            ("centro", Valor::Vector(self.centro)),
        ]
    }

    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        match nombre {
            "radio" => self.radio = real(nombre, valor)?,
            "filas" => self.filas = entero(nombre, valor, 3)?,
            "columnas" => self.columnas = entero(nombre, valor, 3)?,
            "centro" => self.centro = vector(nombre, valor)?,
            _ => return Err(desconocido(self.tipo(), nombre)),
        }
        Ok(())
    }

    #[allow(clippy::cast_precision_loss, reason = "filas y columnas de una esfera caben de sobra en f32")]
    fn cocinar(&self, _entradas: &[&Geometria]) -> Result<Geometria, String> {
        let filas = u32::try_from(self.filas).ok().filter(|&f| f >= 3).ok_or("hacen falta al menos 3 filas")?;
        let columnas =
            u32::try_from(self.columnas).ok().filter(|&c| c >= 3).ok_or("hacen falta al menos 3 columnas")?;
        let [cx, cy, cz] = self.centro;
        let r = self.radio;
        let mut g = Geometria::nueva();
        let polo_norte = g.punto([cx, cy + r, cz]);
        // Los paralelos, sin los polos: θ va de 0 (norte) a π (sur).
        for f in 1..filas - 1 {
            let (seno_t, coseno_t) = sin_cos(core::f32::consts::PI * f as f32 / (filas - 1) as f32);
            for c in 0..columnas {
                let (seno_f, coseno_f) = sin_cos(core::f32::consts::TAU * c as f32 / columnas as f32);
                g.punto([cx + r * seno_t * coseno_f, cy + r * coseno_t, cz + r * seno_t * seno_f]);
            }
        }
        let polo_sur = g.punto([cx, cy - r, cz]);

        let anillos = filas - 2;
        let p = |anillo: u32, c: u32| 1 + anillo * columnas + c % columnas;
        // Las caras, en sentido antihorario visto desde fuera (normales hacia fuera).
        for c in 0..columnas {
            g.poligono(&[polo_norte, p(0, c + 1), p(0, c)]);
        }
        for anillo in 0..anillos - 1 {
            for c in 0..columnas {
                g.poligono(&[p(anillo, c), p(anillo, c + 1), p(anillo + 1, c + 1), p(anillo + 1, c)]);
            }
        }
        for c in 0..columnas {
            g.poligono(&[polo_sur, p(anillos - 1, c), p(anillos - 1, c + 1)]);
        }
        Ok(g)
    }
}
