# telar-guard

El guardián de las reglas de TELAR 3D. Rust, sin un solo crate.

> Marca lo peligroso, hazlo pequeño, justifícalo y que un guardián lo vigile.

```text
cargo guard check     revisa las reglas (sale con 1 si alguna falla)
cargo guard rules     cada regla, con su porqué y su arreglo
cargo verify          guardián + rustfmt + clippy pedantic + tests (y activa los hooks de git)
```

## Las reglas de código

| Regla | Qué exige |
|---|---|
| `semaforos` | cada carpeta lleva SEMAFORO.md con riesgo y consumo; FFI ROJO y jerarquía de riesgos coherente |
| `cero-deps` | en los `Cargo.toml`, solo dependencias por ruta dentro de `crates/` (tablas en línea, `[dependencies.x]` y `[target.….dependencies]` incluidas) |
| `fronteras` | cada crate usa solo los crates que declara en `[package.metadata.telar] puede-usar`; `telar-ventana` y `telar-gpu`, aisladas por completo (ni entre ellas); el núcleo solo usa el núcleo |
| `nucleo-sin-std` | núcleo y `telar-render`: `#![no_std]`, sin `std::` ni `extern crate std` |
| `unsafe-acotado` | `unsafe` (y `allow(unsafe_code)`) solo en `crates/*/src/ffi/`, con `// SAFETY:` en el bloque de comentarios justo encima, o `/// # Safety` en una `unsafe fn`; los tipos de puntero a función no lo necesitan |
| `ffi-aislada` | `extern "…"`, `#[link]`, `LoadLibraryW`, `GetProcAddress`, `dlopen` y `dlsym`, solo en `src/ffi/` |
| `tamano` | ningún archivo de código pasa de 400 líneas |
| `sin-restos` | ni `dbg!`, `todo!`, `unimplemented!` ni `#[ignore]` sin motivo |
| `lf` | saltos de línea LF en los archivos de texto |
| `crate-completo` | cada crate con README.md y tests |

Las reglas de la GPU (`memoria-gpu`, `barreras`, `limites`, `determinismo`) vivirán en el compilador de HILO, y las de resultados llegarán con sus fases. Ver [docs/GUARDIANES.md](../../docs/GUARDIANES.md).

## Cómo lee el código

`escaner::blanquear` sustituye por espacios los comentarios (también los `/* */` anidados) y el interior de las cadenas (normales, en bruto `r#"…"#`, de bytes) y de los caracteres, sin tocar los lifetimes y conservando los saltos de línea. Así:

- un `unsafe` en un comentario o un `dbg!` en un mensaje no cuentan;
- cada problema sale con su `archivo:línea` exactos.

## Dónde vigila

- **`cargo guard check`** y **`cargo verify`**.
- **Hooks de git** (`.githooks/`, que `cargo verify` activa):
  - `pre-commit` ejecuta `cargo verify`;
  - `commit-msg` exige el formato `tipo(ámbito): descripción` (`telar-guard commit-msg`).
- **Hooks de Claude Code** (`.claude/settings.json`), también aquí dentro (`telar-guard hook …`, con el evento JSON por la entrada estándar):
  - `pre-bash` deniega `cargo add`/`install`, `--no-verify`, `git commit -n`, `git push --force` y cualquier cambio de `core.hooksPath`;
  - `post-edit` pasa el guardián tras cada edición;
  - `stop` no deja terminar con reglas rotas o tests fallando.

## Tests

```bash
cargo test -p telar-guard
```

`tests/reglas.rs` monta un repositorio de prueba por regla, lo rompe y comprueba que bloquea en el sitio exacto, sin confundirse con lo que solo lo parece. `tests/piezas.rs` prueba el escáner, los commits, el JSON y los hooks.
