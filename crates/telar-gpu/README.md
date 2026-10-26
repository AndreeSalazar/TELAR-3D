# telar-gpu

La GPU de TELAR 3D: **Vulkan a mano**, sin crates y sin enlazar el SDK (`vulkan-1.dll` se carga al arrancar, y cada función por su nombre). **Aislada por completo** (regla `fronteras`): no usa ningún crate de TELAR; de la ventana solo recibe dos números ([`Superficie`]).

```rust
let mut gpu = Gpu::nueva(&Opciones::default(), Superficie { instancia, ventana }, (1280, 720))?;
gpu.dibujar(eleccion::hex(0x35e3ff))?;   // limpia a cian y presenta, sincronizado con el monitor
let calculo = Calculo::nuevo(&Opciones::default())?; // sin ventana: para calcular y para los tests
```

## Cómo está hecho

| Dónde | Qué |
|---|---|
| `src/eleccion.rs` | las decisiones puras, sin `unsafe` y probadas sin GPU: qué dispositivo (la dedicada primero), qué formato (sRGB), qué tamaño, cuántas imágenes y el color en luz lineal |
| `src/ffi/tipos.rs` | las estructuras y constantes, copiadas de `vulkan_core.h`/`vulkan_win32.h` y **comprobadas una a una contra los headers del SDK 1.4.350** |
| `src/ffi/carga.rs` · `funciones.rs` | `vulkan-1.dll` y cada función con su firma; las de la swapchain son opcionales (sin ventana no existen) |
| `src/ffi/contexto.rs` | instancia, capas de validación, superficie Win32, elección de la GPU, dispositivo y cola |
| `src/ffi/presentacion.rs` | swapchain, sincronización por fotograma y el dibujo |

Todo el `unsafe` vive en `src/ffi/`, cada bloque con su `// SAFETY:`; los tipos de puntero a función no lo necesitan (no ejecutan nada), y el guardián lo sabe.

## Sincronización

- Dos fotogramas en vuelo, cada uno con su valla y su semáforo de adquisición.
- Un semáforo de «listo para presentar» **por imagen**: la presentación lo usa hasta que esa imagen vuelve a adquirirse, así que reutilizarlo por fotograma provocaría avisos, y carreras de verdad.
- La swapchain se rehace sola al cambiar de tamaño o al volver de minimizada.

## Validación

Con `Opciones::validacion` (por defecto) y el SDK instalado, se activa `VK_LAYER_KHRONOS_validation` con un mensajero que cuenta cada aviso: [`avisos_de_validacion`] tiene que ser **0**.

Los avisos de tipo GENERAL (el cargador, y capas que otros programas meten en todas las apps Vulkan, como el overlay de GOG Galaxy) se enseñan como `[vulkan · sistema, no cuenta]`.

Las capas de validación ya cazaron dos fallos en el desarrollo: cargar funciones de la swapchain en un dispositivo sin ella, y un dispositivo que no se destruía cuando algo fallaba a medias.

## Tests

```bash
cargo test -p telar-gpu -- --nocapture
```

Las decisiones puras, y un contexto de Vulkan de verdad (sin ventana) con validación: **cero avisos**. En una máquina sin Vulkan, el test lo dice y no cuenta como fallo.
