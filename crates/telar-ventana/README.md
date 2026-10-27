# telar-ventana

La ventana de TELAR 3D: **Win32 a mano**, sin crates. **Aislada por completo** (regla `fronteras`): no usa ningún crate de TELAR y no sabe que existe Vulkan.

```rust
let mut ventana = Ventana::nueva(&Config::default())?;
for evento in ventana.eventos() { /* Cerrar, Tamano, Tecla, Raton, Boton, Rueda */ }
let m = ventana.manejadores(); // HINSTANCE y HWND, como números: para la superficie de la GPU
```

## Qué hace

- **Centrada y nítida en cada monitor:** se declara *per-monitor DPI aware v2*, escala el tamaño pedido con el DPI y se recoloca sola al cambiar de monitor (`WM_DPICHANGED`).
- **Barra de título oscura** (`DWMWA_USE_IMMERSIVE_DARK_MODE`) y fondo del color del cosmos (#05070a) hasta el primer fotograma de Vulkan: sin el destello blanco.
- **Sin parpadeo al redimensionar:** no borra el fondo (`WM_ERASEBKGND`), porque pinta Vulkan.
- **La X no cierra sola:** llega `Evento::Cerrar` y decide la app.
- **Es de su hilo:** `Ventana` no es `Send`, como exige Win32.
- **El ratón:**
  - los tres botones, pulsados y soltados;
  - la rueda, en muescas con signo (fraccionarias en los paneles de precisión);
  - mientras un botón está pulsado, la ventana se queda con el ratón (`SetCapture`): el «soltado» llega aunque se suelte fuera, y el arrastre no se queda pegado.

## Dónde está lo peligroso

Todo el `unsafe` vive en `src/ffi/` (`win32.rs`: declaraciones copiadas de la documentación de Microsoft; `mod.rs`: funciones seguras por encima), cada uno con su `// SAFETY:`. La traducción de mensajes a eventos (`src/evento.rs`) es código puro, y se prueba sin abrir ninguna ventana.

## Tests

```bash
cargo test -p telar-ventana
```

Traducción de cada mensaje (también coordenadas negativas del ratón, botones, la rueda hacia los dos lados y la ventana minimizada), UTF-16, y una ventana de verdad, oculta, que se abre, se mide y se destruye.
