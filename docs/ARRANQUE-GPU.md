# Arranque TLAR v1 · contrato implementado

Implementado el 27 de octubre de 2026. El visor usa un encabezado residente de
**32 u32 / 128 bytes**, aparte del control mutable y de la cabecera estándar del
módulo SPIR-V. Los shaders SPIR-V siguen siendo válidos para Vulkan.

## Datos del dispositivo y selección

`telar-gpu::PerfilGpu` consulta fabricante, dispositivo, tipo, versiones crudas
de API/controlador, límites de grupos/hilos, memoria compartida, rangos SSBO,
descriptores y heaps locales. `telar-app` adapta esos datos al tipo puro
`telar_kernel::arranque::Dispositivo`, sin crear dependencia entre ambos crates.

El selector actual exige una cola de gráficos+cómputo y presentación cuando hay
ventana, y prioriza dedicada sobre integrada, virtual y CPU/otro. Es una política
de selección por tipo (`eleccion::puntuar`), no una prueba de cuál es más rápida.
El constructor del contrato rechaza límites incompatibles con el plan solicitado.

La suma de heaps DEVICE_LOCAL describe memoria anunciada, **no memoria libre**.
En dispositivos UMA puede ser compartida con CPU. No se consulta el número de SM,
reloj, registros físicos ni tamaños de caché: no son parte de este perfil básico.
La versión del controlador se conserva cruda porque su codificación puede variar.
Referencias: [límites Vulkan](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceLimits.html),
[heaps y memoria UMA](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceMemoryProperties.html).

## Layout del bloque de solo lectura, enlace 4

Las posiciones son palabras u32, no un struct Rust enviado con padding. El
contrato no incluye punteros host ni tipos dependientes de Windows. Es portable
entre implementaciones compatibles; no afirma compatibilidad con cualquier GPU.

| Palabras | Contenido |
|---|---|
| 0–2 | Firma TLAR, versión 1, tamaño declarado de 32 palabras |
| 3–4 | Máscaras TELAR de capacidades presentes/requeridas: cálculo e indirectos |
| 5–8 | Fabricante, dispositivo, controlador, versión Vulkan |
| 9–13 | Grupos X, invocaciones/grupo, hilos X, memoria compartida, rango SSBO |
| 14–15 | Memoria local u64, mitad baja y alta |
| 16–20 | Capacidad por lado y tamaños declarados de parámetros, vértices, índices y control |
| 21–24 | Grupo de 64 hilos, etapas declaradas, presupuesto de grupos por despacho, generación de plan |
| 25–26 | Límite conservador SSBO por etapa/conjunto; tipo de dispositivo |
| 27–31 | Reservado, debe ser cero |

Las máscaras son capacidades del contrato TELAR, no números `OpCapability` de
SPIR-V. Los kernels aún usan grupos de 64; los límites habilitan validación, no
selección automática de tamaño. Etapas y generación son metadatos del plan;
no autentican el código ni prueban lo que hace un shader externo.

El host es la autoridad de la consulta física. El shader comprueba coherencia
y disponibilidad de regiones; no puede demostrar que un fabricante o límite
escrito por un host malicioso corresponda a hardware real.

## Guardián ejecutado en GPU

`compilar_con_arranque` añade el prólogo de `verificacion.rs` al coordinador:

1. Consulta `OpArrayLength` antes de leer campos del encabezado. Una cabecera
   de una palabra o truncada no se lee como si tuviera 32.
2. Comprueba firma, versión y tamaño; luego capacidad 2..1024, coincidente con
   la capacidad de parámetros proporcionada por el ejecutor.
3. Calcula tamaños sin desbordamiento tras validar capacidad y contrasta los
   tamaños declarados con longitudes reales de vértices, índices y control.
4. Verifica máscaras básicas, perfil mínimo, grupo de 64 y rangos SSBO. El límite
   de descriptores es el menor entre máximos por etapa y por conjunto.
5. Comprueba etapas 2..32, generación positiva, presupuesto de grupos y reservados.
6. Si admite el contrato, habilita trabajo y servicios. Si lo rechaza, registra
   un código y publica cero trabajo de generación, puntos, dibujo y servicios.

El backend garantiza control de 32 palabras y parámetros de cuatro antes de
enviar. Los bloques crudos de arranque aceptan 1..256 palabras para otros emisores
y pruebas de rechazo; la cabecera vacía se rechaza en host antes de crear un
descriptor inválido. No se admite un descriptor de control arbitrariamente corto.

