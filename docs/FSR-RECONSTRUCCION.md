# FSR y la idea de un segundo dibujante en GPU

Estudio de fuentes oficiales AMD/GPUOpen y diagnóstico del código de TELAR,
**27 de octubre de 2026**. No se ha integrado FSR ni se ha ejecutado un modelo ML.

## Qué parte de la idea tiene sentido

La imagen puede pasar por otra etapa de la **misma GPU** que reconstruya su salida
antes de presentarla. No es una segunda GPU física ni una mente que decide qué
objeto debería existir. Se ejecuta un algoritmo con entradas, historial y criterios
de validez definidos. No garantiza una imagen perfecta ni elimina todos los artefactos.

FSR es una familia de tecnologías. No basta decir «FSR usa IA» sin identificar
la versión y la función. La matriz actual de AMD distingue escalado espacial,
temporal, ML y generación de fotogramas:
[AMD FSR Technologies](https://www.amd.com/en/products/graphics/technologies/fidelityfx/super-resolution.html).

| Variante estudiada | Técnica | Distinción práctica |
|---|---|---|
| FSR 1 | Escalado espacial analítico | Trabaja sobre la imagen; no es reconstrucción temporal ni un modelo ML |
| FSR 2 | Reconstrucción temporal analítica | Usa información temporal y produce una salida con antialiasing |
| FSR 3 / 3.1 | Reconstrucción analítica y generación de fotogramas | Escalar una imagen y generar un fotograma intermedio son funciones distintas |
| FSR 4 / actual AMD FSR Upscaling | Escalado basado en ML | Requiere una implementación/modelo y una plataforma soportados |
| Redstone | Familia de funciones, incluidas variantes ML | No debe confundirse la generación ML reciente con el algoritmo analítico anterior |

Fuentes de las variantes:
[FSR 1](https://gpuopen.com/fidelityfx-superresolution/),
[FSR 2](https://gpuopen.com/fidelityfx-superresolution-2/),
[FSR 3](https://gpuopen.com/fidelityfx-super-resolution-3/),
[AMD FSR Upscaling](https://gpuopen.com/amd-fsr-upscaling/).

## Plataformas: no todas las versiones son intercambiables

La documentación de FSR 2 y 3 incluye Vulkan. Es una referencia pertinente para
nuestro backend, no una integración ya comprobada en TELAR.

La página de desarrolladores de **FSR Upscaling 4.1.1 / SDK 2.3** enumera DirectX 12
y Radeon RX 9000/7000 para su ruta ML. También distingue fallbacks analíticos.
No presenta la ruta ML como un componente Vulkan universal para una RTX 3060.
La ficha comercial y las páginas antiguas abarcan versiones y conjuntos de
hardware diferentes: comprobar siempre la versión concreta del SDK y su backend.
[Requisitos del desarrollador](https://gpuopen.com/amd-fsr-upscaling/).

Para TELAR, estudiar reconstrucción temporal **sin ML** es una propuesta razonable
para nuestra ruta Rust/SPIR-V/Vulkan. No se deduce de ello soporte para cualquier
GPU ni rendimiento mejor que MSAA. Una implementación propia tampoco sería FSR
oficial ni tendría automáticamente su calidad.

## Datos que explica la integración temporal

El manual de FSR 2 pide color, profundidad, movimiento y jitter de cámara; contempla
exposición, máscaras reactivas y tratamiento de transparencia. Reproyecta historial
y trata superficies recién visibles. Su antialiasing temporal evita necesitar otra
pasada TAA independiente. La ubicación del efecto importa: recomienda ejecutar
tone mapping después del escalado. Son requisitos de esa técnica, no de toda
variante FSR. [Manual FSR 2](https://gpuopen.com/manuals/fidelityfx_sdk/techniques/super-resolution-temporal/).

No se propone apilar indiscriminadamente MSAA 4x, TAA y reconstrucción. Cada modo
necesita un contrato y una comparación de calidad/coste. Para estudiar la técnica,
primero podemos mantener resolución nativa y evaluar la estabilidad; el escalado
desde una imagen interna menor vendría después como otro experimento.

Generar fotogramas intermedios no vuelve más exacta la simulación ni es la solución
directa al contorno escalonado. La calidad de reconstrucción, la cadencia de salida
y la latencia deben medirse por separado. Esto no sustituye tampoco los rebotes,
materiales y transporte de luz del futuro render de producción.

## Estado concreto de TELAR y cambios necesarios

Este diagnóstico sale de nuestros archivos, no de suponer que ya implementamos
todos los recursos de AMD:

| Recurso o etapa | Estado observado | Trabajo propuesto |
|---|---|---|
| Color de escena | `telar-render/src/superficie.rs` aplica Reinhard y salida sRGB antes de entregar el color | Separar una imagen de escena HDR de la salida final |
| Profundidad | `dibujo.rs` usa `NO_GUARDAR`; `multimuestreo.rs` crea profundidad como adjunto | Conservar profundidad legible en la ruta temporal; definir muestras y sincronización |
| Movimiento de cámara | Tenemos cámara actual, no un buffer de movimiento por píxel | Conservar transformaciones anteriores y comprobar sus convenciones |
| Movimiento de geometría | Onda/Relieve cambian posiciones, pero no producen vectores de movimiento | Incluir el desplazamiento del objeto además del de cámara |
| Jitter | No hay secuencia de desplazamientos subpíxel de proyección en el visor | Añadir y probar la secuencia, su escala y cancelación en movimiento |
| Historial | No hay historial de color para reconstrucción | Reservas residentes, reproyección, rechazo e invalidación |
| Dos juegos en vuelo | Aíslan recursos de ejecución | No confundirlos con «fotograma anterior» del historial; definir índices y barreras |
| Interfaz | Se compone en el mismo pase que la escena | Dibujarla nativa después de reconstruir, para no arrastrar letras y controles con la escena |
| Compute de imágenes | El ejecutor procedural usa descriptores de buffers | Preparar lectura/escritura de imágenes y sus disposiciones para una nueva pasada |

La ruta que proponemos probar sería:

```
GPU: geometría y sombreado → color HDR + profundidad + movimiento
                          → reconstrucción temporal con historial en VRAM
                          → tone mapping + interfaz nativa → presentación
```

En una cámara que orbita normalmente se intenta reutilizar historial mediante
movimiento. Un corte de cámara o un cambio incompatible exige reiniciar/rechazarlo.
No reiniciar indiscriminadamente en cada movimiento, ni aceptar todo el pasado:
ambos extremos impiden una reconstrucción útil. Las superficies que aparecen
detrás de otra no deben heredar su color anterior. Con humo y partículas habrá que
probar reactividad/transparencia, no suponer que el movimiento de superficies opacas basta.

## Qué significa «sin vigilancia de CPU»

La CPU no tiene que leer, inspeccionar o corregir cada píxel. Las pasadas pueden
encadenarse mediante comandos, recursos y barreras, conservando el historial en
GPU. Ese es el sentido viable de la propuesta.

El host sigue preparando y enviando trabajo, atendiendo entradas y gestionando
adquisición/presentación y sincronización. En el código actual están las llamadas
de envío y presentación en `telar-gpu/src/ffi/presentacion.rs`. Un compute shader
no sustituye directamente al sistema de ventanas ni controla por sí solo el monitor.
[Khronos: WSI](https://docs.vulkan.org/spec/latest/chapters/VK_KHR_surface/wsi.html).

## Hitos de un prototipo propio, todavía pendientes

1. Separar color HDR, profundidad conservada y composición final; mantener la ruta
   MSAA como referencia seleccionable y verificar orientación, exposición y formatos.
2. Obtener movimiento correcto de cámara y Onda/Relieve, incluyendo los estados
   anteriores coherentes. Probar cámara quieta, órbita y deformación por separado.
3. Reconstruir a resolución nativa con reproyección e historial residentes. Rechazar
   datos inválidos y reiniciar ante cortes, redimensionado o generaciones incompatibles.
4. Probar oclusiones nuevas, objetos finos, cambios de material/luz y movimientos
   rápidos. Medir estelas, parpadeo, pérdida de detalle y latencia.
5. Solo después, comparar escalado interno frente a MSAA y referencia de mayor
   muestreo. Añadir nitidez limitada si mejora la imagen sin halos.

Son propuestas para TELAR, no una descripción exhaustiva de FSR ni funciones
ya implementadas. La generación de fotogramas y un modelo ML propio quedan fuera
de este primer prototipo.

## Consumo, dependencia y evidencia

La reconstrucción necesita lecturas/escrituras por píxel e historial: tiene coste
GPU, ancho de banda y VRAM. El ahorro de sombrear a menos resolución debe superar
ese coste para acelerar la imagen completa. No se prometen FPS ni ahorro todavía.

Ejemplo de presupuesto **propuesto**, no consumo de FSR: dos imágenes RGBA16F de
1920×1080 suman 33.177.600 bytes, unos 31,64 MiB lógicos, antes de color actual,
profundidad, movimiento, máscaras y alineación. La memoria real y sus vidas útiles
deben medirse antes de implementar reutilización de regiones.

La regla actual de TELAR exige cero dependencias de terceros. Este estudio no
instala ni enlaza el SDK. Una futura integración oficial tendría que tratar sus
fuentes, licencia, binarios y herramientas explícitamente; una implementación
original inspirada en técnicas públicas mantiene otra identidad y necesita sus
propias pruebas. No se incorpora código ajeno por llamarlo «inspiración».

**ABIERTO:** implementación, calidad, tiempos GPU, latencia, VRAM y compatibilidad
experimental de una reconstrucción temporal en TELAR. Lo comprobado en esta entrega
es la consulta documental y el diagnóstico del código; el ejecutable no cambia.
