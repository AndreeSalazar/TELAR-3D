# TELAR 3D — reglas del proyecto

Sistema 3D procedural de nodos en **Rust**, con el núcleo `no_std` y la GPU al mando. Comentarios, mensajes y documentación en **español**. Los planes están en `docs/`; léelos antes de cambiar la arquitectura.

## Mapa (✅ = ya existe; ver docs/ARQUITECTURA.md)

```
crates/
  telar-guard/   ✅ los guardianes (10 reglas)                   std, sin crates
  telar-ventana/ ✅ Win32 a mano (unsafe SOLO en src/ffi/)       std · aislada: no usa nada nuestro
  telar-gpu/     ✅ Vulkan a mano (unsafe SOLO en src/ffi/)      std · aislada: no usa nada nuestro
  telar-ui/      ✅ estado y triángulos de interfaz; sin GPU/Win32 ni dependencias    no_std + alloc
  telar-app/     ✅ `telar`: une ventana, GPU, shaders y geometría (el visor; en la Fase 8, la interfaz de nodos)
  nudo/          ✅ geometría, atributos y grafo que cocina      no_std + alloc
  hilo/          ✅ IR tipado, intérprete CPU y compilación SPIR-V; texto pendiente   no_std + alloc
  hilo-spirv/    ✅ SPIR-V emitido a mano (el primer kernel y los shaders del visor)  no_std + alloc
  telar-nodos/   ✅ un nodo por carpeta (código, libreto, tests)  no_std + alloc · referencias/ con su etiqueta
```

Cada crate declara a quién puede usar en su `Cargo.toml` (`[package.metadata.telar] puede-usar = [...]`, regla `fronteras`). Formato: `rustfmt` a 120 columnas (`rustfmt.toml`).

```bash
cargo guard check     # el guardián
cargo verify          # guardián + rustfmt + clippy pedantic + tests: antes de dar algo por terminado
cargo run -p telar-app --release         # rejilla residente: generación, deformación y dibujo GPU
cargo run -p telar-app --release -- --esfera-cpu # referencia histórica de la Fase 4
cargo run -p telar-app -- --prueba        # 120 fotogramas oculta: 0 avisos de validación o falla
cargo run -p telar-app --release -- --calculo   # el primer kernel: GPU contra CPU, bit a bit
```

## Reglas (las hará cumplir `telar-guard`)

1. **Cero dependencias de terceros.** Ni un crate de crates.io. Si hace falta algo, se escribe aquí. Las herramientas de verificación externas (capas de validación de Vulkan, `spirv-val`, Houdini Apprentice como referencia) sí valen: miden, no se enlazan.
2. **`no_std` en el núcleo:** `nudo`, `hilo`, `hilo-spirv` y `telar-nodos` llevan `#![no_std]` y solo usan `core` y `alloc`.
3. **`unsafe` solo en `crates/*/src/ffi/`**, siempre con un `// SAFETY:` justo encima. Las declaraciones `extern "system"`, `LoadLibraryW` y `GetProcAddress`, también solo ahí.
4. **Memoria de la GPU marcada:** cada escritura compartida, cada barrera y cada atómico en un kernel lleva su justificación (`// GPU:`). Ver docs/GUARDIANES.md.
5. **Determinismo:** la misma entrada da los mismos bytes. Prohibidas las sumas de `float` con atómicos desordenados, salvo marcadas.
6. **Cada nodo, completo:** código, `libreto.md` (qué hace, la matemática y su fuente), tests y referencia contra Houdini.
7. **Rendimiento vigilado:** un nodo que se vuelve más lento que su referencia guardada no entra.
8. **Ningún archivo de código pasa de 400 líneas.**
9. **Sin restos de depuración:** ni `dbg!`, `todo!`, `unimplemented!` ni `#[ignore]` sin motivo.
10. **Clippy pedantic con `-D warnings`.** Una excepción necesita `#[allow(..., reason = "...")]`.
11. **Commits convencionales** `tipo(ámbito): descripción`. Nunca `--no-verify` ni `push --force`.
12. **Honestidad en los resultados:** cada comparación lleva su etiqueta (IGUAL, MÁS RÁPIDO, DISTINTO, ABIERTO) y la máquina en la que se midió.

## Inspiración, no copia

Se aprende de la documentación de Houdini y de los artículos y libros públicos (docs/MATEMATICAS.md). No se copia su código, sus textos ni sus archivos. Las ideas y la matemática son de todos; la implementación es nuestra.
