# telar-app

`telar`, la aplicación de TELAR 3D. Es **el único sitio que conoce a la vez la ventana, la GPU, los shaders y la geometría** (regla `fronteras`, declarado en su `Cargo.toml`):

```text
telar-ventana ──(HINSTANCE, HWND como números)──► telar-app ──► telar-gpu
nudo + telar-nodos ──(vértices e índices)────────►    │
hilo-spirv ──(SPIR-V como &[u32])────────────────►    ┘
```

Ninguno de esos crates sabe que existen los otros.

```bash
cargo run -p telar-app --release            # el visor: la esfera de un millón de puntos (Esc o la X para salir)
cargo run -p telar-app -- --prueba          # oculta, 120 fotogramas y dos cambios de tamaño; sale con 1 si hay avisos
cargo run -p telar-app -- --segundos 5      # visible, y se cierra sola (para medir los fps)
cargo run -p telar-app -- --puntos 250000   # otra esfera (por defecto, un millón; en la prueba, 40 000)
cargo run -p telar-app -- --sin-validacion  # sin las capas de validación
```

## El visor (Fase 4)

Arrastrar con el botón izquierdo orbita; la rueda acerca y aleja; quieta 3 s, la cámara gira sola. El título dice los puntos, la GPU y los fps.

En la RTX 3060, con validación:

```text
telar: esfera de 996872 puntos · 1993740 triángulos · cocinada en 107 ms
telar: 451 fotogramas · 0 avisos de validación
telar: 6.0 s · 75.1 fps de media
```

75 fps es la frecuencia del monitor: la presentación va sincronizada (FIFO).

## La Fase 1, medida

En la máquina de referencia (Ryzen 5 5600X, RTX 3060, monitor a 75 Hz):

- **75 fps**, sincronizado con el monitor (377 fotogramas en 5 s);
- **0 avisos** de las capas de validación, también al cambiar de tamaño (la swapchain se rehace);
- un ejecutable de unos **166 KB**.

Los avisos de tipo GENERAL (del cargador de Vulkan y de capas que otros programas instalan en todas las apps, como el overlay de GOG Galaxy) se enseñan aparte y no cuentan: no hablan de TELAR.

## Tests

- **`tests/app.rs`** lanza el ejecutable de verdad con `--prueba` y exige «120 fotogramas · 0 avisos de validación». Lo repite con el millón de puntos.
- **`tests/visor.rs`** dibuja esferas de NUDO con los shaders de `hilo-spirv` en capturas fuera de pantalla y mira los píxeles. Comprueba tres cosas: arriba es arriba, se ven las caras de delante (y no las de dentro) y lo de delante tapa lo de atrás.

## La Fase 2: el primer kernel

```bash
cargo run -p telar-app --release -- --calculo                    # 16 777 216 números
cargo run -p telar-app --release -- --calculo --elementos 1000000
```

`hilo-spirv` escribe el kernel, `telar-gpu` lo ejecuta y lo mide, y la app lo compara **bit a bit** con `hash_pcg` en la CPU. En la RTX 3060:

```text
GPU (solo el kernel): 0.196 ms · CPU (un hilo): 16.868 ms
GPU y CPU iguales bit a bit (16777216 de 16777216) · 0 avisos de validación
```

El test `el_primer_kernel_da_lo_mismo_en_la_gpu_que_en_la_cpu` lo exige con 1 048 576 números.
