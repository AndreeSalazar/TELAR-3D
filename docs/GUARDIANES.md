# Guardianes de TELAR

> Marca lo peligroso, hazlo pequeño, justifícalo y que un guardián lo vigile.

Los guardianes no son frenos: son **banderas**. Dicen exactamente dónde puede fallar algo, para acelerar sin miedo en todo lo demás. Cada bloqueo explica la regla, el motivo, el culpable exacto (archivo y línea) y cómo arreglarlo.

## Dónde vigilan

1. **`telar-guard`** (Rust, sin crates), sobre todo el repositorio: `cargo run -p telar-guard -- check`.
2. **El propio compilador de HILO:** algunas reglas viven dentro del lenguaje, así que un kernel peligroso **ni siquiera compila**.
3. **Hooks de git** (`pre-commit`, `commit-msg`), **hooks de Claude Code** y `verify`. Nadie es root: ni el autor, ni la IA.

## Las reglas

### Sobre el código

| Regla | Qué exige | Por qué |
|---|---|---|
| `cero-deps` | en los `Cargo.toml`, solo dependencias por ruta dentro de `crates/` | control total: cada línea que corre es nuestra |
| `fronteras` | cada crate usa solo lo que declara en `[package.metadata.telar] puede-usar`; `telar-ventana` y `telar-gpu` no usan nada nuestro (ni entre ellas); el núcleo solo usa el núcleo | cada pieza se cambia sin romper las demás; solo `telar-app` une la ventana y la GPU |
| `nucleo-sin-std` | `nudo`, `hilo`, `hilo-spirv` y `telar-nodos` con `#![no_std]`, sin `std::` | que el núcleo funcione en cualquier sistema, también en BMO-X |
| `unsafe-acotado` | `unsafe` solo en `crates/*/src/ffi/`, con `// SAFETY:` justo encima (los tipos de puntero a función, `unsafe extern "system" fn(…)`, no lo necesitan: no ejecutan nada) | lo peligroso, pequeño, en un sitio y justificado |
| `ffi-aislada` | `extern "system"`, `LoadLibraryW` y `GetProcAddress`, solo en `ffi/`, y el módulo `ffi` nunca es `pub` | hablar con Windows y con Vulkan, solo por una puerta, que no se exporta |
| `tamano` | ningún archivo de código pasa de 400 líneas | partir por responsabilidades |
| `sin-restos` | ni `dbg!`, `todo!`, `unimplemented!` ni `#[ignore]` sin motivo | lo que no está hecho se rechaza con motivo, no se esconde |
| `lf` | saltos de línea LF | sin diffs fantasma |

### Sobre la GPU (en el compilador de HILO)

| Regla | Qué exige | Por qué |
|---|---|---|
| `memoria-gpu` | cada escritura en memoria compartida lleva `// GPU:` con su justificación (quién escribe, quién lee y qué barrera los separa) | las carreras entre miles de hilos no avisan: corrompen en silencio |
| `barreras` | cada lectura de algo escrito en otro paso del mismo kernel necesita su barrera; el compilador la comprueba | una barrera olvidada funciona «casi siempre», que es lo peor |
| `limites` | los accesos a buffers llevan comprobación de límites, salvo que el compilador demuestre que no hace falta | fuera del buffer, la GPU no da *segfault*: da basura |
| `determinismo` | prohibidas las sumas de `float` con atómicos, salvo marcadas `@desordenado` y con motivo | el orden de las sumas cambia el resultado: dos ejecuciones darían bytes distintos |

### Sobre los resultados

| Regla | Qué exige | Por qué |
|---|---|---|
| `nodo-completo` | cada `telar-nodos/<nodo>/` tiene `nodo.rs`, `libreto.md`, tests y `referencias/<nodo>.md` | cada nodo se explica y se demuestra solo |
| `spirv-valido` | todo SPIR-V que emite HILO pasa `spirv-val` en los tests | el SPIR-V lo escribimos a mano: que lo verifique alguien que no somos nosotros |
| `validacion-vulkan` | los tests de la GPU corren con las capas de validación, y cualquier aviso es un fallo | el driver perdona errores que otra GPU no perdonará |
| `gpu-igual-cpu` | cada kernel tiene su referencia en la CPU y coinciden, dentro de la tolerancia o bit a bit | la CPU como testigo |
| `rendimiento` | ningún nodo se vuelve más de un 10 % más lento que su referencia guardada (mediana de varias ejecuciones, en la máquina de referencia) | la velocidad no se pierde poco a poco sin que nadie lo note |
| `etiquetas` | cada comparación contra Houdini lleva IGUAL, MÁS RÁPIDO, DISTINTO o ABIERTO y la máquina | lo abierto se dice, no se finge |

## El orden: el guardián va primero

Como en VERX: **el guardián se escribe antes que el código que vigila.** Cada regla nueva entra con su test (un repositorio de prueba que la rompe y comprueba que bloquea). Si una regla molesta de verdad, se discute y se cambia **en el guardián**, con su test. Nunca se esquiva.
