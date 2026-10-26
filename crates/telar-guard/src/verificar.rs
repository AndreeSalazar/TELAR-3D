//! `cargo verify`: todo lo que tiene que pasar antes de dar algo por terminado.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use crate::informe::texto;
use crate::reglas;
use crate::repo::Repo;

/// El `cargo` con el que nos lanzaron (o el del `PATH`).
fn cargo() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

/// Ejecuta un comando en la raíz y devuelve su salida si falla.
fn correr(raiz: &Path, programa: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Result<(), String> {
    let salida = Command::new(programa)
        .args(args)
        .current_dir(raiz)
        .output()
        .map_err(|e| format!("no se pudo ejecutar: {e}"))?;
    if salida.status.success() {
        Ok(())
    } else {
        let mut texto = String::from_utf8_lossy(&salida.stdout).into_owned();
        texto += &String::from_utf8_lossy(&salida.stderr);
        Err(texto.trim().to_owned())
    }
}

/// `cargo test` de todo el workspace.
///
/// # Errors
///
/// Con la salida de los tests si alguno falla.
pub fn tests(raiz: &Path) -> Result<(), String> {
    correr(raiz, cargo(), &["test", "--workspace", "-q"])
}

/// Guardián, rustfmt, clippy pedantic y tests, en ese orden; para en el
/// primero que falla. Antes, activa los hooks de git (`.githooks`).
#[must_use]
pub fn verify(raiz: &Path) -> bool {
    if raiz.join(".git").exists() {
        match correr(raiz, "git", &["config", "core.hooksPath", ".githooks"]) {
            Ok(()) => println!("telar: hooks de git activos (.githooks)"),
            Err(e) => println!("telar: no pude activar los hooks de git: {e}"),
        }
    }

    println!("telar: guardián…");
    match Repo::abrir(raiz) {
        Ok(repo) => {
            let resultados = reglas::revisar(&repo);
            if resultados.iter().any(|r| !r.problemas.is_empty()) {
                print!("{}", texto(&resultados, repo.archivos.len()));
                return false;
            }
        }
        Err(e) => {
            println!("✖ no pude leer el repo: {e}");
            return false;
        }
    }

    let pasos: [(&str, &[&str]); 3] = [
        ("rustfmt", &["fmt", "--all", "--check"]),
        ("clippy pedantic", &["clippy", "--workspace", "--all-targets", "-q", "--", "-D", "warnings"]),
        ("tests", &["test", "--workspace", "-q"]),
    ];
    for (nombre, args) in pasos {
        println!("telar: {nombre}…");
        if let Err(salida) = correr(raiz, cargo(), args) {
            println!("{salida}\n✖ {nombre} falla");
            return false;
        }
    }
    println!("✔ todo en orden");
    true
}
