//! Lo justo de JSON para los hooks de Claude Code: leer el evento que llega
//! por la entrada estándar y escribir la respuesta.

use std::fmt::Write;

/// Un valor JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    /// `null`
    Nulo,
    /// `true` / `false`
    Bool(bool),
    /// Un número, guardado tal cual (los hooks no hacen cuentas).
    Numero(String),
    /// Una cadena, ya sin escapes.
    Texto(String),
    /// `[…]`
    Lista(Vec<Valor>),
    /// `{…}`, en el orden del texto.
    Objeto(Vec<(String, Valor)>),
}

impl Valor {
    /// Sigue una ruta de claves: `["tool_input", "command"]`.
    #[must_use]
    pub fn en(&self, ruta: &[&str]) -> Option<&Valor> {
        ruta.iter().try_fold(self, |v, clave| match v {
            Valor::Objeto(pares) => pares.iter().find(|(k, _)| k == clave).map(|(_, v)| v),
            _ => None,
        })
    }

    /// El texto, si es una cadena.
    #[must_use]
    pub fn texto(&self) -> Option<&str> {
        match self {
            Valor::Texto(t) => Some(t),
            _ => None,
        }
    }
}

/// Lee un documento JSON entero.
///
/// # Errors
///
/// Si no es JSON válido, con la posición del primer error.
pub fn leer(texto: &str) -> Result<Valor, String> {
    let c: Vec<char> = texto.chars().collect();
    let mut i = 0;
    let v = valor(&c, &mut i)?;
    espacios(&c, &mut i);
    if i == c.len() { Ok(v) } else { Err(format!("JSON: sobra texto en la posición {i}")) }
}

fn espacios(c: &[char], i: &mut usize) {
    while c.get(*i).is_some_and(|ch| ch.is_whitespace()) {
        *i += 1;
    }
}

fn valor(c: &[char], i: &mut usize) -> Result<Valor, String> {
    espacios(c, i);
    match c.get(*i) {
        Some('{') => objeto(c, i),
        Some('[') => lista(c, i),
        Some('"') => cadena(c, i).map(Valor::Texto),
        Some('t') => palabra(c, i, "true", Valor::Bool(true)),
        Some('f') => palabra(c, i, "false", Valor::Bool(false)),
        Some('n') => palabra(c, i, "null", Valor::Nulo),
        Some(ch) if *ch == '-' || ch.is_ascii_digit() => {
            let inicio = *i;
            while c.get(*i).is_some_and(|ch| ch.is_ascii_digit() || "+-.eE".contains(*ch)) {
                *i += 1;
            }
            Ok(Valor::Numero(c[inicio..*i].iter().collect()))
        }
        _ => Err(format!("JSON: valor inesperado en la posición {i}")),
    }
}

fn palabra(c: &[char], i: &mut usize, p: &str, v: Valor) -> Result<Valor, String> {
    let fin = *i + p.chars().count();
    if c.get(*i..fin).is_some_and(|s| s.iter().copied().eq(p.chars())) {
        *i = fin;
        Ok(v)
    } else {
        Err(format!("JSON: se esperaba `{p}` en la posición {i}"))
    }
}

fn cadena(c: &[char], i: &mut usize) -> Result<String, String> {
    *i += 1; // la comilla de apertura
    let mut out = String::new();
    while let Some(&ch) = c.get(*i) {
        *i += 1;
        match ch {
            '"' => return Ok(out),
            '\\' => {
                let esc = c.get(*i).copied().ok_or("JSON: escape cortado")?;
                *i += 1;
                match esc {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'u' => {
                        let hex: String = c.get(*i..*i + 4).ok_or("JSON: \\u cortado")?.iter().collect();
                        *i += 4;
                        let code = u32::from_str_radix(&hex, 16).map_err(|e| format!("JSON: \\u{hex}: {e}"))?;
                        out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                    }
                    otro => out.push(otro), // \" \\ \/
                }
            }
            otro => out.push(otro),
        }
    }
    Err("JSON: cadena sin cerrar".to_owned())
}

fn lista(c: &[char], i: &mut usize) -> Result<Valor, String> {
    *i += 1;
    let mut items = Vec::new();
    loop {
        espacios(c, i);
        if c.get(*i) == Some(&']') {
            *i += 1;
            return Ok(Valor::Lista(items));
        }
        items.push(valor(c, i)?);
        espacios(c, i);
        if c.get(*i) == Some(&',') {
            *i += 1;
        }
    }
}

fn objeto(c: &[char], i: &mut usize) -> Result<Valor, String> {
    *i += 1;
    let mut pares = Vec::new();
    loop {
        espacios(c, i);
        match c.get(*i) {
            Some('}') => {
                *i += 1;
                return Ok(Valor::Objeto(pares));
            }
            Some('"') => {}
            _ => return Err(format!("JSON: se esperaba una clave en la posición {i}")),
        }
        let clave = cadena(c, i)?;
        espacios(c, i);
        if c.get(*i) != Some(&':') {
            return Err(format!("JSON: falta `:` en la posición {i}"));
        }
        *i += 1;
        pares.push((clave, valor(c, i)?));
        espacios(c, i);
        if c.get(*i) == Some(&',') {
            *i += 1;
        }
    }
}

/// Una cadena JSON con sus comillas y sus escapes.
#[must_use]
pub fn cadena_json(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
