//! Mensajes de commit convencionales: `tipo(ámbito): descripción`.

/// Los tipos permitidos.
pub const TIPOS: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// Comprueba la primera línea útil del mensaje (las que empiezan por `#` son
/// comentarios de git).
///
/// # Errors
///
/// Si no sigue el formato, con una explicación y un ejemplo.
pub fn validar(mensaje: &str) -> Result<(), String> {
    let Some(primera) = mensaje
        .lines()
        .map(str::trim_end)
        .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
    else {
        return Err("el mensaje está vacío".to_owned());
    };
    // Los que escribe git solo (fusiones y reversiones) pasan tal cual.
    if primera.starts_with("Merge ") || primera.starts_with("Revert \"") {
        return Ok(());
    }
    if cumple(primera) {
        Ok(())
    } else {
        Err(format!(
            "el mensaje no sigue el formato convencional:\n  {primera}\n\
             Formato: tipo(ámbito): descripción, con tipo en {}\n\
             Ejemplo: feat(telar-guard): regla nucleo-sin-std",
            TIPOS.join(", ")
        ))
    }
}

fn cumple(linea: &str) -> bool {
    let Some(tipo) = TIPOS.iter().find(|t| linea.starts_with(**t)) else {
        return false;
    };
    let mut resto = &linea[tipo.len()..];
    if let Some(dentro) = resto.strip_prefix('(') {
        let Some((ambito, despues)) = dentro.split_once(')') else {
            return false;
        };
        let valido = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || "/_-".contains(c);
        if ambito.is_empty() || !ambito.chars().all(valido) {
            return false;
        }
        resto = despues;
    }
    resto = resto.strip_prefix('!').unwrap_or(resto);
    resto
        .strip_prefix(": ")
        .is_some_and(|d| d.trim().chars().count() >= 3)
}
