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
Fresnel Schlick, visibilidad Schlick-GGX aproximada, dos luces direccionales sin
sombras, ambiente hemisférico aproximado y exposición con mapeo Reinhard.
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

No hay iluminación global, sombras de área, trazado de caminos, texturas, pelo,
subsurface, volúmenes ni denoiser. Son futuras etapas; no activar todo a la vez.
Siguiente avance de render: sombras y luz de área con una escena verificable;
después acumulación de muestras y transporte de luz. La cola GPU sigue siendo
el siguiente hito del coordinador del kernel.

## Riesgo y consumo

Ver los SEMAFORO.md de esta carpeta y sus subcarpetas. El shader consume ALU por
fragmento y ancho de banda de vértices; el coste crece con área visible, resolución
y sobre-dibujo. No reserva VRAM directamente. El backend mantiene destinos, profundidad,
malla y presentación. No hay una medición comparativa que demuestre aceleración.

Las pruebas validan SPIR-V, rechazo de materiales inválidos y cambios reales de
píxeles por material/exposición en RTX 3060/Windows. Otros dispositivos quedan ABIERTOS.
