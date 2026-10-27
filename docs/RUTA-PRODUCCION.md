# Ruta hacia el estudio VFX y el render de producción

Estado del **27 de octubre de 2026**, después de los parámetros por nodo (`9b4da19`).
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
| `.tlar` | Exporta/inspecciona arranque y coordinador | Todavía no conserva escenas ni los ajustes del inspector |
| Bordes | MSAA nativo incorporado en esta entrega | Reduce aliasing de cobertura; no aumenta detalle geométrico |

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

P1 es el próximo avance de uso: hacer una escena real que pueda guardarse.
P2 sigue siendo la prioridad técnica del kernel (H2/H3 en RUMBO-KERNEL.md).
Ambas vías se coordinan: el compilador prepara un plan válido y el kernel decide
trabajo dentro de él. No hace falta terminar un administrador GPU general para
probar P3/P4 con una escena pequeña; tampoco debe posponerse su corrección por
buscar primero una imagen vistosa.

El hito visible de render posterior será una escena pequeña con geometría general,
sombras entre objetos y acumulación progresiva. Mantendremos el visor interactivo
para editar y un modo final que invierta más muestras en converger. No se promete
que cualquier escena de producción converja en tiempo real en una RTX 3060.

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

- [PLAN.md](PLAN.md): fases históricas de construcción y objetivo de esfera de fuego.
- [RUMBO-KERNEL.md](RUMBO-KERNEL.md): H1..H5 de autonomía, cola, regiones y efectos.
- [NODOS-PARAMETROS.md](NODOS-PARAMETROS.md): controles reales y límites del grafo.
- [RENDER-AREA.md](RENDER-AREA.md): iluminación actual, coste y límites de sombras.
- [ANTIALIAS.md](ANTIALIAS.md): diagnóstico y corrección de bordes de esta entrega.

Los planes anteriores se conservan como historial; esta página da el estado y
orden actual. Render progresivo, GI, proyectos completos y cola general siguen pendientes.
