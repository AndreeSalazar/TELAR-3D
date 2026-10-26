//! `unsafe-acotado` y `ffi-aislada`: lo peligroso, en un solo sitio y justificado.

use crate::escaner::{blanquear, buscar_palabra, buscar_seguida};
use crate::informe::Problema;
use crate::repo::Repo;

/// ¿Está en una zona `ffi/` (`crates/<crate>/src/ffi/…`)?
pub(crate) fn en_ffi(archivo: &str) -> bool {
    archivo
        .strip_prefix("crates/")
        .and_then(|r| r.split_once("/src/ffi/"))
        .is_some_and(|(c, _)| !c.contains('/'))
}

/// Lo que habla directamente con el sistema: solo puede vivir en `ffi/`.
const LLAMADAS_SISTEMA: [&str; 5] = [
    "LoadLibraryW",
    "LoadLibraryA",
    "GetProcAddress",
    "dlopen",
    "dlsym",
];

pub(crate) fn unsafe_acotado(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for archivo in repo.rust_en("crates/") {
        let Some(src) = repo.leer(archivo) else {
            continue;
        };
        let codigo = blanquear(&src);
        let lineas: Vec<&str> = src.lines().collect();
        if en_ffi(archivo) {
            for (n, _) in buscar_palabra(&codigo, "unsafe") {
                if !justificado(&lineas, n) {
                    problemas.push(Problema::en(
                        archivo,
                        n,
                        "`unsafe` sin `// SAFETY:` justo encima (ni `/// # Safety` en su función)",
                    ));
                }
            }
        } else {
            for (n, _) in buscar_palabra(&codigo, "unsafe") {
                problemas.push(Problema::en(archivo, n, "`unsafe` fuera de src/ffi/"));
            }
            for (n, _) in buscar_palabra(&codigo, "unsafe_code") {
                problemas.push(Problema::en(
                    archivo,
                    n,
                    "permite `unsafe_code` fuera de src/ffi/",
                ));
            }
        }
    }
    problemas
}

pub(crate) fn ffi_aislada(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for archivo in repo.rust_en("crates/").filter(|f| !en_ffi(f)) {
        let Some(src) = repo.leer(archivo) else {
            continue;
        };
        let codigo = blanquear(&src);
        for n in buscar_seguida(&codigo, "extern", "\"") {
            problemas.push(Problema::en(
                archivo,
                n,
                "declaración `extern \"…\"` fuera de src/ffi/",
            ));
        }
        for (n, linea) in codigo.lines().enumerate() {
            if linea.contains("#[link") {
                problemas.push(Problema::en(archivo, n + 1, "`#[link]` fuera de src/ffi/"));
            }
        }
        for nombre in LLAMADAS_SISTEMA {
            for (n, _) in buscar_palabra(&codigo, nombre) {
                problemas.push(Problema::en(
                    archivo,
                    n,
                    format!("`{nombre}` fuera de src/ffi/"),
                ));
            }
        }
    }
    problemas
}

/// ¿El `unsafe` de la línea `n` (desde 1) lleva su justificación?
///
/// - En la misma línea, un `// SAFETY:` al final.
/// - Encima, en el bloque de comentarios pegado (los atributos `#[…]` no
///   cuentan como separación), una línea que empieza por `// SAFETY:`.
/// - Si es una `unsafe fn`, también vale `/// # Safety` en su documentación.
pub(crate) fn justificado(lineas: &[&str], n: usize) -> bool {
    let actual = lineas.get(n - 1).copied().unwrap_or_default();
    if actual.contains("// SAFETY:") {
        return true;
    }
    let es_fn = actual
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w == ["unsafe", "fn"]);
    for linea in lineas[..n - 1].iter().rev().map(|l| l.trim()) {
        if linea.starts_with("#[") || linea.starts_with("#![") {
            continue;
        }
        if linea.starts_with("// SAFETY:") || (es_fn && linea.starts_with("/// # Safety")) {
            return true;
        }
        if !linea.starts_with("//") {
            return false;
        }
    }
    false
}
