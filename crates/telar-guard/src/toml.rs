//! Lo justo de TOML para leer los `Cargo.toml`: secciones y pares
//! `clave = valor` de una línea (con tablas en línea `{ … }`). Las líneas que
//! no son ni lo uno ni lo otro (el interior de un array largo) se ignoran.

/// Un par `clave = valor` con su sección y su línea.
pub(crate) struct Entrada {
    /// La sección en la que está: `dependencies`, `target.'cfg(windows)'.dependencies`…
    pub(crate) seccion: String,
    /// Línea de la cabecera de la sección (0 si no hay).
    pub(crate) linea_seccion: usize,
    pub(crate) clave: String,
    /// El valor en crudo, sin el comentario final.
    pub(crate) valor: String,
    pub(crate) linea: usize,
}

/// Todas las entradas del archivo, en orden.
pub(crate) fn entradas(texto: &str) -> Vec<Entrada> {
    let mut seccion = String::new();
    let mut linea_seccion = 0;
    let mut out = Vec::new();
    for (n, crudo) in texto.lines().enumerate() {
        let linea = sin_comentario(crudo).trim();
        if let Some(cabecera) = linea.strip_prefix('[') {
            seccion.clear();
            seccion.push_str(cabecera.trim_matches(|c| c == '[' || c == ']').trim());
            linea_seccion = n + 1;
            continue;
        }
        let Some((clave, valor)) = linea.split_once('=') else {
            continue;
        };
        let clave = clave.trim().trim_matches('"').to_owned();
        if clave.is_empty() || clave.contains(char::is_whitespace) {
            continue;
        }
        out.push(Entrada {
            seccion: seccion.clone(),
            linea_seccion,
            clave,
            valor: valor.trim().to_owned(),
            linea: n + 1,
        });
    }
    out
}

/// Quita el comentario `# …` final, respetando las `#` dentro de comillas.
fn sin_comentario(linea: &str) -> &str {
    let mut dentro = false;
    for (i, ch) in linea.char_indices() {
        match ch {
            '"' => dentro = !dentro,
            '#' if !dentro => return &linea[..i],
            _ => {}
        }
    }
    linea
}

/// En una tabla en línea `{ path = "../x", version = "1" }`, el valor de una
/// clave (sin comillas).
pub(crate) fn campo<'a>(tabla: &'a str, clave: &str) -> Option<&'a str> {
    let dentro = tabla.trim().strip_prefix('{')?.strip_suffix('}')?;
    dentro.split(',').find_map(|par| {
        let (k, v) = par.split_once('=')?;
        (k.trim() == clave).then(|| v.trim().trim_matches('"'))
    })
}
