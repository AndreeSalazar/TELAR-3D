# TELAR UI · primer estudio nativo

Interfaz pura en `no_std + alloc`, solo depende de `telar-estilo`, sin crates externos ni conocimiento de Vulkan,
Win32, NUDO o HILO. Produce triángulos 2D y consume posiciones de ratón; la app
conecta eventos, parámetros y renderizado. No se modifica la geometría 3D en CPU.

![Estudio nativo](../../docs/capturas/nodos-salida.png)

## Qué funciona

- Grafo con nodos seleccionables y arrastrables: rejilla, onda, relieve y salida. La preparación de
  argumentos GPU es interna y no ocupa un nodo visual.
- Inspector propio de cada nodo: Rejilla (resolución), Onda (amplitud/velocidad),
  Relieve (factor vivo HILO) y Salida (exposición/rugosidad/material). Los valores
  se conservan al cambiar de selección; pausa es global.
- El visor central recibe órbita y zoom; arrastrar un deslizador no orbita.
- Visor a la izquierda; ajustes arriba a la derecha y lienzo de grafo debajo,
  tomando la distribución de la referencia aportada por el usuario.
- Identidad propia: símbolo de tejido, tarjetas redondeadas, puertos y cables
  Bézier. Los cables siguen el movimiento; las posiciones relativas se conservan
  al redimensionar. La selección aparece por encima de nodos superpuestos.
- Composición grafito/cian/cobre, puntos discretos y controles mínimos.
- Tipografía proporcional con antialias y atlas residente; realce de nodos,
  deslizadores, pausa y colores al pasar el cursor. Tab recorre los cuatro nodos.
- Bajo 720×600 se ocultan paneles para conservar un visor utilizable.

Guardar/Abrir solicitan E/S a la app; el codec puro de estado conserva los
parámetros y posiciones. Ver [escenas v2/v3](../../docs/ESCENAS.md).
La red ya crea Ondas/Relieves, reconecta puertos y omite operaciones; hasta ocho
operaciones sobre una rejilla. Los campos del inspector admiten edición por teclado.
Ver [grafo editable](../../docs/GRAFO-EDITABLE.md). Siguen pendientes las tablas de atributos. Los colores de vista
previa seleccionan perfiles GGX opacos del renderizador, todavía sin transmisión
ni asignaciones de material por objeto. Faltan escalado DPI completo, composición de texto
Unicode más allá del alfabeto latino, accesibilidad y navegación por teclado de
todos los controles. Espacio pausa/reanuda desde la ventana.
Mover una tarjeta cambia su disposición; conectar puertos define el orden de
composición. Solo se compila el camino alcanzable desde Salida.

## Responsabilidades

El contenido se compone en Rust; la apariencia sale de la hoja de `telar-estilo`.
`Estudio::cargar_estilos` aplica personalizaciones atómicamente sobre el tema
incorporado, y `Estudio::distribucion` reutiliza la hoja sin analizar CSS por
fotograma. Paneles y nodos consumen el modelo de cajas; las etiquetas se miden
con los avances de la fuente y se abrevian cuando no caben. La distribución
se usa tanto para dibujar como para recibir eventos. La escena continúa en GPU.

| Archivo | Función |
|---|---|
| `src/lib.rs` | estado, personalización CSS y consumo de eventos |
| `src/parametros.rs` | configuraciones y controles propios de cada nodo |
| `src/distribucion.rs` | cajas adaptables calculadas con estilos compartidos |
| `src/dibujo.rs` | composición de paneles, red e inspector |
| `src/lienzo.rs` | triángulos, glifos de atlas y fuente 5×7 de respaldo |
| `src/fuente.rs` | validación de métricas y cobertura sin conocer Windows |
| `tests/estudio.rs` | límites, interacción, distribución y geometría finita |

La app compone triángulos solo al cambiar estado o tamaño. El backend conserva
dos buffers mapeados y solo copia la revisión nueva cuando la valla del juego
permite escribir. El overlay se dibuja en GPU, sin profundidad, después de la
escena. La CPU prepara esta geometría pequeña de interfaz, separada de los datos
procedurales residentes.

`telar-ventana` rasteriza Segoe UI a 24 px una vez usando GDI aislado en su FFI;
la app entrega métricas a `telar-ui` y cobertura a `telar-gpu`. El shader interpola
la cobertura y mezcla alfa. No se rasterizan letras en CPU por fotograma. La UI
emite seis vértices por glifo, con posición, coordenada de atlas y RGBA (32 bytes).
La fuente 5×7 sigue disponible para componer sin atlas en pruebas puras.

`telar-app/tests/interfaz_gpu.rs` captura la escena con el overlay y verifica
píxeles y cambios reales de superficie al usar el deslizador. La captura de
esta página sale de `telar-app/examples/capturar_estudio.rs`: Vulkan real, 256²
puntos, imagen 1280×720, sin capturar otras ventanas ni una maqueta HTML.

Contrato y evidencia de parámetros vivos: [NODOS-PARAMETROS.md](../../docs/NODOS-PARAMETROS.md).

Salida expone calidad MSAA 1x/2x/4x. El backend acepta `Opciones::muestras`
y `Gpu::configurar_muestras`; el inspector muestra la cantidad efectiva.
[Contrato, pruebas y coste](../../docs/CALIDAD-RENDER.md).

Persistencia del estudio: `.tlar` v2/v3, Guardar/F5 y Abrir/F9; la E/S queda en
la app y el estado portable en UI. `--abrir-escena` reconstruye capacidad y
`--guardar-escena` elige destino y guarda al salir. [Contrato](../../docs/ESCENAS.md).

Grafo editable de rejilla: instancias de Onda/Relieve, conexiones, omisión y
serialización v3. El compilador HILO incorpora Seno/Coseno escalares y fusiona
las operaciones alcanzables en un kernel por punto. La edición numérica aplica
con Enter y cancela con Escape. [Contrato y costes](../../docs/GRAFO-EDITABLE.md).
