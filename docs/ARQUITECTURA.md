# Arquitectura: la GPU manda

**Este documento describe la arquitectura objetivo.** La Fase 4.5 ya incorpora
una ruta residente de rejilla con generación, deformación y dibujo indirecto,
dos juegos de recursos y compute pregrabado. El grafo general de NUDO continúa
en CPU como referencia; ya existe el primer IR tipado de HILO con intérprete CPU
y backend SPIR-V. La sintaxis textual y el solver de fuego siguen pendientes.
Ver [REVISION-GPU.md](REVISION-GPU.md) para el diagnóstico histórico y el avance.

La rejilla ya incluye caché de topología decidida por GPU y parámetros mapeados
persistentemente. Cambiar fase o amplitud no obliga a regenerar sus índices.
El ejecutor acepta planes de 2..=32 operaciones sobre el ABI actual: cada
operación declara dominio y nombre; las dependencias siguen su orden y las
barreras entre etapas. El visor incorpora una operación adicional de relieve.
Los grafos con recursos arbitrarios siguen pendientes. El relieve del visor se
compila desde el IR HILO al preparar la escena. `hilo` usa `hilo-spirv` y la
matemática pura de `nudo`; `telar-app` pasa los bytes al backend GPU aislado.

El coordinador del visor ya pertenece a `telar-kernel`, una pieza `no_std` que
emite SPIR-V y no conoce Vulkan. El control creció a 128 bytes: conserva los
primeros 16 u32 e incorpora un buzón de servicios residente, versión y contadores.
Trabajadores GPU pueden pedir rehacer topología, suspender y reanudar trabajo.
La próxima pasada del coordinador consume la petición y publica argumentos sin
descargarla al host. Las pasadas de dominio `Servicio` ejecutan una invocación
incluso suspendidas, por lo que pueden solicitar reanudación. El buzón admite una
entrada y productores serializados; no es todavía una cola general concurrente.
El contrato y las limitaciones están en `crates/telar-kernel/README.md`.
El contrato de arranque ya está implementado en [ARRANQUE-GPU.md](ARRANQUE-GPU.md):
cabecera inmutable en enlace 4, perfil físico consultado y guardián GPU que anula
trabajo y servicios si rechaza el contrato. El visor tiene ahora 10 buffers
residentes, con las mismas 2 grabaciones, 8 pipelines y 2 mapeos. La cola general
y la memoria por regiones siguen propuestas en [RUMBO-KERNEL.md](RUMBO-KERNEL.md).

Ya existe una primera interfaz: `telar-ui` produce estado y
triángulos; la app conecta sus controles con los parámetros GPU. El backend
dibuja el overlay en un pipeline 2D y mantiene sus propios buffers por juego.
La red tiene nodos seleccionables y arrastrables en un lienzo bajo el inspector;
las conexiones de ejecución siguen fijas, todavía sin edición de topología ni atributos.

`telar-estilo` separa la apariencia del contenido con una hoja CSS nativa:
variables, clases, estados, cascada, herencia tipográfica y cajas `border-box`.
`telar-ui` solo depende de esa pieza pura (`no_std + alloc`); `telar-gpu` y
`telar-ventana` continúan aisladas. La hoja se analiza al crear/personalizar el
estudio; las cajas se comparten entre pintura y ratón. La app admite
`--estilo ruta.css` al arrancar. No es un navegador ni implementa CSS completo;
ver `crates/telar-estilo/README.md` para el alcance exacto.

`telar-render` es otro componente puro: emite shaders de materiales GGX y consume
un bloque de cámara/material/luz de 128 bytes; el backend recibe esos bytes sin
conocer el modelo de material. La aplicación conserva la ruta histórica del visor
para sus referencias y usa el render original en el estudio procedural.
El contenedor `.tlar` guarda por ahora arranque y coordinador, no un proyecto completo.
Cada carpeta declara riesgo y consumo en SEMAFORO.md, verificado por `telar-guard`.

## La idea en una frase

**Los datos nacen, viven y mueren en la GPU.** La CPU edita el grafo, lo compila y le pasa a la GPU unos pocos bytes de parámetros por fotograma. Todo lo demás (emitir, simular, decidir cuánto trabajo hay y renderizar) lo hace la GPU sin volver a preguntar.

En casi todos los motores la CPU es la jefa y la GPU obedece: la CPU decide, manda, espera y vuelve a decidir. Cada ida y vuelta cuesta milisegundos. En TELAR, la CPU es la **arquitecta** (prepara el plan una vez) y la GPU es la **obra** (lo ejecuta y se organiza sola).

## Los crates, aislados

```
                 ┌──────────── no_std + alloc (sin sistema operativo) ────────────┐
                 │                                                                │
   telar-nodos ──┤──► nudo  (geometría, atributos, grafo, cocinado)               │
                 │                                                                │
   hilo ─────────┤──► IR tipado ──► hilo-spirv (SPIR-V emitido a mano)            │
                 └────────────────────────────────────────────────────────────────┘
                                   │ bytes SPIR-V + plan de despachos
                                   ▼
   telar-gpu (std; Vulkan a mano, unsafe solo en src/ffi/)
   telar-ventana (std; Win32 a mano, unsafe solo en src/ffi/)
                                   ▼
   telar-app (std): la interfaz de nodos, elegante
```

