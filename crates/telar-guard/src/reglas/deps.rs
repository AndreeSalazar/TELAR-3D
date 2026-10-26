//! `cero-deps`: en los `Cargo.toml`, solo dependencias por ruta dentro de `crates/`.

use std::collections::BTreeMap;

use crate::informe::Problema;
use crate::repo::Repo;
use crate::toml;

/// ¿Es una sección de dependencias? `dependencies`, `dev-dependencies`,
/// `build-dependencies`, `workspace.dependencies` o la de un `target`.
fn es_de_dependencias(seccion: &str) -> bool {
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .any(|d| seccion == *d || seccion.ends_with(&format!(".{d}")))
}

pub(crate) fn revisar(repo: &Repo) -> Vec<Problema> {
    let mut problemas = Vec::new();
    for archivo in repo.archivos.iter().filter(|f| f.ends_with("Cargo.toml")) {
        let Some(texto) = repo.leer(archivo) else {
            continue;
        };
        let dir = archivo.strip_suffix("Cargo.toml").unwrap_or_default();
        // La forma de tabla, `[dependencies.foo]`: se junta cada crate con sus campos.
        let mut tablas: BTreeMap<String, (usize, Option<String>, bool)> = BTreeMap::new();
        for e in toml::entradas(&texto) {
            if es_de_dependencias(&e.seccion) {
                revisar_entrada(archivo, dir, &e.clave, &e.valor, e.linea, &mut problemas);
            } else if let Some((seccion, _)) = e.seccion.rsplit_once('.')
                && es_de_dependencias(seccion)
            {
                let tabla =
                    tablas
                        .entry(e.seccion.clone())
                        .or_insert((e.linea_seccion, None, false));
                match e.clave.as_str() {
                    "path" => tabla.1 = Some(e.valor.trim_matches('"').to_owned()),
                    "workspace" => tabla.2 = e.valor == "true",
                    _ => {}
                }
            }
        }
        for (seccion, (linea, ruta, workspace)) in tablas {
            let nombre = seccion
                .rsplit_once('.')
                .map_or(seccion.as_str(), |(_, n)| n);
            if let Some(motivo) = juzgar(dir, ruta.as_deref(), workspace) {
                problemas.push(Problema::en(
                    archivo,
                    linea,
                    format!("`{nombre}`: {motivo}"),
                ));
            }
        }
    }
    problemas
}

fn revisar_entrada(
    archivo: &str,
    dir: &str,
    nombre: &str,
    valor: &str,
    linea: usize,
    out: &mut Vec<Problema>,
) {
    let motivo = if valor.starts_with('{') {
        juzgar(
            dir,
            toml::campo(valor, "path"),
            toml::campo(valor, "workspace") == Some("true"),
        )
    } else {
        Some("es un crate de terceros (una versión de crates.io)".to_owned())
    };
    if let Some(motivo) = motivo {
        out.push(Problema::en(
            archivo,
            linea,
            format!("`{nombre}`: {motivo}"),
        ));
    }
}

/// `None` si la dependencia es nuestra; si no, por qué no lo es.
fn juzgar(dir: &str, ruta: Option<&str>, workspace: bool) -> Option<String> {
    if workspace {
        return None; // la decide [workspace.dependencies], que pasa por esta misma regla
    }
    let Some(ruta) = ruta else {
        return Some("no tiene `path`: viene de crates.io o de git".to_owned());
    };
    match normalizar(&format!("{dir}{ruta}")) {
        Some(destino) if destino.starts_with("crates/") => None,
        _ => Some(format!("`path = \"{ruta}\"` apunta fuera de crates/")),
    }
}

/// `crates/a/../b` → `crates/b`. `None` si se sale de la raíz.
fn normalizar(ruta: &str) -> Option<String> {
    let mut partes: Vec<&str> = Vec::new();
    for parte in ruta.split(['/', '\\']) {
        match parte {
            "" | "." => {}
            ".." => {
                partes.pop()?;
            }
            p => partes.push(p),
        }
    }
    Some(partes.join("/") + "/")
}
