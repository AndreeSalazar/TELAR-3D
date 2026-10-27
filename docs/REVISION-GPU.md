# Revisión del rumbo: un núcleo procedural en GPU

Revisión del 26 de octubre de 2026 sobre `b8f4109`. El diagnóstico inicial queda
como registro histórico. Después se implementó la primera demo residente de
la Fase 4.5, descrita a continuación; el cocinado general del grafo sigue abierto.
Las cifras históricas de las fases 2 y 4 no se usan como medidas de esta ruta.

## Avance implementado tras la revisión

- Visor predeterminado: rejilla generada y deformada con tres kernels; dispatch
  y draw indirectos cuyos argumentos escribe la GPU. Sin cocinado ni malla CPU.
- Dos juegos residentes: 8 buffers, 6 pipelines de compute y 2 comandos de compute
  pregrabados. Parámetros escritos después de la valla de su juego; envío junto
  con la presentación. Cero descargas en vivo.
- Separación Rust: `hilo-spirv/src/rejilla.rs` emite programas; la API segura vive
  en `telar-gpu/src/procedural.rs`; recursos, comandos y diagnóstico se separan en
  `ffi/residente*.rs`; `telar-app/src/escena.rs` los conecta.
- Pruebas: 120 pasos con tamaños y parámetros cambiantes, referencias numéricas
  CPU (tolerancia 2·10⁻⁵), repetición bit a bit y píxeles que cambian; caso máximo
  de 1 048 576 puntos y 2 093 058 triángulos. Sin avisos de validación del proyecto,
  incluidas pruebas de cálculo con `VK_LAYER_VALIDATE_SYNC=1`.
- Etiquetas: **IGUAL** frente a la referencia CPU dentro de tolerancia para esta
  superficie; **ABIERTO** contra Houdini y para el grafo general. Máquina: Ryzen
  5 5600X / NVIDIA GeForce RTX 3060 / Windows 11.

El exceso de capacidad produce cero trabajo y marca error en GPU. La ruta viva
todavía no informa ese estado asíncronamente. No hay compactación en esta demo;
el orden de los índices se obtiene directamente de las coordenadas de cada celda.
La siguiente ampliación es generalizar el ejecutor para los nodos y HILO.

## Segunda entrega: la GPU decide reutilizar

El productor ya conserva la última topología válida. Cuando el lado no cambia,
escribe cero grupos en el argumento de generación; el despacho de deformación
sigue independiente. Vacío o exceso no destruyen la caché. Un contador entero
GPU permite verificar que no se volvió a generar. El control creció a 64 bytes
y se inicializa a cero con una operación GPU al crear cada juego.

Los parámetros están mapeados de forma persistente. La ruta viva mantiene
2 mapeos en total, en vez de mapear y desmapear en cada actualización. Conserva
8 buffers residentes, 6 pipelines, 2 grabaciones reutilizables y 16 bytes de
parámetros de superficie por paso. La cámara/iluminación sigue separada.

`tests/cache_gpu.rs` comprueba 120 pasos con cambios y reutilización frente a un
estado esperado CPU; la prueba máxima comprueba que el segundo paso conserva
1 generación para 1 048 576 puntos. Etiqueta: **IGUAL** para la geometría con la
misma entrada; **ABIERTO** para comparaciones de rendimiento publicables y Houdini.

En una ejecución diagnóstica en la RTX 3060, el intervalo GPU de generación fue
161 792 ns al generar y 2 272 ns al omitirla; el segundo intervalo contiene el
coste de comandos y temporización, no generación de vértices. Son muestras
aisladas, no un benchmark ni una medida del tiempo completo de fotograma.

Fuentes: [despacho indirecto](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchIndirect.html)
y [relleno de buffers](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdFillBuffer.html)
de Khronos, consultadas durante esta entrega.

## Lo que demostraba el código de b8f4109

Las entregas anteriores conservan sus cifras históricas. El estado nuevo se
describe en la sección de planes extensibles al final de este documento.

| Pieza | Evidencia | Alcance real |
|---|---|---|
| Geometría | `telar-app/src/visor.rs`: `esfera.cocinar`, `Malla::de` y `subir_malla` | Se genera y triangula en CPU; después se dibuja en GPU. |
| Grafo | `nudo/src/grafo.rs`: resultados `Option<Geometria>` y `Nodo::cocinar` | Caché y evaluación de geometría en CPU. Es una referencia útil, todavía no un plan de ejecución GPU. |
| Cálculo | `telar-gpu/src/ffi/calculo.rs`: `Recursos::nuevo`, despacho, copia y espera de valla | Kernel real en GPU, pero recursos de una ejecución y descarga completa al terminar. |
| Dibujo | `telar-gpu/src/ffi/dibujo.rs`: `dibujar_indices` | Cantidad de índices proporcionada por el host. |
| Comandos | `telar-gpu/src/ffi/presentacion.rs`: reiniciar, empezar y grabar | Se vuelve a grabar el dibujo cada fotograma. |
| Despacho | `telar-gpu/src/ffi/funciones_calculo.rs`: `vkCmdDispatch` | La ruta actual no carga `vkCmdDispatchIndirect`. |

