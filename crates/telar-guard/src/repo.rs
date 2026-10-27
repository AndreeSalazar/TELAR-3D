//! El repositorio: dónde está su raíz y qué archivos tiene.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Carpetas generadas o externas: compilación, Git y dependencias de otro entorno.
const SALTAR: [&str; 3] = ["target", ".git", "node_modules"];

/// Los archivos del repositorio, con rutas relativas y separadas por `/`.
pub struct Repo {
    /// La carpeta raíz (la del `Cargo.toml` con `[workspace]`).
    pub raiz: PathBuf,
    /// Todos los archivos, ordenados: `crates/telar-guard/src/lib.rs`…
    pub archivos: Vec<String>,
}

impl Repo {
    /// Recorre `raiz` y lista sus archivos.
    ///
    /// # Errors
    ///
    /// Si alguna carpeta no se puede leer.
    pub fn abrir(raiz: &Path) -> io::Result<Self> {
        let mut archivos = Vec::new();
        recorrer(raiz, raiz, &mut archivos)?;
        archivos.sort();
        Ok(Self { raiz: raiz.to_path_buf(), archivos })
    }

    /// El texto de un archivo, o `None` si no existe o no es UTF-8.
    #[must_use]
    pub fn leer(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.raiz.join(rel)).ok()
    }

    /// Los bytes de un archivo.
    #[must_use]
    pub fn bytes(&self, rel: &str) -> Option<Vec<u8>> {
        fs::read(self.raiz.join(rel)).ok()
    }

    /// ¿Existe este archivo o carpeta?
    #[must_use]
    pub fn existe(&self, rel: &str) -> bool {
        self.raiz.join(rel).exists()
    }

    /// Los crates: cada carpeta `crates/<nombre>/` con su `Cargo.toml`.
    #[must_use]
    pub fn crates(&self) -> Vec<String> {
        self.archivos
            .iter()
            .filter_map(|f| f.strip_prefix("crates/")?.strip_suffix("/Cargo.toml"))
            .filter(|nombre| !nombre.contains('/'))
            .map(str::to_owned)
            .collect()
    }

    /// Los archivos `.rs` que viven debajo de `prefijo`.
    pub fn rust_en<'a>(&'a self, prefijo: &'a str) -> impl Iterator<Item = &'a String> + 'a {
        self.archivos
            .iter()
            .filter(move |f| f.starts_with(prefijo) && Path::new(f).extension().is_some_and(|e| e == "rs"))
    }
}

fn recorrer(raiz: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entrada in fs::read_dir(dir)? {
        let entrada = entrada?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let ruta = entrada.path();
        if entrada.file_type()?.is_dir() {
            if !SALTAR.contains(&nombre.as_str()) {
                recorrer(raiz, &ruta, out)?;
            }
        } else if let Ok(rel) = ruta.strip_prefix(raiz) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// Sube desde `desde` hasta encontrar la raíz del repo: la carpeta con un
/// `Cargo.toml` que declara `[workspace]` y una carpeta `crates/`.
#[must_use]
pub fn buscar_raiz(desde: &Path) -> Option<PathBuf> {
    desde.ancestors().find_map(|dir| {
        let toml = fs::read_to_string(dir.join("Cargo.toml")).ok()?;
        (toml.lines().any(|l| l.trim() == "[workspace]") && dir.join("crates").is_dir()).then(|| dir.to_path_buf())
    })
}
