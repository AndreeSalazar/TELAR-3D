//! La geometría, con el modelo de Houdini (según su documentación):
//!
//! - **puntos:** posiciones en el espacio (`P`);
//! - **vértices:** cada uno apunta a un punto; un mismo punto puede estar en
//!   varios vértices (la esquina compartida de dos caras);
//! - **primitivas:** cada una tiene su lista de vértices, en orden (un polígono);
//! - **atributos:** datos con nombre en cualquiera de esas clases, o en la
//!   geometría entera (*detail*).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use crate::mate::Vec3;

/// A qué pertenece un atributo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clase {
    /// Uno por punto.
    Punto,
    /// Uno por vértice.
    Vertice,
    /// Uno por primitiva.
    Primitiva,
    /// Uno para toda la geometría (*detail* en Houdini).
    Detalle,
}

/// Los valores de un atributo, uno por elemento de su clase.
#[derive(Debug, Clone, PartialEq)]
pub enum Valores {
    /// Un número real por elemento (`f32`).
    Real(Vec<f32>),
    /// Un vector por elemento (`Cd`, `N`, `v`…).
    Vector(Vec<Vec3>),
    /// Un entero por elemento (`id`).
    Entero(Vec<i32>),
}

impl Valores {
    /// Cuántos valores hay.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Valores::Real(v) => v.len(),
            Valores::Vector(v) => v.len(),
            Valores::Entero(v) => v.len(),
        }
    }

    /// ¿No hay ninguno?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Los mismos valores, con `n` valores por defecto (cero) al final.
    #[must_use]
    pub fn con_ceros(&self, n: usize) -> Valores {
        match self {
            Valores::Real(v) => Valores::Real(v.iter().copied().chain(core::iter::repeat_n(0.0, n)).collect()),
            Valores::Vector(v) => Valores::Vector(v.iter().copied().chain(core::iter::repeat_n([0.0; 3], n)).collect()),
            Valores::Entero(v) => Valores::Entero(v.iter().copied().chain(core::iter::repeat_n(0, n)).collect()),
        }
    }

    /// Solo ceros: `n` valores del mismo tipo.
    #[must_use]
    pub fn ceros(&self, n: usize) -> Valores {
        match self {
            Valores::Real(_) => Valores::Real(alloc::vec![0.0; n]),
            Valores::Vector(_) => Valores::Vector(alloc::vec![[0.0; 3]; n]),
            Valores::Entero(_) => Valores::Entero(alloc::vec![0; n]),
        }
    }

    /// `self` seguido de `otros`, si son del mismo tipo.
    #[must_use]
    pub fn juntar(&self, otros: &Valores) -> Option<Valores> {
        Some(match (self, otros) {
            (Valores::Real(a), Valores::Real(b)) => Valores::Real(a.iter().chain(b).copied().collect()),
            (Valores::Vector(a), Valores::Vector(b)) => Valores::Vector(a.iter().chain(b).copied().collect()),
            (Valores::Entero(a), Valores::Entero(b)) => Valores::Entero(a.iter().chain(b).copied().collect()),
            _ => return None,
        })
    }
}

/// Una geometría: puntos, vértices, primitivas y atributos.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Geometria {
    /// La posición de cada punto (el atributo `P` de Houdini).
    pub puntos: Vec<Vec3>,
    /// A qué punto apunta cada vértice.
    pub vertices: Vec<u32>,
    /// Los vértices de cada primitiva: un rango dentro de `vertices`.
    pub primitivas: Vec<Range<u32>>,
    /// Los demás atributos, por clase y nombre.
    pub atributos: BTreeMap<(Clase, String), Valores>,
}

impl Geometria {
    /// Una geometría vacía.
    #[must_use]
    pub fn nueva() -> Self {
        Self::default()
    }

    /// Añade un punto y devuelve su número.
    pub fn punto(&mut self, p: Vec3) -> u32 {
        self.puntos.push(p);
        u32::try_from(self.puntos.len() - 1).unwrap_or(u32::MAX)
    }

    /// Añade un polígono con esos puntos (en orden: el sentido define hacia dónde mira).
    pub fn poligono(&mut self, puntos: &[u32]) {
        let inicio = u32::try_from(self.vertices.len()).unwrap_or(u32::MAX);
        self.vertices.extend_from_slice(puntos);
        let fin = u32::try_from(self.vertices.len()).unwrap_or(u32::MAX);
        self.primitivas.push(inicio..fin);
    }

    /// Cuántos elementos tiene una clase.
    #[must_use]
    pub fn cantidad(&self, clase: Clase) -> usize {
        match clase {
            Clase::Punto => self.puntos.len(),
            Clase::Vertice => self.vertices.len(),
            Clase::Primitiva => self.primitivas.len(),
            Clase::Detalle => 1,
        }
    }

    /// Pone un atributo.
    ///
    /// # Errors
    ///
    /// Si no tiene un valor por elemento de su clase.
    pub fn poner(&mut self, clase: Clase, nombre: &str, valores: Valores) -> Result<(), String> {
        if valores.len() != self.cantidad(clase) {
            return Err(alloc::format!(
                "el atributo `{nombre}` tiene {} valores y la clase {clase:?} tiene {} elementos",
                valores.len(),
                self.cantidad(clase)
            ));
        }
        self.atributos.insert((clase, String::from(nombre)), valores);
        Ok(())
    }

    /// Un atributo, si existe.
    #[must_use]
    pub fn atributo(&self, clase: Clase, nombre: &str) -> Option<&Valores> {
        self.atributos.get(&(clase, String::from(nombre)))
    }

    /// ¿Es coherente? Cada vértice apunta a un punto que existe, cada primitiva
    /// cae dentro de los vértices y cada atributo tiene un valor por elemento.
    ///
    /// # Errors
    ///
    /// La primera incoherencia que encuentra.
    pub fn comprobar(&self) -> Result<(), String> {
        let n_puntos = self.puntos.len();
        if let Some(v) = self.vertices.iter().find(|&&v| v as usize >= n_puntos) {
            return Err(alloc::format!("un vértice apunta al punto {v} y solo hay {n_puntos}"));
        }
        let n_vertices = u32::try_from(self.vertices.len()).unwrap_or(u32::MAX);
        if let Some(p) = self.primitivas.iter().find(|p| p.start > p.end || p.end > n_vertices) {
            return Err(alloc::format!("la primitiva {p:?} se sale de los {n_vertices} vértices"));
        }
        for ((clase, nombre), valores) in &self.atributos {
            if valores.len() != self.cantidad(*clase) {
                return Err(alloc::format!("el atributo `{nombre}` ({clase:?}) no tiene un valor por elemento"));
            }
        }
        Ok(())
    }
}
