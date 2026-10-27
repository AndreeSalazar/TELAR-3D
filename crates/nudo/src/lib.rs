//! # nudo
//!
//! El núcleo de TELAR: la geometría (puntos, vértices, primitivas y
//! atributos, con el modelo de Houdini) y **el grafo que cocina solo lo que
//! cambia**. En `no_std`: solo `core` y `alloc`, incluso para `sin`, `cos` y
//! `sqrt`, que se escriben aquí ([`mate`]).
//!
//! ```
//! use nudo::{Geometria, Estadisticas};
//!
//! let mut g = Geometria::nueva();
//! let a = g.punto([0.0, 0.0, 0.0]);
//! let b = g.punto([1.0, 0.0, 0.0]);
//! let c = g.punto([0.0, 0.0, 1.0]);
//! g.poligono(&[a, b, c]);
//! let e = Estadisticas::de(&g);
//! assert_eq!((e.puntos, e.vertices, e.primitivas), (3, 3, 1));
//! ```

#![no_std]

extern crate alloc;

pub mod camara;
pub mod estadisticas;
pub mod geometria;
pub mod grafo;
pub mod malla;
pub mod mate;

pub use camara::{Camara, Mat4};
pub use estadisticas::Estadisticas;
pub use geometria::{Clase, Geometria, Valores};
pub use grafo::{Grafo, IdNodo, Nodo, Valor};
pub use malla::Malla;
