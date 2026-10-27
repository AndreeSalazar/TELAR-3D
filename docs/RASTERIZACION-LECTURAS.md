# Rasterización: historia, matemática y uso para TELAR

Bibliografía inicial y guía de estudio, consultada el **27 de octubre de 2026**.
No es un catálogo de todos los documentos publicados ni una afirmación de haber
leído o reproducido íntegramente cada artículo. Las fuentes son artículos originales,
documentos del fabricante, del desarrollador del producto o especificaciones.

## Dos aclaraciones antes de estudiar

**La GPU no acepta solamente rasterización.** Vulkan distingue trabajo gráfico,
cómputo y, con las capacidades/extensiones correspondientes, trazado de rayos.
TELAR ya usa compute para generar y deformar geometría antes de dibujarla.
Ver [pipelines](https://docs.vulkan.org/spec/latest/chapters/pipelines.html),
[dispatch](https://docs.vulkan.org/spec/latest/chapters/dispatch.html) y
[ray tracing](https://docs.vulkan.org/guide/latest/extensions/ray_tracing.html).

**La rasterización sigue vigente.** La especificación actual de Vulkan define
su cobertura, interpolación y multisampling. Que un renderizador de cine haya
cambiado de arquitectura no demuestra que toda la industria la abandonara.
Pixar documenta su evolución desde Reyes, pasando por combinaciones con rayos,
hasta RIS; eso describe la historia de RenderMan, no una retirada universal de
los rasterizadores. [Vulkan](https://docs.vulkan.org/spec/latest/chapters/primsrast.html),
[Pixar](https://renderman.pixar.com/the-evolution-of-renderman).

## Línea histórica y documentos

Las fechas siguientes son las de publicación o lanzamiento descritas en la
fuente, no las fechas de indexación de un buscador. Son hitos seleccionados;
la historia incluye otros investigadores, fabricantes y arquitecturas.

| Fecha | Fuente primaria | Qué estudiar | Relación con TELAR |
|---|---|---|---|
| 1965 | J. E. Bresenham, [Algorithm for Computer Control of a Digital Plotter](https://ibm-1401.info/Pics3/bresenham1965.pdf) | Trazado incremental de líneas con aritmética discreta | Antecedente para entender precisión y recorridos; es un plotter, no una GPU 3D |
| 1982 | James H. Clark, [The Geometry Engine: A VLSI Geometry System for Graphics](https://graphics.stanford.edu/courses/cs148-10-summer/docs/1982--clark--geometry_engine.pdf) | Aceleración geométrica dedicada | El hardware gráfico especializado precede a la etiqueta comercial moderna GPU |
| 1987 | Cook, Carpenter y Catmull, [The Reyes Image Rendering Architecture](https://graphics.pixar.com/library/Reyes/paper.pdf) | Micropolígonos, localidad, sombreado y visibilidad muestreada | Referencia directa para la dirección cinematográfica y superficies adaptativas |
| 1988 | Juan Pineda, [A Parallel Algorithm for Polygon Rasterization](https://people.csail.mit.edu/ericchan/bib/pdf/p17-pineda.pdf) | Funciones de borde lineales, incrementos y paralelismo | Fundamento de una referencia matemática de cobertura de triángulos |
| 1999 | NVIDIA, [Transform and Lighting](https://developer.download.nvidia.com/assets/gamedev/docs/TransformAndLighting.pdf) | Traslado de transformación e iluminación al procesador gráfico | Distinguir esas etapas de rasterizar y de la aplicación CPU |
| 2004 | Cem Cebenoyan, [GPU Gems, capítulo 28: Graphics Pipeline Performance](https://developer.nvidia.com/gpugems/gpugems/part-v-performance-and-practicalities/chapter-28-graphics-pipeline-performance) | Localizar el cuello de botella y variar cargas por etapa | Método de medida; los detalles de aquel hardware no se extrapolan directamente |
| 2006 | NVIDIA, [GeForce 8800 GPU Architecture Technical Brief](https://www.nvidia.com/content/PDF/Geforce_8800/GeForce_8800_GPU_Architecture_Technical_Brief.pdf) | Shaders unificados y soporte de cómputo | La GPU programable no queda reducida a cubrir triángulos |
| 2011 | Samuli Laine y Tero Karras, [High-Performance Software Rasterization on GPUs](https://research.nvidia.com/publication/2011-08_high-performance-software-rasterization-gpus) | Rasterizador programado en GPU, orden, cobertura sin huecos y MSAA | Experimento de flexibilidad; no prueba que compute supere al hardware fijo |

El artículo de Clark quedó localizado, pero la descarga completa desde Stanford
falló por timeout durante esta consulta. No se presenta como lectura completa.
El contexto histórico del Geometry Engine también consta en
[la reseña institucional de Stanford](https://engineering.stanford.edu/about/history/heroes/2013-heroes/james-h-clark).
De los demás artículos se consultaron resúmenes, introducciones o secciones
pertinentes; su reproducción experimental sigue pendiente.

NVIDIA presentó GeForce 256 en 1999 bajo su definición comercial de GPU. Eso no
significa que antes no existiesen procesadores gráficos. No fijamos una única
fecha de nacimiento de toda la categoría. [Cronología de NVIDIA](https://www.nvidia.com/content/timeline/time_99.html).

## Documentación vigente y temas transversales

| Fuente | Para qué sirve |
|---|---|
| [Khronos: Rasterization](https://docs.vulkan.org/spec/latest/chapters/primsrast.html) | Reglas de puntos, líneas, triángulos, muestras y cobertura; referencia normativa para nuestro backend |
| [Khronos: Fragment Operations](https://docs.vulkan.org/spec/latest/chapters/fragops.html) | Profundidad, stencil, máscaras y operaciones de fragmentos; estudiar condiciones de pruebas tempranas |
| [Khronos: Dispatching Commands](https://docs.vulkan.org/spec/latest/chapters/dispatch.html) | Compute directo/indirecto y ejecución de kernels fuera de la rasterización |
| [Khronos: Ray Tracing](https://docs.vulkan.org/guide/latest/extensions/ray_tracing.html) | Capacidades y comandos adicionales; no están integrados todavía en TELAR |
| [PBRT: Sampling Theory](https://www.pbr-book.org/4ed/Sampling_and_Reconstruction/Sampling_Theory) | Aliasing, posiciones de muestra y reconstrucción; separar precisión geométrica de calidad de imagen |
| [W3C: SVG Paths](https://www.w3.org/TR/SVG/paths.html) | Curvas y arcos como descripción de forma; al visualizarlos también hay que resolver píxeles |
| [Pixar: Evolution of RenderMan](https://renderman.pixar.com/the-evolution-of-renderman) | Contextualizar Reyes, enfoques híbridos y RIS sin confundirlos con nuestra implementación |

Las páginas `latest` son documentos vivos: comprobar versión y capacidades antes
de aplicar una función. Que aparezca en la especificación no prueba que nuestra
GPU/controlador la admita ni que TELAR la haya habilitado.

## La matemática que conviene comprender primero

Para una arista de extremos A=(x0,y0), B=(x1,y1), una función posible es:

```
E(x,y) = (x-x0)*(y1-y0) - (y-y0)*(x1-x0)
E(x+1,y) = E(x,y) + (y1-y0)
E(x,y+1) = E(x,y) - (x1-x0)
```

El signo clasifica el lado de la recta. Tres aristas, una orientación coherente
y una regla para empates permiten decidir cobertura de un triángulo. La forma
incremental evita repetir toda la evaluación en posiciones vecinas. Pineda es
la lectura inicial para esta idea; no se afirma que su implementación histórica
describa los circuitos de cada GPU actual.

Una implementación correcta debe definir bordes compartidos, recorte, precisión,
triángulos degenerados, interpolación y muestras. Una fórmula breve no sustituye
esas reglas. El rasterizador Vulkan actual ya las aplica bajo su contrato.

La cobertura no calcula por sí sola la iluminación. Este esquema es conceptual;
el hardware puede solapar o adelantar operaciones cuando el contrato lo permite:

```
geometría → transformación/recorte → cobertura de muestras
         → sombreado y pruebas de visibilidad → mezcla/resolución → imagen
```

En MSAA se comprueban varias posiciones dentro de cada píxel y se resuelven sus
valores. Una curva matemática continua sigue necesitando muestreo al mostrarse
en una pantalla. Muestreo espacial, sombras, reflejos y estabilidad temporal son
problemas relacionados, pero no idénticos; ver PBRT y la especificación Vulkan.

## Cómo estudiar y optimizar TELAR

Orden de lectura sugerido: **Vulkan rasterización → Pineda → PBRT muestreo →
Reyes → GPU Gems rendimiento**. Después, ampliar hacia implementación programada
y transporte de luz. Se mantienen las referencias CPU pequeñas como comprobación;
eso no traslada el trabajo del visor vivo al host.

| Experimento propuesto | Qué comparar | Condición antes de adoptarlo |
|---|---|---|
| Referencia matemática de cobertura | Triángulos compartidos, degenerados, recorte y posiciones de muestra | Sin huecos ni dobles coberturas indebidas bajo las reglas elegidas |
| MSAA 1x/2x/4x | Contorno, estabilidad, memoria y tiempo GPU | Añadir selección explícita de 2x para el benchmark; hoy se solicita hasta 4x o se desactiva |
| Descartar trabajo invisible | Coste de clasificación frente al dibujo evitado | No descartar objetos visibles; mejorar el tiempo total en la escena medida |
| Pruebas tempranas/profundidad | Fragmentos evitados y coste de una pasada adicional, si la hubiera | Respetar semántica del shader y demostrar beneficio; no asumirlo |
| Detalle adaptativo de superficies | Error de silueta en pantalla y triángulos emitidos | Presupuesto acotado y sin grietas ni saltos inaceptables |
| Localidad por regiones de pantalla | Accesos, orden, sobre-dibujo y memoria | Medición y orden correcto, especialmente con mezcla/transparencia |
| Prototipo de rasterizador compute | Flexibilidad, exactitud, coste de memoria y tiempo contra Vulkan | Solo una ruta experimental; no reemplazar el hardware por promesas |

Estos experimentos **no están implementados** por añadir esta guía. No se prometen
ganancias universales. El estudio de Laine/Karras demuestra que se puede construir
una ruta programada; sus cifras de 2011 no predicen las de nuestra RTX 3060.

En TELAR ya existen cómputo residente, despacho indirecto, shaders GGX y MSAA de
hardware. Nuestra medición 1280×720 pasa de 0,8330 a 1,4495 ms con MSAA 4x en la
escena documentada: **DISTINTO**, más coste y mejor contorno. No es rendimiento
de una producción general. Ver [ANTIALIAS.md](ANTIALIAS.md).

La decisión de arquitectura sigue siendo combinar herramientas según su utilidad:
rasterización para el visor, compute para geometría/simulación/coordinación y
una futura ruta progresiva de rayos. Datos, límites y decisiones permanecen en
GPU donde esté implementado; la CPU conserva preparación, entradas y envíos.
Ver [RUTA-PRODUCCION.md](RUTA-PRODUCCION.md). Esta guía no cambia código ejecutable.