El millón de puntos a 75 fps acredita el visor de una malla estática. No mide
generación procedural, deformación o simulación en GPU por fotograma. El hash
acredita un primer kernel, no un ejecutor de grafos residente.

## Qué significa «la GPU manda»

El **kernel** es el programa SPIR-V ejecutado por los hilos de la GPU. El núcleo
del producto incluye además el modelo de datos, el compilador y el ejecutor.
`no_std` permite aislar código del sistema operativo; no convierte Rust en un
programa que se ejecute en la GPU.

Con la ruta Vulkan base elegida, la CPU crea recursos y pipelines, graba las
dependencias y envía trabajo. Atiende también la ventana y la presentación.
La autonomía buscada consiste en que los resultados intermedios y la cantidad
de trabajo siguiente se resuelvan en GPU, sin consultarlos desde CPU.

Un despacho indirecto lee sus dimensiones de un buffer al ejecutarse. Un kernel
productor puede escribir ese buffer; no lanza por ello kernels arbitrarios ni
cambia el pipeline de la siguiente pasada. El grafo de pasadas sigue pregrabado.
Esta base basta para empezar; las extensiones opcionales no son un requisito.

## Corrección del orden del plan

La fase 7 posponía demasiado la prueba de autonomía. Se añade la **fase 4.5**
antes de ampliar HILO: primero demostrar la cadena GPU → GPU → dibujo; después
generalizarla con el compilador. Los ladrillos dispersos siguen en la fase 7.

### 4.5 A · Recursos residentes y ejecución repetible

- Separar creación de recursos, ejecución y lectura de diagnóstico. Conservar
  `Calculo::ejecutar` como herramienta de comparación, no usarlo como bucle vivo.
- Crear una API de recursos persistentes en `telar-gpu`, sin depender de NUDO ni
  HILO: tamaños, usos, identificadores y bytes SPIR-V llegan desde `telar-app`.
- Parámetros en un buffer por fotograma en vuelo. No sobrescribir parámetros
  todavía leídos por la GPU. Las constantes empujadas actuales están grabadas
  en comandos: no cambian solas al escribir un buffer del host.
- Separar comandos de cálculo reutilizables de los comandos de presentación,
  dependientes de la imagen adquirida. Resolver también la alternancia de los
  buffers de lectura/escritura; por ejemplo, con dos variantes pregrabadas.

### 4.5 B · La GPU determina el siguiente trabajo

- Un productor escribe datos y argumentos `VkDispatchIndirectCommand`; el
  consumidor usa `vkCmdDispatchIndirect`, sin leer el contador en CPU.
- Buffers de argumentos con usos de almacenamiento e indirecto. Comprobar
  alineación, tamaño y límites de grupos del dispositivo. Cantidad activa cero
  produce trabajo cero; nunca acceder fuera de capacidad.
- Barrera desde escritura de compute hacia lectura de comandos indirectos
  (`COMPUTE_SHADER/SHADER_WRITE` → `DRAW_INDIRECT/INDIRECT_COMMAND_READ`). Añadir
  también dependencia de compute a compute para los datos del consumidor.
- Reservar capacidad explícita. Al excederla, marcar desbordamiento en GPU y
  limitar las escrituras. La interfaz puede informar y ampliar capacidad fuera
  del paso normal; nunca continuar silenciosamente con resultados incompletos.
- La compactación tendrá orden estable mediante prefijos y posiciones de
  entrada; un contador atómico entero por sí solo no ordena la lista resultante.

### 4.5 C · Demostración visible de geometría

Primera entrega: **rejilla generada y deformada en compute**, con normales e
índices escritos en GPU, y dibujo desde esos mismos buffers. Una rejilla permite
aislar memoria, sincronización y recuentos antes de portar la triangulación
general y el tratamiento de aristas de `Malla::de`.

Un kernel prepara el despacho según las dimensiones del parámetro; otro genera
la rejilla; una pasada de deformación consume los datos residentes. La GPU
escribe también un `VkDrawIndexedIndirectCommand`. El rasterizador usa
`vkCmdDrawIndexedIndirect` y barreras de compute hacia lectura de vértices,
índices y argumentos. Ningún vértice de esa ruta nace en CPU.

Después se porta la esfera y los nodos existentes, conservando las referencias
CPU y sus libretos. Esto prueba el soporte de ejecución antes de construir el
lenguaje completo.

