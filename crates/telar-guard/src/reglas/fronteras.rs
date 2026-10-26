//! `fronteras`: cada crate solo usa los crates que declara, y lo aislado
//! está aislado de verdad.
//!
//! Cada crate declara en su `Cargo.toml` a quién puede usar:
//!
//! ```toml
//! [package.metadata.telar]
//! puede-usar = ["telar-ventana", "telar-gpu"]
//! ```
//!
//! Sin esa sección, no puede usar ningún crate nuestro. Además, dos fronteras
//! no se negocian: el núcleo (`no_std`) solo usa el núcleo, y la ventana y la
//! GPU no se usan entre ellas ni usan nada (las une `telar-app`).

use crate::informe::Problema;
use crate::repo::Repo;
use crate::toml;

use super::deps::{es_de_dependencias, normalizar};
use super::nucleo::NUCLEO;

/// Aislados por completo: no dependen de ningún crate nuestro.
pub(crate) const AISLADOS: [&str; 2] = ["telar-ventana", "telar-gpu"];

pub(crate) fn revisar(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for nombre in repo.crates() {
        let archivo = format!("crates/{nombre}/Cargo.toml");
        let Some(texto) = repo.leer(&archivo) else {
            continue;
        };
        let entradas = toml::entradas(&texto);
        let permitidos: Vec<String> = entradas
            .iter()
            .find(|e| e.seccion == "package.metadata.telar" && e.clave == "puede-usar")
            .map(|e| lista(&e.valor))
            .unwrap_or_default();

        for e in entradas.iter().filter(|e| es_de_dependencias(&e.seccion)) {
            let Some(ruta) = toml::campo(&e.valor, "path") else {
                continue; // lo que no es por ruta ya lo juzga cero-deps
            };
            let Some(destino) = normalizar(&format!("crates/{nombre}/{ruta}")) else {
                continue;
            };
            let Some(otro) = destino.strip_prefix("crates/").and_then(|d| d.strip_suffix('/')) else {
                continue;
            };
            if let Some(motivo) = juzgar(&nombre, otro, &permitidos) {
                problemas.push(Problema::en(&archivo, e.linea, motivo));
            }
        }
    }
    problemas
}

/// `None` si `crate_` puede usar a `otro`; si no, por qué.
fn juzgar(crate_: &str, otro: &str, permitidos: &[String]) -> Option<String> {
    if AISLADOS.contains(&crate_) {
        return Some(format!("{crate_} está aislado por completo y no puede usar `{otro}` (los une telar-app)"));
    }
    if NUCLEO.contains(&crate_) && !NUCLEO.contains(&otro) {
        return Some(format!("{crate_} es del núcleo (no_std) y no puede usar `{otro}`, que no lo es"));
    }
    if permitidos.iter().any(|p| p == otro) {
        None
    } else {
        Some(format!("usa `{otro}` sin declararlo en [package.metadata.telar] puede-usar"))
    }
}

/// `["a", "b"]` → `[a, b]`.
fn lista(valor: &str) -> Vec<String> {
    valor
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}
