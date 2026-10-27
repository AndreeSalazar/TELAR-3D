# Rumbo del kernel GPU y elección de SPIR-V

Plan actualizado el **27 de octubre de 2026**. Describe decisiones y propuestas;
los hitos pendientes de esta página todavía no están implementados.

## Decisión: continuar con Vulkan y SPIR-V

Vulkan es la API para recursos, sincronización, cálculo y dibujo. SPIR-V es la
representación intermedia de los programas; el controlador la compila para su
GPU. El kernel coordinador es uno de esos programas, con un contrato de datos.
Son niveles distintos. [Khronos: SPIR-V](https://www.khronos.org/spirv/).

Mantendremos SPIR-V como formato principal de ejecución de TELAR porque permite
compartir contratos de cálculo y gráficos entre implementaciones compatibles.
Esta es una decisión de arquitectura y portabilidad, no un resultado de un
benchmark que demuestre superioridad universal.

| Ruta | Qué representa | Implicación para TELAR |
|---|---|---|
| Vulkan + SPIR-V | API y representación intermedia abierta para gráficos/cómputo | Ruta principal; ya tenemos emisor, backend y pruebas |
| Direct3D + DXIL | API y representación intermedia de shaders de Microsoft | Alternativa posible mediante otro backend; no decide por sí sola el rendimiento |
| CUDA + PTX | Entorno de cómputo y máquina virtual/ISA intermedia de NVIDIA | Útil como referencia técnica; vincular la ruta principal a NVIDIA reduciría nuestro alcance |
| Metal + sus bibliotecas | API y formato de programas del ecosistema Apple | Requeriría otra integración; Apple ofrece conversión desde DXIL a metallib |

Fuentes de las alternativas: [DXIL de Microsoft](https://github.com/microsoft/DirectXShaderCompiler/blob/main/docs/DXIL.rst),
[PTX de NVIDIA](https://docs.nvidia.com/cuda/parallel-thread-execution/index.html),
[Metal shader converter de Apple](https://developer.apple.com/metal/shader-converter/).
La tabla compara niveles de software; no supone que estas rutas sean equivalentes
en características o soporte de hardware.

**Windows tiene varias rutas**, entre ellas Direct3D y Vulkan. Vulkan también
está disponible en Linux; ver [plataformas de Khronos](https://docs.vulkan.org/guide/latest/platforms.html).
El núcleo portable no convierte automáticamente nuestra ventana Win32 en una
aplicación Linux: esa integración sigue pendiente.

Valve utiliza Vulkan en componentes de Proton: DXVK implementa Direct3D 8/9/10/11
sobre Vulkan y vkd3d-proton implementa Direct3D 12. Es evidencia de uso de Vulkan
para compatibilidad, no una prueba de que cualquier programa Vulkan sea más rápido.
Fuentes: [Proton](https://github.com/ValveSoftware/Proton),
[DXVK](https://github.com/doitsujin/dxvk),
[vkd3d-proton](https://github.com/HansKristian-Work/vkd3d-proton).

## Qué determina el rendimiento

Nuestra hipótesis de trabajo: pesan el algoritmo, accesos a memoria, cantidad de
pasadas, sincronización, ocupación y código final del controlador. Escribir SPIR-V
a mano ofrece control del programa emitido; cada optimización debe medirse.

Mediremos tiempo CPU de preparación/envío, tiempo GPU por pasada, transferencias,
memoria máxima y latencia del efecto completo. FPS con sincronización vertical
no demuestra velocidad del kernel. Tampoco fijaremos un reparto CPU/GPU 10/90:
buscamos reducir decisiones y transferencias host dentro del trabajo pesado.

## Estado comprobado antes de este plan

- `telar-kernel` emite el coordinador SPIR-V usado por el visor.
- ABI v1: 32 u32, 128 bytes de control por juego de recursos en vuelo.
- Buzón de una entrada con productores serializados; servicios de rehacer
  topología, suspender y reanudar; rechazos por ocupación y códigos desconocidos.
- Un trabajador de prueba solicita un servicio según datos residentes en GPU.
- El backend conserva recursos, graba el plan y lo envía; la geometría no vuelve
  al host en la ruta viva. El coordinador no asigna VRAM ni lanza programas arbitrarios.
- Cada juego tiene estado propio: todavía no hay coordinación global entre ambos.

Evidencia: `crates/telar-app/tests/kernel_gpu.rs`, `crates/telar-kernel/tests/spirv.rs`
y prueba de aplicación de 120 fotogramas. Máquina: RTX 3060 / Windows 11.
**IGUAL:** comportamiento de servicios frente a expectativas de las pruebas.
**ABIERTO:** rendimiento comparativo, otros fabricantes, Linux y VFX generales.
Validar SPIR-V para Vulkan 1.1/1.3 no equivale a probar físicamente esas plataformas;
el backend de la aplicación actual requiere Vulkan 1.3 y renderizado dinámico.

## Hitos de implementación

| Orden | Entrega | Hecho cuando |
|---|---|---|
| H1 | Contrato de arranque validado | Cabeceras incompatibles y regiones fuera de rango se rechazan; trabajo inválido no alcanza la geometría |
| H2 | Cola GPU acotada | Se publican, consumen y completan varias tareas; lleno/vacío y dependencias se resuelven sin leer contadores desde CPU |
| H3 | Regiones de memoria GPU | Las tareas reciben subregiones de una reserva existente; exceso, alineación y vida útil están comprobados |
| H4 | Nodos independientes conectados al plan | Dos instancias Onda tienen parámetros propios; conectar, desactivar y cambiar una afecta el resultado correcto |
| H5 | Un efecto VFX completo | Emisor → fuerzas → integración → color/tamaño → dibujo de partículas, con límites y resultados verificables |

Las herramientas actuales bastan para empezar H1. Una herramienta nueva se
incorporará por una necesidad concreta; se mantienen las reglas de dependencias.

### H1: siguiente paso concreto

Diseñar un bloque de arranque TELAR separado del estado mutable y de la cabecera
estándar del módulo SPIR-V. No modificar los cinco u32 iniciales que exige SPIR-V.
Tampoco anteponer datos al control actual: draw lee desde el byte 12 y generación
desde el byte 48. La migración del ABI deberá conservarlos o actualizar lectores
y comandos juntos, con pruebas de compatibilidad.

Propuesta preferida: un bloque de configuración de solo lectura, reservado al
preparar. Binding y tamaño definitivo se fijarán al comprobar la migración del
layout de descriptores; no caben todos estos datos en las siete palabras libres.

Contenido propuesto: firma TELAR, versión mayor/menor del contrato, longitud de
cabecera, capacidades habilitadas, dimensiones de regiones, límites de tareas,
presupuesto de trabajo y generación del plan. Las capacidades las consulta el
host al preparar Vulkan; la GPU recibe y verifica ese contrato.

El host debe rechazar recursos/descriptores inválidos antes de enviarlos. La GPU
comprobará longitudes reales de buffers antes de leer campos extendidos y validará
offsets y tamaños sin desbordamientos. Una cabecera que declara memoria no prueba
que esa memoria exista. No se sustituye validación de acceso por una firma mágica.

Pruebas de aceptación previstas:

1. Firma o versión incompatible: estado de error y argumentos indirectos cero.
2. Cabecera truncada o región inválida: rechazo antes de acceder al dato inválido.
3. Capacidades requeridas ausentes: rechazo explícito o variante compatible preparada.
4. Arranque válido: conserva superficie, servicios y caché actuales.
5. Entrada de servicio con arranque fallido: respeta el estado de error; no basta
   anular los trabajadores indirectos porque `Servicio` hoy se despacha directamente.
6. Cero avisos de validación, guardas de memoria sin cambios y ninguna descarga
   empleada como condición para decidir trabajo. Las inspecciones de test son explícitas.

### H2: cola antes de concurrencia general

Empezar con capacidad fija, un productor y un consumidor por pasada, con orden
determinista. Cada tarea tendrá operación, regiones autorizadas, dependencias,
estado y generación del plan. El número de operación referirá trabajo preparado;
no supondrá cargar y ejecutar un módulo SPIR-V arbitrario desde un shader.

Probar lleno, vacío, solicitudes duplicadas, dependencias incompletas, recuperación
y límites de vueltas. Luego evaluar múltiples productores con reserva atómica y
publicación correctamente ordenada. Un contador atómico entero no garantiza por
sí solo que el orden de llegada de tareas sea repetible: hará falta un contrato
de orden estable. La cola no usará un bucle infinito como mecanismo inicial.

### H3: asignación dentro de memoria reservada

El host reserva buffers; la GPU puede administrar subregiones dentro de ellos.
Primera versión: arena por juego, asignación acotada y reinicio con vida útil
comprobada. Probar alineación, agotamiento, desbordamiento y referencias antiguas.
Definir quién confirma que una región dejó de usarse antes de reciclarla, también
cuando hay dos juegos y presentación en vuelo. No equivaldrá a asignar VRAM del
sistema ni a tener protección de memoria entre procesos.

### H4 y H5: autonomía al servicio del efecto

Cada nodo declara entradas/salidas tipadas, parámetros, recursos y operación.
La UI edita ese mismo grafo. HILO y el compilador preparan las operaciones al
cambiar topología; la GPU decide trabajo activo dentro del plan. La independencia
lógica de los nodos permite fusionar operaciones compatibles y compartir recursos.

La primera simulación de partículas tendrá un paso temporal definido y un máximo
de subpasos. Humo/fuego volumétrico llegará tras demostrar un efecto completo y
un presupuesto de memoria medido. El objetivo final sigue siendo la esfera de fuego.

## Ideas nuevas para evaluar, no compromisos de rendimiento

| Idea | Beneficio buscado | Cómo decidir |
|---|---|---|
| Máscara de tareas listas en GPU | Omitir nodos sin cambios y ramas sin consumidores | Contar ejecuciones y medir el ahorro frente al coste de coordinar |
| Identificador de generación en tareas y regiones | Rechazar referencias antiguas tras editar el plan | Probar reutilización, reinicios y desbordamiento del contador |
| Separar configuración inmutable de estado por juego | Reducir escrituras accidentales y aclarar propiedad | Probar los dos juegos sin solicitudes perdidas ni estado mezclado |
| Trazas GPU acotadas | Registrar tarea y motivo de fallo sin bloquear cada paso | Medir sobrecoste; leer de forma asíncrona solo para diagnóstico |
| Variantes de grupo por operación | Ajustar paralelismo al tipo de trabajo | Medir variantes que respeten límites reales; elegir al preparar |
| Fusión de nodos por punto | Reducir pasadas y accesos de memoria | Comparar tiempos y resultados con la cadena separada |
| Compactación estable de partículas activas | Dibujar/simular solo elementos útiles | Comparar coste de compactación con trabajo ahorrado y comprobar orden |
| Caché de programas y pipelines | Reducir preparación repetida | Versionar por código, contrato, dispositivo y datos requeridos por el controlador |
| Contrato numérico por operación | Poder optimizar sin ocultar errores | Exactitud entera; tolerancias flotantes explícitas y casos NaN/infinito |

Una variante rápida de matemáticas solo se aceptará con error máximo definido.
La igualdad bit a bit entre fabricantes no se presumirá para punto flotante.
Mediremos primero; SPIR-V válido y ausencia de avisos Vulkan no demuestran por sí
solos seguridad de todos los accesos, corrección matemática ni mayor velocidad.

## Registro de decisiones

- **Adoptado:** SPIR-V/Vulkan como ruta principal y `telar-kernel` como coordinador.
- **Próximo:** H1, contrato de arranque y rechazo seguro, antes de ampliar servicios.
- **Después:** H2 y H3 con límites claros; H4 conecta el editor con operaciones reales.
- **Opcional:** estudiar otros backends solo ante una necesidad de plataforma o
  una medición concreta que justifique el coste. No se cambia de formato por fama.
- **No demostrado:** que una sola función gigante sea mejor, que toda tarea deba
  migrar a GPU o que menos uso de CPU implique automáticamente menos latencia.
