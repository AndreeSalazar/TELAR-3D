# TELAR kernel · coordinador GPU

El rumbo y sus criterios de aceptación están en
[RUMBO-KERNEL.md](../../docs/RUMBO-KERNEL.md): contrato de arranque, cola,
regiones de memoria, nodos y primer efecto VFX. El contrato de arranque ya está
implementado: [ARRANQUE-GPU.md](../../docs/ARRANQUE-GPU.md). Cola y arena siguen pendientes.

Pieza propia en `no_std + alloc`, sin objetos Vulkan ni dependencias de terceros.
Rust prepara módulos SPIR-V; el coordinador y sus productores ejecutan en GPU.
El visor usa `telar_kernel::compilar_con_arranque()` como primera pasada residente.
`compilar()` conserva la variante anterior sin encabezado para compatibilidad.

## Organización

| Archivo | Responsabilidad |
|---|---|
| `src/abi.rs` | contrato versionado, buzón, estados y servicios |
| `src/coordinador.rs` | límites, caché, despacho y dibujo indirectos |
| `src/servicios.rs` | productores GPU y atención de solicitudes |
| `src/buzon.rs` | interfaz para que otros kernels emitan solicitudes y reciban aceptación GPU |
| `src/maquina.rs` | emisión de acceso a los buffers SPIR-V |
| `src/arranque.rs` | constructor del encabezado TLAR y límites consultados |
| `src/verificacion.rs` | guardián GPU de cabecera, regiones y admisión de trabajo |
| `tests/spirv.rs` | validación oficial del código emitido |

## Primeros servicios internos

El buzón tiene una entrada. Un productor de una invocación publica un número
en `control[16]`; el coordinador lo consume en su próxima vuelta:

1. `RehacerTopologia`: invalida la caché para que la GPU regenere al activarse.
2. `Suspender`: anula cantidades de despacho/dibujo conservando memoria y caché.
3. `Reanudar`: habilita trabajo sujeto a límites de capacidad.

El coordinador no descarga solicitudes al host. Un código desconocido se consume
y se registra como error. Si otro productor encuentra el buzón ocupado, conserva
la primera solicitud e incrementa el contador de rechazos. No hay FIFO ni soporte
para productores concurrentes en esta versión: las pasadas tienen barreras Vulkan.

Cada juego de recursos en vuelo tiene su propio control, buzón y contador de
vueltas. La solicitud afecta a ese juego y se atiende al volver a ejecutarlo;
no es todavía un estado global compartido entre los dos juegos del visor.

`solicitar_en(codigo, vuelta)` es un productor de demostración: compara el contador
residente de vueltas y publica desde GPU. Se incorpora con dominio `Servicio`,
que siempre ejecuta una invocación, incluso con trabajo geométrico suspendido.
Así puede solicitarse reanudación sin depender de un kernel por punto desactivado.
En vivo, la aplicación no añade estas solicitudes de demostración automáticamente.

`Buzon::solicitar` permite insertar la solicitud en otros kernels y devuelve un
Id booleano GPU de aceptación. La prueba de integración también construye un
trabajador externo que decide solicitar regeneración leyendo la cantidad de
puntos residente, sin un calendario de solicitudes calculado en CPU.

Estos servicios se parecen a una interfaz de llamadas internas, pero no son
syscalls de Windows/Linux. No acceden a archivos, red ni recursos externos.

## ABI y memoria

Enlace 0: cuatro u32 de parámetros (lado, fase, amplitud, capacidad). Enlaces 1/2:
geometría, usada por los trabajadores. Enlace 3: control de **32 u32 / 128 bytes**,
reservado e inicializado una sola vez por el backend.

- `[0..16)`: ABI de geometría anterior, compatible con sus trabajadores.
- `[16..25)`: solicitud, suspensión, resultado, atendidas, versión, último código,
  vuelta, rechazos por ocupación y códigos desconocidos; ver constantes en `abi`.
- `[25..32)`: estado de arranque, despacho indirecto de servicios y metadatos.

La ruta con arranque añade una cabecera inmutable de 32 u32 en enlace 4, construida
con el perfil físico por el host y verificada por GPU antes del trabajo. La ruta
sin cabecera conserva cuatro enlaces. El contrato exacto está en ARRANQUE-GPU.md.

Los contadores u32 son modulares. El número de versión describe el contrato;
no es una negociación dinámica entre versiones. Los productores deben cumplirlo.
Cada pasada posee sus escrituras; el ejecutor publica compute → compute/indirecto.
El coordinador nunca reserva VRAM ni selecciona pipelines arbitrarios.

## Alcance real y portabilidad

**Implementado:** coordinación de esta superficie, caché, servicios residentes y
argumentos indirectos decididos en GPU; un lote pregrabado por juego de recursos.
**Pendiente:** cola general, dependencias de grafos, arena GPU de memoria,
selección general de operaciones y simulaciones VFX. No es un kernel persistente
infinito ni un administrador de toda la tarjeta. Los servicios no detienen la UI.

El backend sigue creando recursos, grabando comandos y enviando lotes desde CPU.
La GPU decide las cantidades del trabajo ya preparado, como permite
[`vkCmdDispatchIndirect`](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchIndirect.html).

Los módulos usan SPIR-V 1.3 y se validan para Vulkan 1.1 y 1.3, sin extensiones de
fabricante. La aplicación actual usa Vulkan 1.3 y renderizado dinámico. Eso es
una base portable entre implementaciones compatibles, **no una garantía para
cualquier GPU**. De momento se verifica físicamente en NVIDIA RTX 3060/Windows;
AMD, Intel y otras configuraciones requieren pruebas propias.

`telar-app/tests/kernel_gpu.rs` comprueba solicitudes, regeneración, suspensión,
reanudación, ocupación, errores y límites en GPU real. Sus descargas son diagnóstico
explícito; ninguna lectura decide el siguiente paso. El visor vivo no descarga.
