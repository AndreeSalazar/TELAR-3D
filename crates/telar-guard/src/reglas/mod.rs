//! Las reglas del repositorio y cómo se revisan.
//!
//! Las reglas de la GPU (`memoria-gpu`, `barreras`, `limites`, `determinismo`)
//! vivirán en el compilador de HILO, y las de resultados (`nodo-completo`,
//! `rendimiento`…) llegarán con sus fases. Ver docs/GUARDIANES.md.

mod deps;
mod fronteras;
mod higiene;
mod inseguro;
mod nodos;
mod nucleo;

use crate::informe::{Regla, Resultado};
use crate::repo::Repo;

/// Todas las reglas de código, en el orden en que se informan.
pub static REGLAS: [Regla; 10] = [
    Regla {
        id: "cero-deps",
        titulo: "Sin crates de terceros: solo dependencias por ruta dentro de crates/",
        motivo: "control total: cada línea que corre es nuestra",
        arreglo: "escríbelo como un crate en crates/ y enlázalo con `path = \"../x\"`",
        revisar: deps::revisar,
    },
    Regla {
        id: "fronteras",
        titulo: "Cada crate usa solo lo que declara; ventana y GPU, aisladas por completo",
        motivo: "cada pieza se cambia sin romper las demás, y el núcleo no conoce el sistema",
        arreglo: "declara la dependencia en [package.metadata.telar] puede-usar, o une las piezas en telar-app",
        revisar: fronteras::revisar,
    },
    Regla {
        id: "nucleo-sin-std",
        titulo: "nudo, hilo, hilo-spirv, telar-nodos y telar-kernel: #![no_std], solo core y alloc",
        motivo: "que el núcleo funcione en cualquier sistema, también en BMO-X",
        arreglo: "usa `core::`/`alloc::`; lo que necesite el sistema va en telar-gpu o telar-app, y los tests en tests/",
        revisar: nucleo::revisar,
    },
    Regla {
        id: "unsafe-acotado",
        titulo: "unsafe solo en crates/*/src/ffi/, con // SAFETY: justo encima",
        motivo: "lo peligroso, pequeño, en un sitio y justificado",
        arreglo: "mueve el unsafe a src/ffi/ con una API segura por encima, y escribe por qué es correcto en `// SAFETY:`",
        revisar: inseguro::unsafe_acotado,
    },
    Regla {
        id: "ffi-aislada",
        titulo: "extern \"…\", #[link], LoadLibraryW y GetProcAddress, solo en src/ffi/ (y ffi privado)",
        motivo: "hablar con Windows y con Vulkan, solo por una puerta",
        arreglo: "declara la función del sistema en src/ffi/ y llámala desde fuera a través de un envoltorio seguro",
        revisar: inseguro::ffi_aislada,
    },
    Regla {
        id: "tamano",
        titulo: "Ningún archivo de código pasa de 400 líneas",
        motivo: "un archivo, una responsabilidad",
        arreglo: "divide el archivo por responsabilidades (un módulo por idea)",
        revisar: higiene::tamano,
    },
    Regla {
        id: "sin-restos",
        titulo: "Sin dbg!, todo!, unimplemented! ni #[ignore] sin motivo",
        motivo: "lo que no está hecho se rechaza con un motivo, no se esconde",
        arreglo: "quita el resto de depuración, o devuelve un error que diga qué falta",
        revisar: higiene::sin_restos,
    },
    Regla {
        id: "lf",
        titulo: "Saltos de línea LF en todos los archivos de texto",
        motivo: "sin diffs fantasma entre Windows y Linux",
        arreglo: "guarda el archivo con LF (el .gitattributes ya lo pide a git)",
        revisar: higiene::lf,
    },
    Regla {
        id: "crate-completo",
        titulo: "Cada crate tiene README.md y tests",
        motivo: "cada pieza se explica y se demuestra sola",
        arreglo: "añade crates/<crate>/README.md y tests (tests/*.rs o #[test] en src/)",
        revisar: higiene::crate_completo,
    },
    Regla {
        id: "nodo-completo",
        titulo: "Cada nodo con su código, su libreto, su test y su comparación con Houdini",
        motivo: "cada nodo se explica y se demuestra solo; lo que no se ha comparado se dice",
        arreglo: "añade mod.rs y libreto.md en su carpeta, tests/<nodo>.rs y referencias/<nodo>.md (con su etiqueta)",
        revisar: nodos::revisar,
    },
];

/// Revisa todas las reglas sobre el repo.
#[must_use]
pub fn revisar(repo: &Repo) -> Vec<Resultado> {
    REGLAS.iter().map(|regla| Resultado { regla, problemas: (regla.revisar)(repo) }).collect()
}