- Las flechas solo van hacia abajo. **El núcleo no conoce Vulkan ni Windows**: produce bytes (SPIR-V) y un plan (qué kernels lanzar, con qué buffers). Así funciona igual en Windows, en Linux o en BMO-X.
- `telar-gpu` es el único sitio que sabe que existe una GPU. Cambiar Vulkan por DX12, o por VERRANO en BMO-X, no toca nada de arriba.
- Cada crate tiene su README, sus tests y su guardián.

## Qué hace cada uno

| | CPU (la arquitecta) | GPU (la obra) |
|---|---|---|
| **Cuándo** | al cambiar el grafo | cada fotograma |
| **Qué** | editar nodos, compilar HILO → SPIR-V, fusionar nodos, reservar buffers y grabar los comandos una vez | emitir, advectar, resolver la presión, quemar el combustible, decidir cuántos ladrillos hay activos, renderizar |
| **Datos** | el grafo (pequeño) | la geometría y los volúmenes (grandes): nunca bajan a la CPU |
| **Por fotograma** | escribir ~256 bytes de parámetros y enviar el lote ya grabado | todo lo demás |

## El modelo de datos

- **Geometría** (como en Houdini): puntos, vértices, primitivas y **atributos** (`P`, `N`, `Cd`, `v`, `id`…). En la GPU, cada atributo es un buffer propio (estructura de arrays): los hilos leen en línea y la memoria va a toda velocidad.
- **Volúmenes:** primero, rejillas densas (128³–256³ caben de sobra en la RTX 3060). Después, **ladrillos dispersos**: solo existe la memoria donde hay humo o fuego, con ideas de OpenVDB/NanoVDB implementadas por nosotros. La GPU decide qué ladrillos se activan.
- **Parámetros:** un único buffer uniforme por fotograma. Mover un deslizador de la interfaz solo cambia esos bytes, no recompila nada.

## El grafo que cocina

1. El grafo vive en la CPU (en `nudo`), con marcas de **sucio**: cambiar un nodo ensucia solo lo que depende de él.
2. Al cambiar la **topología** (añadir o quitar nodos o cables), `hilo` recompila y **fusiona**: los nodos seguidos que trabajan punto a punto se unen en un solo kernel, sin leer ni escribir la memoria entre medias.
3. Al cambiar un **valor** dentro de la capacidad y los tipos del plan, no se recompila: se escribe el buffer de parámetros. Cambiar capacidad, tipos o estructura puede exigir nuevos recursos o un plan distinto.
4. El resultado es un **plan**: pasadas, recursos y dependencias que `telar-gpu` graba para reutilizar el cálculo. Debe contemplar los buffers alternos y los fotogramas en vuelo. La presentación depende de la imagen adquirida y puede grabarse aparte.

## Cómo «piensa» la GPU

- **Despacho indirecto** (`vkCmdDispatchIndirect`): un kernel cuenta, por ejemplo, cuántos ladrillos tienen fuego y escribe él mismo cuántos grupos lanzar en la pasada siguiente. La CPU no se entera ni necesita enterarse.
- **Alcance:** el host sigue preparando y enviando el grafo de pasadas. El despacho indirecto cambia sus dimensiones desde un buffer; no permite a un shader lanzar cualquier pipeline. La autonomía se mide por decisiones y datos que permanecen en GPU.
- **Colas de trabajo en la GPU:** los kernels añaden tareas con atómicos enteros (deterministas si se ordenan) y otra pasada las consume.
- **Más adelante:** la generación de comandos en la GPU (`VK_EXT_device_generated_commands`), o Work Graphs en DX12. Se valorará si el driver de la RTX 3060 lo soporta.
- **Sin lecturas de vuelta:** las estadísticas (fps, ladrillos activos) bajan en un anillo asíncrono, con varios fotogramas de retraso, sin frenar nunca a la GPU.

## Su procesamiento propio: por qué la GPU necesita algo distinto

Sí: la GPU necesita su propia forma de procesar, no la de la CPU traducida.

- **Miles de hilos a la vez:** los algoritmos se eligen por ser paralelos, no por ser los más cortos. Ejemplo: Jacobi o *red-black* Gauss-Seidel para la presión, y multigrid después; no un solver secuencial.
- **La memoria manda más que los cálculos:** fusionar nodos y usar estructuras de arrays vale más que optimizar una multiplicación.
- **Determinismo a propósito:** número de iteraciones fijo, reducciones en árbol con orden fijo y sin sumas de `float` atómicas. Si dos ejecuciones dan bytes distintos, es un fallo.
- **Precisión elegida:** `f32` en general, y `f16` solo donde se mida que no se pierde calidad.

## La CPU como testigo, no como jefa

Cada kernel tiene una **implementación de referencia en la CPU**, en `no_std`, lenta y clara: el mismo IR de HILO, interpretado. Los tests comparan GPU contra CPU (con tolerancia en `float`, o bit a bit donde se exige determinismo). Por encima está **Houdini Apprentice como referencia externa**, con el mismo nodo y la misma entrada.

Orden de autoridad, como en BMO-X:

1. el resultado en la GPU real, medido;
2. la referencia en la CPU (corrección);
3. Houdini Apprentice (comparación);
4. la teoría del artículo.

## Un fotograma, de punta a punta

```
CPU: escribe parámetros (256 B) ─► envía el lote grabado ─► sigue con la interfaz
GPU: emitir ─► advectar ─► fuerzas ─► presión ─► combustión ─► ¿ladrillos activos? (indirecto)
     ─► marcha de rayos con emisión de cuerpo negro ─► tonemap ─► presentar
     └─► (anillo asíncrono) estadísticas para la interfaz
```
