# Ruta hacia el estudio VFX y el render de producción

Estado del **27 de octubre de 2026**, con parámetros por nodo, MSAA 1x/2x/4x y persistencia del estudio en .tlar v2.
Objetivo personal: TELAR 3D con un flujo procedural inspirado en Houdini y una
dirección visual cinematográfica inspirada en RenderMan. Código y contratos propios;
no se afirma compatibilidad ni equivalencia con esos productos.

## Dónde estamos y por qué

Estamos en un **prototipo funcional de estudio procedural y visor interactivo**.
La base ya ejecuta geometría en GPU, pero la escena está limitada a cuatro etapas
fijas. Una superficie bonita y fluida no demuestra transporte de luz completo,
fiabilidad de una producción ni un editor de grafos general.

Se construyó primero esta ruta pequeña para verificar los contratos: SPIR-V válido,
Vulkan aislado, geometría residente, trabajo indirecto, parámetros vivos, caché e
interacción. Así se puede comprobar cada ampliación contra una referencia antes
de añadir escenas, efectos y millones de rayos con dependencias complejas.

| Pieza | Estado comprobado | Límite actual |
|---|---|---|
| Ventana y backend | Win32, Vulkan, validación y pruebas con redimensionado | Otras plataformas y fabricantes requieren integración/pruebas |
| Kernel GPU | Arranque TLAR, servicios, indirectos y caché | Sin cola general ni asignador de regiones todavía |
| Geometría residente | Rejilla → Onda → Relieve | No es un grafo GPU de recursos arbitrarios |
| HILO | IR tipado, intérprete y SPIR-V; escalares vivos | Sin lenguaje textual ni atributos generales |
| Interfaz | CSS nativo e inspector propio por nodo | No crea instancias ni reconecta cables aún |
| Render | GGX, luz de área, autosombra de campo de alturas | Sin BVH, GI, trazado de caminos ni acumulación HDR |
| `.tlar` | V1 arranque; V2 conserva parámetros, cámara, fase y posiciones del estudio | V2 representa las cuatro etapas fijas, no escenas arbitrarias |
| Bordes | MSAA nativo seleccionable 1x/2x/4x; ver CALIDAD-RENDER.md | Reduce aliasing de cobertura; no aumenta detalle geométrico |

## Orden propuesto y criterios de aceptación

| Orden | Entrega | Hecho cuando |
|---|---|---|
| P1 | Escena persistente y grafo editable | Dos Ondas con parámetros independientes; crear/conectar/desactivar nodos; guardar y reabrir `.tlar` reproduce la escena |
| P2 | Cola GPU acotada y regiones de memoria | Tareas, dependencias, lleno/vacío, referencias antiguas y agotamiento se resuelven y prueban dentro de reservas existentes |
| P3 | Geometría general e intersecciones | BVH y rayos de sombra entre varios objetos coinciden con una referencia; construcción/actualización y límites medidos |
| P4 | Render progresivo | Acumulación HDR y muestras por píxel residentes; cambios de escena/cámara/material/luz invalidan correctamente; una escena pequeña converge |
| P5 | Transporte de luz | Rebotes acotados, reflejos y muestreo de luz/BRDF; comparación numérica y visual con casos de referencia |
| P6 | Salida de producción | Texturas, gestión de color, exportación HDR, pases, secuencias y recuperación de errores producen entregables reproducibles |
| P7 | Efectos y materiales avanzados | Partículas, humo/fuego y materiales adicionales tienen contratos, referencias y presupuestos; un efecto completo se construye desde la interfaz |

P1 avanza en dos partes: **persistencia del estudio implementada** (Guardar/F5,
Abrir/F9, .tlar v2; ver ESCENAS.md). Quedan identificadores estables, instancias
independientes y conexiones editables: P1 completo sigue abierto.
P2 sigue siendo la prioridad técnica del kernel (H2/H3 en RUMBO-KERNEL.md).
Ambas vías se coordinan: el compilador prepara un plan válido y el kernel decide
trabajo dentro de él. No hace falta terminar un administrador GPU general para
probar P3/P4 con una escena pequeña; tampoco debe posponerse su corrección por
buscar primero una imagen vistosa.

El hito visible de render posterior será una escena pequeña con geometría general,
sombras entre objetos y acumulación progresiva. Mantendremos el visor interactivo
para editar y un modo final que invierta más muestras en converger. No se promete
que cualquier escena de producción converja en tiempo real en una RTX 3060.

## Dos perfiles de calidad: edición y producción

Decisión del usuario: usar **MSAA 2x como referencia para el visor interactivo**
y permitir más trabajo GPU en la salida de producción. El selector actual sigue
ofreciendo 1x/2x/4x; su valor inicial todavía es 4x. Esta decisión queda anotada
como dirección del perfil, sin afirmar que ya existe un modo final.

La propuesta de «16x» se divide en dos controles con unidades diferentes:

