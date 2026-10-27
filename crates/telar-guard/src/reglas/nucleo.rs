//! `nucleo-sin-std`: el núcleo no depende de ningún sistema operativo.

use crate::escaner::{blanquear, buscar_seguida};
use crate::informe::Problema;
use crate::repo::Repo;

/// Los crates del núcleo: `core` y `alloc`, nada más. Así corren en Windows,
/// en Linux y algún día en BMO-X sin cambiar una línea.
pub(crate) const NUCLEO: [&str; 5] = ["nudo", "hilo", "hilo-spirv", "telar-nodos", "telar-kernel"];

pub(crate) fn revisar(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for nombre in repo.crates().iter().filter(|c| NUCLEO.contains(&c.as_str())) {
        let raiz = format!("crates/{nombre}/src/lib.rs");
        match repo.leer(&raiz) {
            Some(src) if blanquear(&src).contains("#![no_std]") => {}
            Some(_) => problemas.push(Problema::en(&raiz, 1, "falta `#![no_std]`")),
            None => problemas.push(Problema::en(&raiz, 0, "un crate del núcleo es una biblioteca: falta src/lib.rs")),
        }
        let prefijo = format!("crates/{nombre}/src/");
        for archivo in repo.rust_en(&prefijo) {
            let Some(src) = repo.leer(archivo) else {
                continue;
            };
            let codigo = blanquear(&src);
            for n in buscar_seguida(&codigo, "std", "::") {
                problemas.push(Problema::en(archivo, n, "usa `std::` (en el núcleo, solo `core::` y `alloc::`)"));
            }
            for n in buscar_seguida(&codigo, "crate", "std") {
                problemas.push(Problema::en(archivo, n, "`extern crate std` en el núcleo"));
            }
        }
    }
    problemas
}
