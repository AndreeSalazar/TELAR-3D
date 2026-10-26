# telar-app

`telar`, la aplicación de TELAR 3D. Es **el único sitio que conoce a la vez la ventana y la GPU** (regla `fronteras`, declarado en su `Cargo.toml`):

```text
telar-ventana ──(HINSTANCE, HWND como números)──► telar-app ──► telar-gpu
```

Ninguno de los dos crates sabe que existe el otro.

```bash
cargo run -p telar-app --release            # la ventana: respira del negro del cosmos al cian (Esc o la X para salir)
cargo run -p telar-app -- --prueba          # oculta, 120 fotogramas y dos cambios de tamaño; sale con 1 si hay avisos
cargo run -p telar-app -- --segundos 5      # visible, y se cierra sola (para medir los fps)
cargo run -p telar-app -- --sin-validacion  # sin las capas de validación
```

## La Fase 1, medida

En la máquina de referencia (Ryzen 5 5600X, RTX 3060, monitor a 75 Hz):

- **75 fps**, sincronizado con el monitor (377 fotogramas en 5 s);
- **0 avisos** de las capas de validación, también al cambiar de tamaño (la swapchain se rehace);
- un ejecutable de unos **166 KB**.

Los avisos de tipo GENERAL (del cargador de Vulkan y de capas que otros programas instalan en todas las apps, como el overlay de GOG Galaxy) se enseñan aparte y no cuentan: no hablan de TELAR.

## Tests

`tests/app.rs` lanza el ejecutable de verdad con `--prueba` y exige «120 fotogramas · 0 avisos de validación».