La API cruda no es un sandbox de shaders. Los programas deben cumplir el contrato,
validarse con `spirv-val` y probarse. Las cabeceras no corrigen accesos ilegales
de un trabajador externo que ignore las reglas del plan.

## Estado mutable y códigos de error

El control conserva su prefijo de 32 palabras y mantiene los offsets anteriores:
deformación en byte 0, dibujo en byte 12 y generación en byte 48.
El backend ahora reserva 40 palabras; las ocho adicionales pertenecen a la caché
optativa de deformación. La cabecera v1 sigue declarando el mínimo de 32.
Ver [la extensión y sus límites](RENDER-AREA.md).

| Control | Significado |
|---|---|
| 25 | Error de arranque: 0 admitido, 1 truncado/parámetros, 2 firma/versión/reservados, 3 regiones/capacidad, 4 capacidades, 5 presupuesto/metadatos |
| 26 | Listo: uno si el contrato fue admitido |
| 27–29 | Argumentos indirectos de servicio, byte 108: [1,1,1] admitido; [0,1,1] rechazado |
| 30–31 | Generación y fabricante copiados desde el encabezado para diagnóstico |

Con arranque, `Servicio` usa estos argumentos indirectos. Esto impide que se
ejecute tras un rechazo aunque antes se despachara directamente. La suspensión
de geometría mantiene servicios habilitados para poder pedir reanudación.
Una solicitud pendiente no se consume si el contrato de arranque se rechaza.

`compilar()` y los planes sin `.con_arranque()` conservan la ruta anterior de
cuatro buffers para pruebas y compatibilidad. La ruta con guardián requiere
cinco SSBO por etapa/conjunto y mantiene los cuatro primeros enlaces.

## Coste y estado de las optimizaciones

Se sube un bloque una vez por juego. El visor pasa de 8 a **10 buffers residentes**,
conservando 8 pipelines compute, 2 grabaciones y 2 mapeos de parámetros. Hay 256
bytes adicionales de contenido de cabecera entre ambos juegos; las asignaciones
físicas pueden ocupar más por alineación del dispositivo.

El prólogo se ejecuta en una invocación del coordinador en cada paso. No hay
consulta de perfil, construcción de cabecera ni subida de esta por fotograma.
Los errores se leen solo mediante diagnóstico explícito; su presentación asíncrona
en UI sigue pendiente. La ruta viva no descarga estado ni geometría.

**No hay una aceleración demostrada en esta entrega.** Todavía no se administran
subregiones de VRAM ni se seleccionan variantes de matemáticas o algoritmos.
La siguiente etapa es la cola acotada del plan; el perfil permitirá medir variantes
compatibles después, sin identificar fabricante con rendimiento.

## Evidencia y verificaciones

- `telar-kernel/tests/arranque.rs`: límites host y contrato sin lista de fabricantes.
- `telar-kernel/tests/spirv.rs`: módulos aprobados para Vulkan 1.1 y 1.3.
- `telar-app/tests/arranque_gpu.rs`: cabeceras truncadas/malformadas, errores de
  capacidad, rangos y capacidades; canario de servicio sin escribir tras rechazo;
  superficie válida y suspensión/reanudación en GPU.
- Prueba de aplicación: 120 fotogramas, dos cambios de tamaño, sin avisos Vulkan;
  cero geometría subida y cero descargas en vivo. RTX 3060 / Windows 11.

Los offsets de propiedades FFI se contrastaron con `offsetof` y `_Static_assert`
compilados contra `vulkan_core.h` del SDK 1.4.350 en MSVC x64: `maxStorageBufferRange`
324, `maxPerStageDescriptorStorageBuffers` 372, `maxDescriptorSetStorageBuffers` 404,
`maxComputeSharedMemorySize` 512, `maxComputeWorkGroupCount` 516,
`maxComputeWorkGroupInvocations` 528 y `maxComputeWorkGroupSize` 532.

**IGUAL:** servicios y geometría conservados dentro de sus pruebas de referencia.
**ABIERTO:** aceleración, pruebas físicas en AMD/Intel, otros sistemas operativos
y guardianes de acceso de propósito general. El encabezado es el contrato de TELAR,
no un arranque bare-metal ni un reemplazo del controlador.
