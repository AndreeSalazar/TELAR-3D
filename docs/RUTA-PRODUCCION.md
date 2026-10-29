# Ruta hacia el estudio VFX y el render de producción

Estado del **28 de octubre de 2026**: edición numérica, MSAA solicitado hasta 64x, escenas .tlar v2/v3/v4 y editor de objetos/luz con vista GPU, .tlar v5 y HDR asíncrono cancelable; el grafo procedural conserva su modo propio.
Objetivo personal: TELAR 3D con un flujo procedural inspirado en Houdini y una
dirección visual cinematográfica inspirada en RenderMan. Código y contratos propios;
no se afirma compatibilidad ni equivalencia con esos productos.

## Dónde estamos y por qué

Estamos en un **prototipo funcional de estudio procedural y visor interactivo**.
La base ejecuta geometría en GPU y conecta hasta ocho operaciones geométricas
sobre una rejilla, con instancias independientes. Una superficie bonita y fluida no demuestra transporte de luz completo,
fiabilidad de una producción ni un editor de grafos general.

Se construyó primero esta ruta pequeña para verificar los contratos: SPIR-V válido,
Vulkan aislado, geometría residente, trabajo indirecto, parámetros vivos, caché e
interacción. Así se puede comprobar cada ampliación contra una referencia antes
de añadir escenas, efectos y millones de rayos con dependencias complejas.

| Pieza | Estado comprobado | Límite actual |
|---|---|---|
| Ventana y backend | Win32, Vulkan, validación y pruebas con redimensionado | Otras plataformas y fabricantes requieren integración/pruebas |
| Kernel GPU | Arranque TLAR, servicios, indirectos y caché | Sin cola general ni asignador de regiones todavía |
| Geometría residente | Rejilla y hasta ocho Ondas/Relieves conectables, compilados a HILO | Un único campo de alturas; sin mallas/recursos arbitrarios |
| HILO | IR tipado, intérprete y SPIR-V; escalares vivos | Sin lenguaje textual ni atributos generales |
| Interfaz | CSS nativo, números editables, instancias, puertos y omisión | Sin borrado/deshacer ni mezcla de geometrías |
| Render | Visor GGX/área/autosombra; render por lotes de 26 triángulos con sombras y HDR | Sin BVH ni GI; historial progresivo implementado en el visor de objetos |
| `.tlar` | V1 arranque; V2 cadena histórica; V3 grafo editable; V4 peticiones MSAA altas; V5 escena de dos cajas, luz y cámara | Un dominio de rejilla, sin recursos externos de producción |
| Bordes | MSAA 2x inicial, peticiones numéricas hasta 64x según soporte; ver CALIDAD-RENDER.md | Reduce aliasing de cobertura; no aumenta detalle geométrico |

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

**P1 básico implementado en el dominio de rejilla:** identidades estables, dos
Ondas con valores propios, creación/conexión/omisión y reapertura v3. Ver
GRAFO-EDITABLE.md. Esto no cierra todavía escenas generales con varios objetos,
mezcla de geometrías, atributos o recursos externos.
**P3 avanzado, sin cerrar:** escena de ensayo con 26 triángulos, rayos de cámara y sombras entre objetos. **P4 base implementada para iluminación directa:** historial HDR residente por píxel, contador, parada e invalidación; ver [VISOR-PROGRESIVO.md](VISOR-PROGRESIVO.md). **P6 parcial:** PFM lineal y PPM sRGB de revisión. Los controles nativos de dos cajas/luz/salida ya guardan .tlar v5; faltan BVH, recursos de malla generales, muestreo adaptativo y transporte completo. Ver [EDITOR-OBJETOS.md](EDITOR-OBJETOS.md). Ver [RENDER-POR-LOTES.md](RENDER-POR-LOTES.md).

P2 sigue siendo la prioridad técnica del kernel (H2/H3 en RUMBO-KERNEL.md).
Ambas vías se coordinan: el compilador prepara un plan válido y el kernel decide
trabajo dentro de él. No hace falta terminar un administrador GPU general para
probar P3/P4 con una escena pequeña; tampoco debe posponerse su corrección por
buscar primero una imagen vistosa.

El ensayo por lotes ya muestra sombras entre objetos. El hito visible posterior
es integrar geometría general en el editor y añadir acumulación progresiva. Mantendremos el visor interactivo
para editar y un modo final que invierta más muestras en converger. No se promete
que cualquier escena de producción converja en tiempo real en una RTX 3060.

## Dos perfiles de calidad: edición y producción

Decisión del usuario: usar **MSAA 2x como referencia para el visor interactivo**
y permitir más trabajo GPU en la salida de producción. El selector actual sigue
ofreciendo 1x/2x/4x; su valor inicial es ahora 2x. El modo final sigue pendiente.

