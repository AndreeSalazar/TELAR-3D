# Houdini Apprentice → TELAR

Referencias públicas de datos e interoperabilidad:
[Partio, LAIKA y formatos/APIs oficiales](../referencias/partio-houdini-intercambio.md).
La investigación está anotada; no demuestra reconstrucción del núcleo ni
compatibilidad de TELAR con archivos de Houdini.

Houdini se organiza en **contextos**: redes de nodos especializadas. Aquí está cada uno, qué enseña y qué será en TELAR. La prioridad sigue a la estrella del norte (la esfera de fuego): primero lo que hace falta para llegar a ella.

| Prioridad | Significado |
|---|---|
| **P0** | la base: sin esto no hay nada |
| **P1** | la estrella: la esfera de fuego |
| **P2** | lo que hace bonito y útil al telar |
| **P3** | simulaciones grandes, después |
| **P4** | más adelante |
| **Fuera** | no por ahora (se dice, no se finge) |

## El mapa

| Houdini | Qué enseña | En TELAR | Prioridad |
|---|---|---|---|
| **SOPs** (geometría) | puntos, vértices, primitivas y atributos; nodos que transforman geometría | `nudo` + `telar-nodos`: caja, esfera, rejilla, transformar, unir, borrar | **P0** |
| **Wrangle / VEX** | un lenguaje que corre por cada punto, en paralelo | **HILO**, compilado a SPIR-V | **P0** |
| **Cocinado del grafo** | solo se recalcula lo que cambió | marcas de sucio en `nudo`, plan fijo en la GPU | **P0** |
| **Visor (viewport)** | ver la geometría al instante | render rasterizado simple en `telar-gpu` | **P0** |
| **Volúmenes y VDB** | rejillas de densidad, temperatura y velocidad | rejillas densas, después ladrillos dispersos | **P1** |
| **Pyro** (DOPs) | humo y fuego: advección, flotación, combustión, presión | el solver de la esfera de fuego, todo en la GPU | **P1** |
| **Karma** (volumétricos) | marcha de rayos, absorción, emisión, dispersión | render volumétrico con cuerpo negro | **P1** |
| **Scatter / Copy to Points** | repartir puntos y copiar geometría sobre ellos | Poisson disk y copiar a puntos, en la GPU | **P2** |
| **VOPs / ruido** | ruido Perlin, simplex, curl, fractal | biblioteca de ruido de HILO | **P2** |
| **POPs** (partículas) | emitir, fuerzas, edad, vida | partículas en la GPU, chispas de la esfera | **P2** |
| **Heightfields** | terrenos: erosión, máscaras | terrenos en la GPU | **P2** |
| **Vellum** (DOPs) | telas, pelo y cuerpos blandos con restricciones | XPBD en la GPU | **P3** |
| **FLIP** (DOPs) | líquidos con partículas y rejilla | FLIP en la GPU | **P3** |
| **RBD** (DOPs) | cuerpos rígidos, fractura | rígidos y Voronoi | **P3** |
| **TOPs / PDG** | grafos de tareas, trabajo por lotes | cola de tareas (cachear simulaciones, variantes) | **P4** |
| **CHOPs** | canales, señales y animación | curvas y señales para parámetros | **P4** |
| **COPs** | composición de imágenes | posproceso: *bloom*, grano, *tonemap* | **P4** |
| **LOPs / Solaris (USD)** | escenas en USD, iluminación | — | **Fuera** |
| **KineFX** | rigging y animación de personajes | — | **Fuera** |
| **Karma completo** | trazado de caminos de producción | — (solo volumétricos y PBR simple) | **Fuera** |

## Cómo se compara contra Houdini

Houdini Apprentice es la **referencia externa**, como el Ryzen lo es en BMO-X.

1. **La misma escena** en los dos: mismo nodo, mismos parámetros y misma semilla.
2. **Corrección:**
   - si Apprentice permite exportar esa geometría, se comparan los datos;
   - si no, se comparan **estadísticas**: recuento de puntos, caja que la contiene y promedio y desviación de cada atributo;
   - en volúmenes, **capturas** a la misma cámara.
3. **Rendimiento:** tiempo por fotograma en la misma máquina (Ryzen 5 5600X y RTX 3060), con el monitor de rendimiento de Houdini y el temporizador de la GPU en TELAR.
4. **Etiqueta honesta** en cada resultado:

| Etiqueta | Significa |
|---|---|
| **IGUAL** | misma salida dentro de la tolerancia |
| **MÁS RÁPIDO** | misma salida y menos tiempo, con la cifra |
| **DISTINTO** | sale otra cosa, y se explica por qué (otro algoritmo, otra semilla…) |
| **ABIERTO** | todavía no comparable, y se dice |

Cada nodo guarda su comparación en `referencias/<nodo>.md`.

## Lo que se aprende de Houdini (y lo que no)

- **Sí:** su modelo de datos, el comportamiento de cada nodo según su documentación, y la matemática de los artículos que cita o que son públicos.
- **No:** copiar su código, sus textos, sus archivos `.hip` ni sus nombres comerciales. TELAR usa sus propios nombres de nodos.
- **Licencia de Apprentice:** es para aprender y para uso no comercial. Úsala para comparar y aprender, no para producir trabajos pagados.

## Ampliación solicitada en octubre de 2026

El alcance inicial acotado de esta página es histórico. La petición de herramientas VFX más amplias se desarrolla en [VFX-PARA-ARTISTAS.md](VFX-PARA-ARTISTAS.md), con datos, HILO, volúmenes y restricciones. Las funciones antes marcadas Fuera pasan a investigación futura; no están implementadas por ese cambio de alcance.

## Estado del solver de humo

Existe un laboratorio MAC 2D con campos residentes y controles: [HUMO-MAC-2D.md](HUMO-MAC-2D.md). Es una base numérica propia, sin equivalencia Pyro demostrada y sin integración todavía en el grafo general.

## Categorías visibles en TELAR

[ESTUDIO-CATEGORIAS.md](ESTUDIO-CATEGORIAS.md) organiza el centro nativo en ocho áreas. Es navegación a herramientas independientes y un mapa de alcance; no implica compatibilidad con contextos/proyectos Houdini ni que todas las tarjetas estén implementadas.
