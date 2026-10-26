//! `rejilla`: un plano de filas × columnas de puntos, en el suelo (plano ZX).
//! Ver `libreto.md`.

use alloc::string::String;
use alloc::vec::Vec;

use nudo::mate::Vec3;
use nudo::{Geometria, Nodo, Valor};

use crate::parametros::{desconocido, entero, vector};

/// Una rejilla de cuadriláteros en el plano ZX (la normal mira hacia +Y).
#[derive(Debug, Clone, PartialEq)]
pub struct Rejilla {
    /// Tamaño en X (el primero) y en Z (el segundo); el tercero no se usa.
    pub tamano: Vec3,
    /// Filas de puntos (a lo largo de Z), al menos 2.
    pub filas: i32,
    /// Columnas de puntos (a lo largo de X), al menos 2.
    pub columnas: i32,
    /// Dónde queda su centro.
    pub centro: Vec3,
}

impl Default for Rejilla {
    fn default() -> Self {
        Self { tamano: [10.0, 10.0, 0.0], filas: 10, columnas: 10, centro: [0.0; 3] }
    }
}

impl Nodo for Rejilla {
    fn tipo(&self) -> &'static str {
        "rejilla"
    }

    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        alloc::vec![
            ("tamano", Valor::Vector(self.tamano)),
            ("filas", Valor::Entero(self.filas)),
            ("columnas", Valor::Entero(self.columnas)),
            ("centro", Valor::Vector(self.centro)),
        ]
    }

    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        match nombre {
            "tamano" => self.tamano = vector(nombre, valor)?,
            "filas" => self.filas = entero(nombre, valor, 2)?,
            "columnas" => self.columnas = entero(nombre, valor, 2)?,
            "centro" => self.centro = vector(nombre, valor)?,
            _ => return Err(desconocido(self.tipo(), nombre)),
        }
        Ok(())
    }

    #[allow(clippy::cast_precision_loss, reason = "filas y columnas de una rejilla caben de sobra en f32")]
    fn cocinar(&self, _entradas: &[&Geometria]) -> Result<Geometria, String> {
        let filas = u32::try_from(self.filas).ok().filter(|&f| f >= 2).ok_or("hacen falta al menos 2 filas")?;
        let columnas =
            u32::try_from(self.columnas).ok().filter(|&c| c >= 2).ok_or("hacen falta al menos 2 columnas")?;
        let mut g = Geometria::nueva();
        let (paso_x, paso_z) = (self.tamano[0] / (columnas - 1) as f32, self.tamano[1] / (filas - 1) as f32);
        for f in 0..filas {
            for c in 0..columnas {
                let x = -self.tamano[0] / 2.0 + c as f32 * paso_x;
                let z = -self.tamano[1] / 2.0 + f as f32 * paso_z;
                g.punto([self.centro[0] + x, self.centro[1], self.centro[2] + z]);
            }
        }
        let p = |f: u32, c: u32| f * columnas + c;
        for f in 0..filas - 1 {
            for c in 0..columnas - 1 {
                // Antihorario visto desde arriba (+Y): la normal mira hacia arriba.
                g.poligono(&[p(f, c), p(f + 1, c), p(f + 1, c + 1), p(f, c + 1)]);
            }
        }
        Ok(g)
    }
}