La propuesta de «16x» se divide en dos controles con unidades diferentes:

| Control | Qué mejora | Plan |
|---|---|---|
| Muestras MSAA | Cobertura rasterizada de bordes | 2x para editar; solicitudes hasta 64 implementadas con límite real de GPU; estudiar calidad/coste de niveles altos |
| Muestras por píxel (spp) del trazador | Estimación del transporte de luz, con integración de cámara y materiales | Acumular en HDR; 16 spp puede ser un punto inicial de prueba, no un techo ni una garantía de calidad |

MSAA 16x no equivale a 16 caminos por píxel y no añade iluminación global,
reflejos, profundidad de campo o desenfoque de movimiento por sí solo.
Vulkan permite consultar las cantidades admitidas para cada configuración de
imagen; no debe activarse una cantidad que la GPU no anuncie. El backend actual
de TELAR acepta peticiones 1/2/4/8/16/32/64 y aplica el mayor nivel admitido que
no las exceda. En la RTX 3060 probada el visor aplica hasta 8; pedir 32 se conserva
como petición y se muestra junto a los 8 efectivos. Ya hay un integrador de iluminación directa por lotes separado; no implementa caminos con rebotes ni cambia el MSAA del visor.

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

**Siguiente entrega:** acelerar intersecciones de escenas mayores y añadir el
primer transporte indirecto validado. La base del visor ya conserva HDR entre
cuadros; aún falta medir error/ruido frente a referencias y decidir trabajo por
convergencia. El render final sigue siendo un trabajo independiente por lotes.
Después ampliar materiales y salida, manteniendo pruebas de cada componente.

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

El render final del editor ya se divide en lotes con un trabajador separado,
progreso y cancelación: [RENDER-ASINCRONO.md](RENDER-ASINCRONO.md). Esto mejora
la coordinación de la ventana. La entrega posterior VISOR-PROGRESIVO.md añade
el historial del visor; muestreo adaptativo y transporte completo siguen pendientes.

El visor de objetos acumula ahora entre fotogramas y conserva el historial al
cambiar exposición. Aún faltan BVH, control adaptativo de ruido y transporte con
rebotes; no se presenta como render de producción terminado.

## Primera base visual de fuego

[Esfera procedural GPU](FUEGO-PROCEDURAL.md): integra un volumen analítico y expone sus controles. Sigue separada del render de objetos y no completa el solver físico ni el render de producción. [VFX-PARA-ARTISTAS.md](VFX-PARA-ARTISTAS.md) concreta los próximos criterios para campos, HILO y restricciones.

## Humo 2D: primera simulación persistente

El laboratorio [HUMO-MAC-2D.md](HUMO-MAC-2D.md) completa una primera advección/flotación/proyección en GPU y pruebas contra referencia. No completa humo 3D, fuego físico ni el render de producción. Próximos pasos: presión con control de error, paso temporal/subpasos, campos 3D y su integración con el visor volumétrico; HILO editable, atributos generales, GI y restricciones siguen abiertos.

## Control de error de presión integrado

[PRESION-ADAPTATIVA.md](PRESION-ADAPTATIVA.md) añade reducción de residual, tolerancias y parada de actualizaciones decidida en GPU. Los despachos siguen acotados desde CPU; eliminación de huecos, solver más eficaz, subpasos y humo 3D permanecen abiertos.

## Rama de explosión volumétrica

El recorrido completo queda registrado en [PLAN-EXPLOSION-PRODUCCION.md](PLAN-EXPLOSION-PRODUCCION.md).
La primera base [HUMO-3D.md](HUMO-3D.md) ya transporta y proyecta campos MAC a 32³
en GPU. No cierra P7 de esta ruta: faltan estabilidad temporal, colisiones,
combustión, iluminación de volumen, secundarios, cachés y secuencias finales.

La rama 3D añade [tiempo y esfera estática](TIEMPO-COLISION-3D.md), con dt decidido en GPU y subpasos acotados. P2 avanzado y la explosión física completa siguen abiertos.

P3 de explosión dispone de una primera [fuente pulsada y expansión compensada](PULSO-EXPANSIVO-3D.md). No añade todavía combustible/reacción ni render volumétrico de producción.

Actualización 29-10-2026: [PLANO-VFX.md](PLANO-VFX.md) integra combustible,
reacción, enfriamiento, secundarios con historial, dispersión simple y caché
reanudable/secuencias HDR. Las notas anteriores describen entregas históricas.
El estado vigente sigue siendo parcial a 32³: faltan regiones activas, colisiones
generales, transporte completo, herramientas artísticas y salida de producción.
