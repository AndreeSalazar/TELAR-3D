//! `nodo-completo`: cada nodo de `telar-nodos` se explica y se demuestra solo.
//!
//! Un nodo es una carpeta `crates/telar-nodos/src/<nodo>/`, y tiene que tener:
//! - su código, `mod.rs`;
//! - su libreto, `libreto.md` (qué hace, la matemática y de dónde sale);
//! - su test, `crates/telar-nodos/tests/<nodo>.rs`;
//! - su comparación con Houdini, `referencias/<nodo>.md`.

use std::collections::BTreeSet;

use crate::informe::Problema;
use crate::repo::Repo;

const NODOS: &str = "crates/telar-nodos/src/";

pub(crate) fn revisar(repo: &Repo) -> Vec<Problema> {
    let nodos: BTreeSet<&str> =
        repo.archivos.iter().filter_map(|f| f.strip_prefix(NODOS)?.split_once('/').map(|(nodo, _)| nodo)).collect();
    let mut problemas = Vec::new();
    for nodo in nodos {
        let carpeta = format!("{NODOS}{nodo}/");
        let requisitos = [
            (format!("{carpeta}mod.rs"), "falta su código"),
            (format!("{carpeta}libreto.md"), "falta su libreto (qué hace, la matemática y de dónde sale)"),
            (format!("crates/telar-nodos/tests/{nodo}.rs"), "falta su test"),
            (format!("referencias/{nodo}.md"), "falta su comparación con Houdini"),
        ];
        for (archivo, motivo) in requisitos {
            if !repo.existe(&archivo) {
                problemas.push(Problema::en(&carpeta, 0, format!("{motivo}: {archivo}")));
            }
        }
    }
    problemas
}
