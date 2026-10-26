//! Los hooks de Claude Code, en Rust y dentro del guardián: nadie es root,
//! tampoco la IA.
//!
//! - `pre-bash`: deniega los comandos que se saltarían las reglas, antes de ejecutarlos.
//! - `post-edit`: tras cada edición, el guardián; si algo se rompe, la IA tiene que arreglarlo.
//! - `stop`: no se da el trabajo por terminado con reglas rotas o tests fallando.

use std::path::Path;

use crate::informe::fallos;
use crate::json::{Valor, cadena_json};
use crate::reglas;
use crate::repo::Repo;
use crate::verificar;

/// Si el comando rompe una regla, el motivo.
#[must_use]
pub fn comando_prohibido(comando: &str) -> Option<&'static str> {
    if comando.contains("--no-verify") {
        return Some("prohibido saltarse los hooks de git (--no-verify): arregla lo que el guardián señala");
    }
    if comando.contains("core.hooksPath") && !comando.contains("core.hooksPath .githooks") {
        return Some("prohibido cambiar core.hooksPath: los hooks de .githooks son obligatorios");
    }
    for tramo in comando.split([';', '&', '|', '(', ')', '\n', '`']) {
        let palabras: Vec<&str> = tramo.split_whitespace().collect();
        let Some(&programa) = palabras.first() else {
            continue;
        };
        let resto = &palabras[1..];
        match programa {
            "cargo" if resto.first().is_some_and(|w| *w == "add" || *w == "install") => {
                return Some("cero-deps: no se añaden crates de terceros; escríbelo como un crate en crates/");
            }
            "git" if resto.contains(&"commit") && resto.iter().any(|w| es_flag_corta(w, 'n')) => {
                return Some("prohibido `git commit -n` (es --no-verify)");
            }
            "git"
                if resto.contains(&"push")
                    && resto.iter().any(|w| w.starts_with("--force") || es_flag_corta(w, 'f')) =>
            {
                return Some("prohibido git push --force");
            }
            _ => {}
        }
    }
    None
}

/// `-n`, `-an`, `-nm`…: una agrupación de flags cortas que incluye `letra`.
fn es_flag_corta(palabra: &str, letra: char) -> bool {
    palabra.strip_prefix('-').is_some_and(|f| {
        !f.starts_with('-') && f.len() <= 4 && f.chars().all(|c| c.is_ascii_alphabetic()) && f.contains(letra)
    })
}

/// Ejecuta un hook a partir de su evento JSON. Devuelve lo que hay que
/// imprimir (vacío si no hay nada que decir).
#[must_use]
pub fn ejecutar(nombre: &str, evento: &Valor, raiz: &Path) -> String {
    match nombre {
        "pre-bash" => {
            let comando = evento.en(&["tool_input", "command"]).and_then(Valor::texto).unwrap_or_default();
            comando_prohibido(comando).map_or_else(String::new, |motivo| {
                format!(
                    "{{\"hookSpecificOutput\":{{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\"permissionDecisionReason\":{}}}}}",
                    cadena_json(&format!("[telar] {motivo}"))
                )
            })
        }
        "post-edit" => {
            let archivo = evento.en(&["tool_input", "file_path"]).and_then(Valor::texto).unwrap_or_default();
            let relevante = [".rs", ".toml", ".hilo", ".md"].iter().any(|ext| archivo.ends_with(ext));
            match revisar(raiz) {
                Some(informe) if relevante => {
                    bloquear(&format!("[telar-guard] Esta edición rompe reglas del repo:\n{informe}"))
                }
                _ => String::new(),
            }
        }
        "stop" => {
            if evento.en(&["stop_hook_active"]) == Some(&Valor::Bool(true)) {
                return String::new(); // ya se bloqueó una vez: no entrar en bucle
            }
            if let Some(informe) = revisar(raiz) {
                return bloquear(&format!("[telar] No se puede terminar con reglas rotas:\n{informe}"));
            }
            match verificar::tests(raiz) {
                Ok(()) => String::new(),
                Err(salida) => bloquear(&format!("[telar] No se puede terminar con tests fallando:\n{salida}")),
            }
        }
        _ => String::new(),
    }
}

/// El informe de fallos, o `None` si todo está en orden.
fn revisar(raiz: &Path) -> Option<String> {
    let repo = Repo::abrir(raiz).ok()?;
    let resultados = reglas::revisar(&repo);
    resultados.iter().any(|r| !r.problemas.is_empty()).then(|| fallos(&resultados))
}

fn bloquear(motivo: &str) -> String {
    format!("{{\"decision\":\"block\",\"reason\":{}}}", cadena_json(motivo))
}
