# TELAR render

Renderizador original, aislado en `no_std + alloc`. Solo depende del emisor
`hilo-spirv`; el backend Vulkan recibe shaders y un bloque opaco de 128 bytes.
La geometría procedural sigue residente en GPU y la interfaz CSS es independiente.

## Dirección visual y primer avance

Buscamos materiales e iluminación con cuidado cinematográfico, tomando como
referencias públicas [PxrSurface](https://renderman.atlassian.net/wiki/spaces/REN27/pages/542213836)
y [Physically-Based Shading at Disney](https://www.disneyanimation.com/publications/physically-based-shading-at-disney/).
Es implementación propia; no integra RenderMan ni es compatible con sus escenas
o materiales. No se presenta como resultado equivalente a una película Pixar.

**Implementado:** rasterización, material metálico/rugoso, distribución GGX,
Fresnel Schlick, visibilidad Schlick-GGX aproximada y exposición con mapeo Reinhard.
El estudio usa luz de área de cuatro muestras y autosombras aproximadas de la rejilla
residente; conserva relleno direccional y ambiente hemisférico aproximado.
La variante histórica de dos luces direccionales sigue disponible.
La salida sRGB se codifica una vez: en shader para UNORM, en hardware para sRGB.
La matemática de microfacetas se referencia en [PBRT](https://www.pbr-book.org/4ed/Reflection_Models/Roughness_Using_Microfacet_Theory);
el código SPIR-V y la composición son propios.

La cámara y los materiales se empaquetan en CPU; iluminación y respuesta del
material se evalúan por fragmento en GPU. Los perfiles cobre, porcelana y esmalte
azul del visor cambian rugosidad y metalicidad junto con el color. El tercer
perfil no tiene transmisión ni refracción de hielo todavía.

## Contrato y límites

Bloque: matriz (64 B), ojo/exposición (16), luz/intensidad (16), color lineal/rugosidad
(16), metal/ambiente/codificación (16). Se validan valores finitos y rangos del
material. Las normalizaciones y divisiones tienen límites inferiores explícitos.
La rugosidad efectiva mínima es 0,08 para evitar una singularidad especular.

No hay iluminación global, trazado de caminos, texturas, pelo, subsurface, volúmenes
ni denoiser. Las sombras actuales son de campo de alturas: 32 pasos, sesgo vertical
y cuatro muestras fijas; pueden perder oclusores y no sirven para mallas arbitrarias.
Ver [contrato, mediciones y ruta progresiva](../../docs/RENDER-AREA.md).

## Riesgo y consumo

Ver los SEMAFORO.md de esta carpeta y sus subcarpetas. El shader consume ALU por
fragmento y ancho de banda de vértices; el coste crece con área visible, resolución
y sobre-dibujo. No reserva VRAM directamente. El backend mantiene destinos, profundidad,
malla y presentación. La medición RTX 3060 de la escena documentada dio 0,834 ms con cuatro muestras
y sombras frente a 0,048 ms sin sombras; DISTINTO, más calidad y mayor coste.
El pico de VRAM y otras escenas siguen ABIERTOS.

Las pruebas validan SPIR-V, rechazo de materiales inválidos y cambios reales de
píxeles por material/exposición en RTX 3060/Windows. Otros dispositivos quedan ABIERTOS.

## Base experimental de rayos

`rayos::intersectar` emite pruebas bilaterales rayo–triángulo; verificadas en GPU
contra referencia f64, todavía fuera del visor, sin BVH ni acumulación. Contrato,
costes y límites numéricos en [INTERSECCIONES-GPU.md](../../docs/INTERSECCIONES-GPU.md).

`escena_rayos::Trabajo` ya genera un render directo por lotes: hasta 32 triángulos
constantes, cámara, luz de área, sombras y 1..1024 spp dentro de un despacho.
Acumula en GPU y entrega HDR lineal, contador y revisión sRGB; sin historial
progresivo ni BVH. [Uso, matemáticas y coste](../../docs/RENDER-POR-LOTES.md).
