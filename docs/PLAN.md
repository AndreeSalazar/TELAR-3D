# El plan, por fases

Cada fase termina con algo que **funciona y se puede enseñar** (una ventana, un kernel, un vídeo) y con su «hecho cuando», comprobable. No hay fechas: cada fase dura lo que tarde en estar bien. El orden pone primero los guardianes, después ver la GPU pronto (motiva) y, desde ahí, el camino más corto hacia la esfera de fuego.

## Fase 0 · El taller

Workspace de Cargo, `rust-toolchain.toml` fijada, `telar-guard` con las reglas de código, hooks de git y de Claude Code, y el comando `verify`.

**Hecho cuando:** `verify` pasa en limpio, y un repositorio de prueba que rompe cada regla queda bloqueado con su motivo.

## Fase 1 · Una ventana y Vulkan, a mano

- **`telar-ventana`:** Win32 sin crates (`CreateWindowExW` y el bucle de mensajes).
- **`telar-gpu`:** cargar `vulkan-1.dll` con `LoadLibraryW`, crear la instancia y el dispositivo (la RTX 3060), la superficie Win32 y la *swapchain*.
- La pantalla se limpia en cian, sincronizada con el monitor.

**Hecho cuando:** la ventana se ve a la frecuencia del monitor, con **cero avisos** de las capas de validación, y todo el `unsafe` está en `ffi/` con su `// SAFETY:`.

**✅ Hecha.** En la RTX 3060:
- 75 fps, la frecuencia del monitor (377 fotogramas en 5 s);
- 0 avisos de validación, también al rehacer la swapchain en dos cambios de tamaño;
- un ejecutable de 166 KB.

La ventana y la GPU están aisladas por completo (regla `fronteras`) y solo `telar-app` las une. La prueba automática es `cargo run -p telar-app -- --prueba`.

## Fase 2 · El primer kernel, en SPIR-V a mano

- **`hilo-spirv`:** un constructor de SPIR-V en `no_std`, una API en Rust que emite las instrucciones. Es el «ensamblador» antes de tener el lenguaje, como TALK-CPU antes de los compiladores.
- El primer *compute shader*: llenar un buffer con `hash(i)`.

**Hecho cuando:** el SPIR-V pasa `spirv-val`, la GPU da los mismos bytes que la referencia en la CPU, y el tiempo se mide con temporizadores de la GPU.

**✅ Hecha.** `hilo-spirv` (`no_std`) escribe el kernel: 230 palabras, aprobadas por `spirv-val`, que en su primera ejecución cazó un fallo del constructor. En la RTX 3060, 16 777 216 números:
- 0,196 ms en la GPU, solo el kernel, medido con sus temporizadores;
- 16,9 ms en la CPU (un hilo);
- **iguales bit a bit**, y 0 avisos de validación.

Son ~342 GB/s, cerca del máximo de la memoria (360 GB/s). Los datos viven en memoria de la GPU y solo al final se copian a la CPU. `telar --calculo` lo repite.

## Fase 3 · NUDO, la geometría y el grafo

Puntos, vértices, primitivas y atributos; el grafo con marcas de sucio; los nodos caja, esfera, rejilla, transformar y unir. Todo en `no_std`, en la CPU y con tests.

**Hecho cuando:** cambiar un parámetro recalcula solo lo que depende de él (hay un test que cuenta los nodos cocinados), y las estadísticas coinciden con Houdini (recuentos, caja y promedios).

**✅ Hecha en TELAR; ABIERTA contra Houdini.**
- **`nudo`** (`no_std`): geometría con el modelo de Houdini y el grafo con marcas de sucio. Con `sin`, `cos` y `raiz` propios: error < 2·10⁻⁷ frente a `std`.
- **`telar-nodos`:** caja, rejilla, esfera, transformar y unir, una carpeta cada uno con su libreto.
- **El contador de los tests:** mover la caja recocina solo `transformar` y `unir`, y dar detalle a la esfera, solo `esfera` y `unir`. Todas las caras miran hacia fuera.
- **La regla nueva `nodo-completo`** exige código, libreto, test y referencia.

Las referencias (`referencias/`) están en **ABIERTO**: hay que abrir Apprentice y anotar sus cifras.

## Fase 4 · El visor

