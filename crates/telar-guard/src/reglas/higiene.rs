//! `tamano`, `sin-restos`, `lf` y `crate-completo`: el orden de la casa.

use crate::escaner::{blanquear, buscar_seguida};
use crate::informe::Problema;
use crate::repo::Repo;

/// El máximo de líneas de un archivo de código.
pub(crate) const MAX_LINEAS: usize = 400;

/// Archivos de código: los que cuentan para `tamano`.
const CODIGO: [&str; 5] = [".rs", ".hilo", ".wgsl", ".sh", ".ps1"];

/// Archivos de texto: los que tienen que ir con saltos LF.
const TEXTO: [&str; 12] =
    [".rs", ".toml", ".md", ".hilo", ".wgsl", ".sh", ".ps1", ".json", ".txt", ".yml", ".gitignore", ".gitattributes"];

pub(crate) fn tamano(repo: &Repo) -> Vec<Problema> {
    repo.archivos
        .iter()
        .filter(|f| CODIGO.iter().any(|ext| f.ends_with(ext)))
        .filter_map(|f| {
            let lineas = repo.leer(f)?.lines().count();
            (lineas > MAX_LINEAS).then(|| {
                Problema::en(f, 0, format!("{lineas} líneas (máximo {MAX_LINEAS}): divide por responsabilidades"))
            })
        })
        .collect()
}

pub(crate) fn sin_restos(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for archivo in repo.rust_en("crates/") {
        let Some(src) = repo.leer(archivo) else {
            continue;
        };
        let codigo = blanquear(&src);
        for macro_ in ["dbg", "todo", "unimplemented"] {
            for n in buscar_seguida(&codigo, macro_, "!") {
                problemas.push(Problema::en(archivo, n, format!("`{macro_}!`")));
            }
        }
        for (n, linea) in codigo.lines().enumerate() {
            let compacta: String = linea.chars().filter(|c| !c.is_whitespace()).collect();
            if compacta.contains("#[ignore]") {
                problemas.push(Problema::en(archivo, n + 1, "`#[ignore]` sin motivo (usa `#[ignore = \"por qué\"]`)"));
            }
        }
    }
    problemas
}

pub(crate) fn lf(repo: &Repo) -> Vec<Problema> {
    repo.archivos
        .iter()
        .filter(|f| f.starts_with(".githooks/") || TEXTO.iter().any(|ext| f.ends_with(ext)))
        .filter_map(|f| {
            let bytes = repo.bytes(f)?;
            let n = bytes.windows(2).position(|w| w == b"\r\n")?;
            let linea = bytes[..n].split(|&b| b == b'\n').count();
            Some(Problema::en(f, linea, "salto de línea CRLF"))
        })
        .collect()
}

pub(crate) fn crate_completo(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for nombre in repo.crates() {
        let base = format!("crates/{nombre}/");
        if !repo.existe(&format!("{base}README.md")) {
            problemas.push(Problema::en(&base, 0, "falta README.md"));
        }
        let tests_aparte = repo.rust_en(&format!("{base}tests/")).next().is_some();
        let tests_dentro = repo
            .rust_en(&format!("{base}src/"))
            .any(|f| repo.leer(f).is_some_and(|src| blanquear(&src).contains("#[test]")));
        if !tests_aparte && !tests_dentro {
            problemas.push(Problema::en(&base, 0, "no tiene tests (ni tests/*.rs ni #[test] en src/)"));
        }
    }
    problemas
}
