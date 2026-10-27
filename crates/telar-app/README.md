# telar-app

`telar`, la aplicación de TELAR 3D. Es **el único sitio que conoce a la vez la ventana, la GPU, los shaders y la geometría** (regla `fronteras`, declarado en su `Cargo.toml`):

```text
telar-ventana ──(HINSTANCE, HWND como números)──► telar-app ──► telar-gpu
nudo + telar-nodos ──(vértices e índices)────────►    │
hilo-spirv ──(SPIR-V como &[u32])────────────────►    ┘
```

Ninguno de esos crates sabe que existen los otros.

```bash
cargo run -p telar-app --release            # rejilla animada de un millón de puntos, generada y deformada en GPU
cargo run -p telar-app -- --prueba          # oculta, 120 fotogramas y dos cambios de tamaño; sale con 1 si hay avisos
cargo run -p telar-app -- --segundos 5      # visible, y se cierra sola (para medir los fps)
cargo run -p telar-app -- --puntos 250000   # rejilla de 500×500 (máximo 1024²; por defecto un millón)
cargo run -p telar-app --release -- --esfera-cpu # esfera histórica, cocinada en CPU
cargo run -p telar-app -- --sin-validacion  # sin las capas de validación
```

## El visor residente (Fase 4.5)

La ruta predeterminada no llama a `Nodo::cocinar`, no triangula en CPU y no sube
vértices. La GPU prepara los argumentos, genera la rejilla y deforma la altura
`A sin(2x + fase) cos(2z)` con su normal analítica. Dibuja desde los mismos
buffers mediante `vkCmdDrawIndexedIndirect`.

Cada fotograma escribe 16 bytes de parámetros de superficie; además se graban
128 bytes de constantes de cámara e iluminación. Dos juegos residentes evitan
sobrescribir los datos en vuelo. Al finalizar, informa de 8 buffers residentes,
6 pipelines de compute, 2 grabaciones de compute, los pasos y transferencias.
La prueba cambia las dimensiones dentro de capacidad sin crear recursos nuevos.

`src/escena.rs` prepara y comprueba la escena; `src/visor.rs` atiende la ventana
y la cámara. Los kernels viven en `hilo-spirv`, y el ejecutor Vulkan en
`telar-gpu`, sin dependencias entre ambos.

## La referencia histórica (Fase 4, --esfera-cpu)

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

- **`tests/app.rs`** lanza la ruta residente con un millón de puntos durante 120 fotogramas y dos cambios de tamaño; exige cero avisos y recursos de compute constantes. Conserva también la prueba de la esfera CPU.
- **`tests/procedural.rs`** comprueba recuentos, índices y normales contra CPU, vacío, grupo parcial, capacidad y exceso; exige repetición bit a bit en la misma GPU y cambios visibles en las capturas. Prueba también 1024² puntos.
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
