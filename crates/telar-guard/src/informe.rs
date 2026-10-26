//! Reglas, problemas y el informe que se imprime.

use crate::repo::Repo;

/// Una regla del repositorio.
pub struct Regla {
    /// Identificador estable, en minúsculas: `unsafe-acotado`.
    pub id: &'static str,
    /// Qué exige, en una línea.
    pub titulo: &'static str,
    /// Por qué existe.
    pub motivo: &'static str,
    /// Cómo se arregla cuando falla.
    pub arreglo: &'static str,
    /// La comprobación.
    pub revisar: fn(&Repo) -> Vec<Problema>,
}

/// Algo que rompe una regla, con su sitio exacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problema {
    /// Archivo relativo a la raíz.
    pub archivo: String,
    /// Línea (desde 1), o 0 si es el archivo entero.
    pub linea: usize,
    /// Qué pasa ahí.
    pub detalle: String,
}

impl Problema {
    /// Un problema en una línea concreta.
    #[must_use]
    pub fn en(archivo: &str, linea: usize, detalle: impl Into<String>) -> Self {
        Self { archivo: archivo.to_owned(), linea, detalle: detalle.into() }
    }

    /// `archivo:línea` (o solo el archivo).
    #[must_use]
    pub fn sitio(&self) -> String {
        if self.linea == 0 { self.archivo.clone() } else { format!("{}:{}", self.archivo, self.linea) }
    }
}

/// El resultado de una regla sobre el repo.
pub struct Resultado {
    /// La regla.
    pub regla: &'static Regla,
    /// Lo que la rompe (vacío si se cumple).
    pub problemas: Vec<Problema>,
}

/// El informe para la consola: una línea por regla y, debajo de cada regla
/// rota, sus problemas y el arreglo.
#[must_use]
pub fn texto(resultados: &[Resultado], archivos: usize) -> String {
    let mut lineas = vec![format!("telar-guard · {} reglas · {archivos} archivos", resultados.len())];
    for r in resultados {
        let marca = if r.problemas.is_empty() { '✔' } else { '✖' };
        lineas.push(format!("  {marca} {:<16} {}", r.regla.id, r.regla.titulo));
        lineas.extend(r.problemas.iter().map(|p| format!("      {}  {}", p.sitio(), p.detalle)));
        if !r.problemas.is_empty() {
            lineas.push(format!("      → {}", r.regla.arreglo));
        }
    }
    let rotas = resultados.iter().filter(|r| !r.problemas.is_empty()).count();
    lineas.push(String::new());
    lineas.push(if rotas == 0 {
        "✔ todo en orden".to_owned()
    } else {
        format!("✖ {rotas} {}", if rotas == 1 { "regla rota" } else { "reglas rotas" })
    });
    lineas.join("\n") + "\n"
}

/// Solo lo que falla (para los hooks: sin las líneas en verde).
#[must_use]
pub fn fallos(resultados: &[Resultado]) -> String {
    texto(resultados, 0)
        .lines()
        .filter(|l| !l.trim_start().starts_with('✔') && !l.starts_with("telar-guard ·"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}