Rasterizar la geometría de NUDO en la GPU (vértices y fragmentos con `hilo-spirv`), con cámara orbital y la profundidad invertida.

**Hecho cuando:** una esfera de un millón de puntos se mueve a 60 fps, sin avisos de validación.

**✅ Hecha.** En la RTX 3060, con las capas de validación activas:
- una esfera de NUDO de **996 872 puntos** (≈ 2 millones de triángulos) gira a **75 fps**, la frecuencia del monitor (451 fotogramas en 6 s);
- **0 avisos**, también al rehacer la swapchain y su profundidad en dos cambios de tamaño.

Cómo está hecho:
- **`nudo`:** la cámara orbital (`Camara`, matrices por columnas, profundidad invertida e infinita) y `Malla`, que parte la geometría en triángulos con normales suaves y aristas de más de 30°.
- **`hilo-spirv`:** los shaders de vértices y de fragmentos, que pasan `spirv-val`. La luz es difusa, más un ambiente de cielo y suelo.
- **`telar-gpu`:** renderizado dinámico de Vulkan 1.3, sin render pass. El pipeline gráfico usa 128 bytes de constantes; la malla se sube a memoria de la GPU.
- **`telar-ventana`:** botones y rueda del ratón, con captura mientras se arrastra.

Las capturas fuera de pantalla (`Calculo::capturar`) dejan probar el dibujo píxel a píxel, sin ventana: arriba es arriba, se descartan las caras traseras y no las delanteras, y lo de delante tapa lo de atrás aunque se dibuje antes. `telar --prueba --puntos 1000000` repite la meta, y `telar --segundos 6` mide los fps.

## Fase 5 · HILO v0

El lenguaje, por punto como un *wrangle*:

```
@P += @N * ruido(@P * 2.0) * 0.1
```

Léxico, sintaxis, tipos, IR en SSA, fusión de nodos y SPIR-V, más el **intérprete en la CPU** como referencia. Las reglas `memoria-gpu`, `barreras`, `limites` y `determinismo` viven en su comprobador de tipos.

**Hecho cuando:** el mismo programa da el mismo resultado en la GPU y en el intérprete, y un kernel con una carrera o con una suma atómica de `float` no compila, con un mensaje que dice por qué.

## Fase 6 · La esfera, hitos 1 a 3

Humo 2D, humo 3D con marcha de rayos y fuego (combustión y color de cuerpo negro). Ver [ESFERA-DE-FUEGO.md](ESFERA-DE-FUEGO.md).

**Hecho cuando:**
- la divergencia tras proyectar queda bajo ε;
- el hash de los campos es idéntico entre ejecuciones;
- la bola de fuego en vivo va a 128³ y 60 fps;
- y hay un vídeo.

## Fase 7 · La esfera, hitos 4 a 6

Detalle (MacCormack, vorticidad y *curl noise*), luz (sombras, dispersión y *bloom*) y **la GPU decide** (ladrillos activos con despacho indirecto).

**Hecho cuando:** la CPU solo escribe los parámetros por fotograma (se comprueba contando sus llamadas a Vulkan por fotograma), y el resultado se compara con un Pyro de Houdini, con su etiqueta.

## Fase 8 · El telar: la interfaz de nodos

`telar-app`: nodos y cables elegantes, parámetros en vivo y el estilo neón de la casa (hielo, cian y rojo). Mientras tanto, las escenas se escriben como **libretos** (archivos de HILO en `libretos/`).

**Hecho cuando:** la esfera de fuego se arma desde cero en la interfaz, sin escribir código, y mover un deslizador cambia el fuego en el siguiente fotograma.

## Fase 9 · Final y comparación publicada

Calidad final a 256³ o más, con multigrid, y la tabla contra Houdini: IGUAL, MÁS RÁPIDO, DISTINTO y ABIERTO, con la máquina al lado. El vídeo para el canal.

## Después (P2–P3)

*Scatter* con Poisson disk, copiar a puntos, partículas y chispas, terrenos, Vellum con XPBD, FLIP y rígidos. Ver [HOUDINI-MAPA.md](HOUDINI-MAPA.md).

## Algún día

El mismo núcleo `no_std` sobre **VERRANO en BMO-X**, cuando se resuelva el caso 0x15 de la RTX. La esfera de fuego en tu propio sistema operativo.