| Control | Qué mejora | Plan |
|---|---|---|
| Muestras MSAA | Cobertura rasterizada de bordes | 2x para editar; estudiar 8x/16x solo si los formatos y la GPU lo admiten y la comparación demuestra utilidad |
| Muestras por píxel (spp) del trazador | Estimación del transporte de luz, con integración de cámara y materiales | Acumular en HDR; 16 spp puede ser un punto inicial de prueba, no un techo ni una garantía de calidad |

MSAA 16x no equivale a 16 caminos por píxel y no añade iluminación global,
reflejos, profundidad de campo o desenfoque de movimiento por sí solo.
Vulkan permite consultar las cantidades admitidas para cada configuración de
imagen; no debe activarse una cantidad que la GPU no anuncie. El backend actual
de TELAR acepta exclusivamente 1/2/4; 8x y 16x aún no están implementados.

Para el nodo Salida del futuro modo de producción se propone exponer:

- Mínimo y máximo de muestras por píxel, con recuento efectivo visible.
- Umbral de error/ruido para muestreo adaptativo, acompañado de una vista de
  muestras usadas; detenerse al cumplir el criterio o alcanzar el máximo.
- Límite de rebotes y controles separados para cámara, luces y materiales
  cuando sus algoritmos existan; no mostrar controles que todavía no tienen efecto.
- Resolución, tiempo GPU, memoria estimada y motivo de parada. Más presupuesto
  significa poder tardar más, conservando límites de memoria, cancelación y
  recuperación de errores.

La acumulación y sus estadísticas deberán permanecer en GPU; al editar escena,
cámara, luz o material se invalidará el historial correspondiente. Se decidirá el
trabajo adicional por necesidad de convergencia, en lugar de repetir trabajo ya
suficiente. Los umbrales y tamaños de lote deben verificarse con escenas reales.

**Siguiente entrega de render, después de la base de escena:** una escena pequeña
con intersecciones y sombras entre objetos, seguida de acumulación HDR con
recuento de muestras. Comparar 16/64/256 spp con una referencia de mayor muestreo,
medir error, ruido, tiempo y memoria; esos valores son casos de prueba, no presets
profesionales prometidos. Después añadir muestreo adaptativo. Nada de esto se
presenta como integrado en el visor actual.

Referencias primarias: [capacidades de imágenes Vulkan](https://docs.vulkan.org/refpages/latest/refpages/source/VkImageFormatProperties.html)
y [muestreo y optimización en RenderMan](https://renderman.pixar.com/debugging-and-optimization).
RenderMan expone mínimo, máximo y varianza de píxel; sirven de inspiración para
un contrato propio de TELAR, sin afirmar equivalencia con su implementación.

## Qué significa mejorar con evidencia

- Corregir aliasing de contorno con MSAA ahora; revisar sombras/reflejos con sus
  propias pruebas cuando corresponda. No confundir un filtro con geometría nueva.
- Conservar datos y acumulaciones válidas en VRAM; invalidar por dependencias,
  no por parecido visual ni por asumir que el GPU «recuerda» un patrón.
- Medir GPU por pasadas, CPU de preparación/envío, transferencias y memoria.
  Menos CPU no garantiza menor latencia; no imponemos un porcentaje 10/90.
- Etiquetar IGUAL, MÁS RÁPIDO, DISTINTO o ABIERTO con escena y máquina.
- Verificar errores y presupuestos antes de ampliar calidad. Sin fechas ni
  porcentajes de avance inventados: cada entrega se cierra con su criterio.

La calidad cinematográfica depende de materiales, luz, geometría, composición,
transporte y herramientas artísticas. El kernel administra el trabajo; disponer
de un kernel propio no sustituye esos componentes.

## Documentos que se complementan

- [FSR-RECONSTRUCCION.md](FSR-RECONSTRUCCION.md): estudio de versiones FSR y
  requisitos de una futura reconstrucción temporal GPU; todavía no integrada.

- [RASTERIZACION-LECTURAS.md](RASTERIZACION-LECTURAS.md): fuentes históricas y
  vigentes; guía de estudio y experimentos medibles para el visor.

- [ESCENAS.md](ESCENAS.md): guardado/apertura del estudio, formato v2 y pruebas.
- [PLAN.md](PLAN.md): fases históricas de construcción y objetivo de esfera de fuego.
- [RUMBO-KERNEL.md](RUMBO-KERNEL.md): H1..H5 de autonomía, cola, regiones y efectos.
- [NODOS-PARAMETROS.md](NODOS-PARAMETROS.md): controles reales y límites del grafo.
- [RENDER-AREA.md](RENDER-AREA.md): iluminación actual, coste y límites de sombras.
- [ANTIALIAS.md](ANTIALIAS.md): diagnóstico y corrección de bordes de esta entrega.

Los planes anteriores se conservan como historial; esta página da el estado y
orden actual. Render progresivo, GI, proyectos completos y cola general siguen pendientes.
