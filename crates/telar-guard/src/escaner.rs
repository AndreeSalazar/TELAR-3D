//! Lectura de código Rust sin un analizador completo.
//!
//! [`blanquear`] sustituye por espacios los comentarios y el interior de los
//! literales de texto, conservando los saltos de línea: la línea N del
//! resultado es la línea N del original. Así las reglas buscan `unsafe`,
//! `dbg!` o `std::` solo en código de verdad, nunca en un comentario o en un
//! mensaje de error.

/// Devuelve `src` con los comentarios y el contenido de las cadenas, de los
/// caracteres y de las cadenas en bruto sustituidos por espacios. Las comillas
/// se conservan (`extern "system"` queda como `extern "      "`).
#[must_use]
pub fn blanquear(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < c.len() {
        let siguiente = c.get(i + 1).copied();
        if c[i] == '/' && siguiente == Some('/') {
            while i < c.len() && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c[i] == '/' && siguiente == Some('*') {
            i = bloque(&c, i, &mut out);
        } else if let Some(hashes) = cadena_en_bruto(&c, i) {
            i = en_bruto(&c, i, hashes, &mut out);
        } else if c[i] == '"' {
            i = cadena(&c, i, &mut out);
        } else if c[i] == '\'' {
            i = caracter(&c, i, &mut out);
        } else {
            out.push(c[i]);
            i += 1;
        }
    }
    out
}

/// Un hueco en blanco que conserva los saltos de línea.
fn hueco(ch: char, out: &mut String) {
    out.push(if ch == '\n' { '\n' } else { ' ' });
}

/// `/* … */`, con anidamiento (en Rust los comentarios de bloque se anidan).
fn bloque(c: &[char], mut i: usize, out: &mut String) -> usize {
    let mut nivel = 0usize;
    while i < c.len() {
        let siguiente = c.get(i + 1).copied();
        if c[i] == '/' && siguiente == Some('*') {
            nivel += 1;
            out.push_str("  ");
            i += 2;
        } else if c[i] == '*' && siguiente == Some('/') {
            nivel -= 1;
            out.push_str("  ");
            i += 2;
            if nivel == 0 {
                break;
            }
        } else {
            hueco(c[i], out);
            i += 1;
        }
    }
    i
}

/// ¿Empieza en `i` una cadena en bruto (`r"…"`, `r#"…"#`, `br"…"`, `cr"…"`)?
/// Devuelve cuántas `#` lleva.
fn cadena_en_bruto(c: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    if matches!(c.get(j), Some('b' | 'c')) && c.get(j + 1) == Some(&'r') {
        j += 1;
    }
    if c.get(j) != Some(&'r') {
        return None;
    }
    if i > 0 && es_ident(c[i - 1]) {
        return None; // `bar"…"` no existe; `foor#…` tampoco: es parte de otro nombre
    }
    j += 1;
    let mut hashes = 0;
    while c.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    (c.get(j) == Some(&'"')).then_some(hashes)
}

fn en_bruto(c: &[char], mut i: usize, hashes: usize, out: &mut String) -> usize {
    // El prefijo (b, c, r y las #) y la comilla de apertura, tal cual.
    while c[i] != '"' {
        out.push(c[i]);
        i += 1;
    }
    out.push('"');
    i += 1;
    while i < c.len() {
        if c[i] == '"' && (1..=hashes).all(|k| c.get(i + k) == Some(&'#')) {
            out.push('"');
            out.extend(std::iter::repeat_n('#', hashes));
            return i + 1 + hashes;
        }
        hueco(c[i], out);
        i += 1;
    }
    i
}

fn cadena(c: &[char], mut i: usize, out: &mut String) -> usize {
    out.push('"');
    i += 1;
    while i < c.len() {
        match c[i] {
            '\\' => {
                hueco(c[i], out);
                if let Some(&escapado) = c.get(i + 1) {
                    hueco(escapado, out);
                }
                i += 2;
            }
            '"' => {
                out.push('"');
                return i + 1;
            }
            ch => {
                hueco(ch, out);
                i += 1;
            }
        }
    }
    i
}

/// `'a'`, `'\n'`, `'\u{1F600}'` y `'\''` son caracteres; `'a` es un lifetime
/// (o una etiqueta) y se deja tal cual.
fn caracter(c: &[char], i: usize, out: &mut String) -> usize {
    let fin = if c.get(i + 1) == Some(&'\\') {
        (i + 3..c.len().min(i + 14)).find(|&j| c[j] == '\'')
    } else if c.get(i + 2) == Some(&'\'') {
        Some(i + 2)
    } else {
        None
    };
    out.push('\'');
    let Some(fin) = fin else { return i + 1 };
    for &ch in &c[i + 1..fin] {
        hueco(ch, out);
    }
    out.push('\'');
    fin + 1
}

/// ¿Puede formar parte de un identificador de Rust?
#[must_use]
pub fn es_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Posiciones (línea desde 1, y columna en bytes) donde aparece `palabra` como
/// palabra completa: sin letras, cifras ni `_` pegadas delante o detrás.
#[must_use]
pub fn buscar_palabra(codigo: &str, palabra: &str) -> Vec<(usize, usize)> {
    let mut hallazgos = Vec::new();
    for (n, linea) in codigo.lines().enumerate() {
        let mut desde = 0;
        while let Some(pos) = linea[desde..].find(palabra) {
            let inicio = desde + pos;
            let fin = inicio + palabra.len();
            let antes = linea[..inicio].chars().next_back();
            let despues = linea[fin..].chars().next();
            if !antes.is_some_and(es_ident) && !despues.is_some_and(es_ident) {
                hallazgos.push((n + 1, inicio));
            }
            desde = fin;
        }
    }
    hallazgos
}

/// Como [`buscar_palabra`], pero la palabra tiene que ir seguida (tras
/// espacios opcionales) de `sufijo`: `dbg` + `!`, `std` + `::`, `extern` + `"`.
#[must_use]
pub fn buscar_seguida(codigo: &str, palabra: &str, sufijo: &str) -> Vec<usize> {
    let lineas: Vec<&str> = codigo.lines().collect();
    buscar_palabra(codigo, palabra)
        .into_iter()
        .filter(|&(n, col)| lineas[n - 1][col + palabra.len()..].trim_start().starts_with(sufijo))
        .map(|(n, _)| n)
        .collect()
}
