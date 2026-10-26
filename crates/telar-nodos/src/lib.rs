//! # telar-nodos
//!
//! La biblioteca de nodos de TELAR: **una carpeta por nodo**, con su código
//! (`mod.rs`), su `libreto.md` (qué hace, la matemática y de dónde sale), su
//! test (`tests/<nodo>.rs`) y su comparación con Houdini
//! (`referencias/<nodo>.md`). Lo exige la regla `nodo-completo`.
//!
//! ```
//! use nudo::{Grafo, Estadisticas};
//! use telar_nodos::{Caja, Transformar};
//!
//! let mut grafo = Grafo::nuevo();
//! let caja = grafo.anadir(Caja::default(), &[])?;
//! let mover = grafo.anadir(Transformar { traslacion: [0.0, 2.0, 0.0], ..Transformar::default() }, &[caja])?;
//! let e = Estadisticas::de(grafo.cocinar(mover)?);
//! assert_eq!(e.centro, Some([0.0, 2.0, 0.0]));
//! # Ok::<(), String>(())
//! ```
//!
//! Es del núcleo (`no_std`): solo usa `nudo`, `core` y `alloc`.

#![no_std]

extern crate alloc;

pub mod caja;
pub mod esfera;
pub mod parametros;
pub mod rejilla;
pub mod transformar;
pub mod unir;

pub use caja::Caja;
pub use esfera::Esfera;
pub use rejilla::Rejilla;
pub use transformar::Transformar;
pub use unir::Unir;