## Hecho cuando: prueba de arquitectura

1. Un mismo ejecutor corre al menos 120 pasos variando parámetros, sin crear
   buffers, pipelines o descriptores por paso estable.
2. Los casos vacío, un elemento, grupo parcial y capacidad máxima dan recuentos
   y salidas correctos; exceder capacidad da un estado de error controlado.
3. El kernel consumidor cambia su trabajo a partir del argumento escrito por
   el productor. La CPU no calcula ese argumento ni lo descarga entre pasadas.
4. Cambiar amplitud o dimensiones dentro de capacidad cambia la imagen sin
   recompilar shaders, recocinar geometría CPU ni subir vértices o índices.
5. La ruta viva tiene cero bytes de descarga de geometría y cero esperas para
   inspeccionar resultados intermedios. Se permiten sincronización de recursos
   en vuelo y adquisición/presentación; se contabilizan por separado.
6. Contadores de recursos creados, grabaciones de compute, envíos, bytes de
   parámetros, subidas y descargas demuestran esos límites. Contar únicamente
   llamadas Vulkan no prueba que las decisiones residan en GPU.
7. Referencia CPU sobre casos pequeños y lectura final explícita en tests;
   SPIR-V validado y cero avisos Vulkan. Determinismo bit a bit entre ejecuciones
   en el mismo dispositivo/configuración; tolerancias declaradas contra CPU
   para operaciones flotantes, sin prometer identidad entre dispositivos.
8. Tiempos GPU de generación, deformación y dibujo separados, y tiempo CPU del
   fotograma y del arranque. Los fps con sincronización vertical no sustituyen
   esas medidas. Máquina, configuración y etiqueta acompañan los resultados.

## Consecuencias para las fases siguientes

- **HILO v0:** separar análisis y compilación del plan de su ejecución. Un cambio
  de valor actualiza parámetros; cambiar conexiones, tipos o capacidad puede
  exigir otro plan o recursos. Fusión solo donde preserve dependencias y
  semántica; primero corrección y residencia.
- **Humo y fuego:** comenzar con rejillas densas y pasadas acotadas, no con un
  megakernel persistente. No hay barrera global ordinaria entre todos los grupos
  de un despacho. Usar despachos separados para dependencias globales.
- **Presión:** iteraciones fijas inicialmente; una terminación adaptativa futura
  requiere reducción y control GPU explícitos dentro de un máximo acotado.
- **Rendimiento:** 128³ a 60 fps sigue siendo una meta, no una conclusión del
  visor. Medir solver, memoria temporal, render y transferencias por separado.
- **Alternativa procedural:** preservar geometría, atributos, grafo y nodos como
  base del producto. La esfera de fuego es la demostración principal; completar
  Pyro no equivale a cubrir todos los contextos de Houdini.

## Fuentes técnicas consultadas

- [Khronos: vkCmdDispatchIndirect](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchIndirect.html).
- [Khronos: vkCmdDrawIndexedIndirect](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDrawIndexedIndirect.html).
- [Khronos: ejemplos de sincronización](https://docs.vulkan.org/guide/latest/synchronization_examples.html),
  incluidas dependencias de compute a compute, dibujo indirecto y entrada de vértices.

## Tercera entrega: planes extensibles

El ejecutor ya no contiene un array fijo de tres kernels ni decide el argumento
indirecto por la posición de la pasada. `PlanProcedural` valida entre 2 y 32 etapas,
con nombre, SPIR-V y dominio explícito. Los dominios definen preparar, generar y
operar sobre puntos; sus dependencias se graban en orden, con barreras entre
etapas. Los pipelines y consultas se dimensionan al preparar el plan.

El visor utiliza preparar → generar → deformar → relieve. La cuarta operación
escala la altura 1,3 y corrige normales en GPU. Mantiene 8 buffers, 8 pipelines,
2 grabaciones y 2 mapeos persistentes para dos juegos. Sigue enviando 16 bytes
de parámetros de superficie, además de las constantes de cámara/iluminación.

La API de tres kernels se conserva como adaptador al plan. El diagnóstico ahora
devuelve vectores de nombres y tiempos por operación. Las pruebas cubren planes
de 2, 3, 4 y 5 etapas, factores positivos, negativos y cero, alteraciones del
orden, vacío y reutilización de la caché. Etiqueta **IGUAL** contra las referencias
CPU con tolerancia 2·10⁻⁵; la comparación con Houdini sigue **ABIERTA**.

Límites: secuencia ordenada y cuatro buffers del ABI de rejilla, no un grafo
general de recursos. Las operaciones por punto conservan x/z. La validación de
metadatos no demuestra el comportamiento del shader: los programas se validan
con spirv-val y con pruebas Vulkan. El IR tipado de HILO es el siguiente paso.
