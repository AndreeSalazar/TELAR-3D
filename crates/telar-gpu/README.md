# telar-gpu

La GPU de TELAR 3D: **Vulkan a mano**, sin crates y sin enlazar el SDK (`vulkan-1.dll` se carga al arrancar, y cada función por su nombre). **Aislada por completo** (regla `fronteras`): no usa ningún crate de TELAR; de la ventana solo recibe dos números ([`Superficie`]).

```rust
let mut gpu = Gpu::nueva(&Opciones::default(), Superficie { instancia, ventana }, (1280, 720))?;
gpu.dibujar(eleccion::hex(0x35e3ff))?;   // limpia a cian y presenta, sincronizado con el monitor
gpu.dibujar_escena(fondo, &constantes)?;  // o dibuja la malla del visor (Fase 4)
let calculo = Calculo::nuevo(&Opciones::default())?; // sin ventana: para calcular y para los tests
```

## Cómo está hecho

La interfaz nativa se prepara con `preparar_interfaz` y se actualiza con
`actualizar_interfaz` cuando cambia. `ffi/interfaz.rs` conserva un pipeline 2D
sin profundidad y dos buffers visibles mapeados (120 000 vértices de capacidad
cada uno). Tras esperar la valla del juego, solo escribe si la revisión cambió.
El dibujo reserva un viewport central para la escena y restaura el área completa
al pintar el overlay. La escena y la UI comparten la imagen de presentación.

Cada vértice UI ocupa 32 bytes: posición NDC, coordenada de atlas en píxeles y
color RGBA. `fuente_interfaz` sube una máscara de cobertura a un SSBO residente;
el shader de fragmentos limita coordenadas, interpola cobertura y mezcla alfa.
El atlas se sube al preparar/cambiar la fuente y tiene su descriptor propio.
Sin atlas se conserva un píxel blanco para formas sólidas y pruebas de respaldo.

Son dos buffers y dos mapeos **adicionales**, separados de los 8 buffers y dos
mapeos de la superficie. La interfaz añade su pipeline gráfico; los contadores
de `ActividadProcedural` no incluyen recursos o triángulos UI. Al editar controles
sí se escriben triángulos UI desde CPU; no se suben vértices procedurales.

`Procedural::capturar_estudio` combina escena y `CapaInterfaz` en una captura
diagnóstica fuera de pantalla. La implementación vive en `ffi/captura_interfaz.rs`.

La ruta residente usa `preparar_procedural` y `dibujar_procedural` en la ventana,
o `Calculo::procedural` para pruebas sin ventana. La API de tres SPIR-V se traduce
al mismo ejecutor de planes; conserva buffers, pipelines, conjuntos y comandos.
Su ABI actual es el de la demostración
de rejilla; todavía no es un ejecutor de grafos arbitrarios.

`PlanProcedural` permite de 2 a 32 operaciones: preparar, generar y cero o más
pasadas por punto o de servicio. Cada `EtapaProcedural` declara nombre, código SPIR-V y dominio;
el dominio especifica el argumento indirecto y el contrato de buffers. El orden
expresa dependencias, con barreras conservadoras entre pasadas. Nombres, dominios
y cabeceras se validan antes de crear recursos. Esto no demuestra seguridad del
shader: deben respetarse sus accesos y verificarse el SPIR-V y su resultado.

Usar `Gpu::preparar_plan_procedural` o `Calculo::procedural_con_plan` para planes
nuevos. Cada operación tiene su nombre y tiempo GPU en `DiagnosticoProcedural`.
No se recorre el plan desde CPU por fotograma. El visor predeterminado encadena
preparar → generar → deformar → relieve, con 8 pipelines para dos juegos.
Las pasadas `Servicio` se despachan con una invocación aunque los argumentos
indirectos por punto sean cero. El backend no interpreta sus solicitudes: el
coordinador SPIR-V suministrado por `telar-kernel` las atiende en GPU.

| Módulo residente | Responsabilidad |
|---|---|
| `src/procedural.rs` | API segura, parámetros, actividad y diagnóstico |
| `src/plan.rs` | contratos de dominio y validación pura del plan ordenado |
| `src/ffi/residente.rs` | recursos persistentes y juegos por fotograma |
| `src/ffi/residente_comandos.rs` | grabación única, barreras y despachos indirectos |
| `src/ffi/residente_lectura.rs` | descargas, capturas y tiempos explícitos de diagnóstico |

La presentación espera su valla antes de escribir parámetros y envía el lote
de compute pregrabado seguido del dibujo. El dibujo usa argumentos GPU desde
el byte 12 del buffer de control. No se consulta un contador desde CPU.
El buffer de control tiene 128 bytes: el despacho de generación empieza en el
byte 48 y el de deformación en el byte 0. El productor mantiene la caché de
dimensiones y escribe cero grupos de generación cuando la geometría sigue
vigente. Se inicializa el control una sola vez con `vkCmdFillBuffer`, con una
barrera transfer → compute antes de leerlo. La inicialización está fuera del
bucle y del contador de grabaciones de compute reutilizables.

Las primeras 16 palabras conservan la geometría; las siguientes 16 quedan para
servicios residentes. `DiagnosticoProcedural::control` devuelve ahora 32 u32;
`PALABRAS_CONTROL_PROCEDURAL` publica esa dimensión sin depender del compilador.
La prueba de integración comprueba que coincide con el ABI del kernel aislado.

Los parámetros quedan mapeados durante toda la vida del ejecutor: `BufferMapeado`
se desmapea al destruirse, después de completar las vallas y antes de liberar
la memoria. Cada paso escribe sus 16 bytes sin llamar a mapear ni desmapear.
`ActividadProcedural::mapeos` cuenta uno por juego residente.
Al exceder capacidad, el productor emite trabajo cero y marca error; evita
producir geometría truncada. El modo vivo no descarga ese estado: la notificación
asíncrona a la interfaz queda pendiente.

`ActividadProcedural` cuenta recursos residentes y parámetros de superficie;
las constantes de cámara y los recursos temporales de diagnóstico quedan fuera
de esos contadores. `descargados` contabiliza la inspección de geometría; las
capturas se solicitan aparte. En la ruta viva no se ejecuta ninguna de ellas.

| Dónde | Qué |
|---|---|
| `src/eleccion.rs` | las decisiones puras, sin `unsafe` y probadas sin GPU: qué dispositivo (la dedicada primero), qué formato (sRGB), qué tamaño, cuántas imágenes y el color en luz lineal |
| `src/ffi/tipos.rs` | las estructuras y constantes, copiadas de `vulkan_core.h`/`vulkan_win32.h` y **comprobadas una a una contra los headers del SDK 1.4.350** |
| `src/ffi/carga.rs` · `funciones.rs` | `vulkan-1.dll` y cada función con su firma; las de la swapchain son opcionales (sin ventana no existen) |
| `src/ffi/contexto.rs` | instancia, capas de validación, superficie Win32, elección de la GPU, dispositivo y cola |
| `src/ffi/presentacion.rs` | la sincronización por fotograma y su ciclo: esperar, adquirir, grabar, enviar, presentar |
| `src/ffi/swapchain.rs` | la swapchain con una vista y un semáforo por imagen, y su imagen de profundidad |
| `src/ffi/dibujo.rs` | grabar un fotograma con renderizado dinámico (barreras, limpiar, dibujar); igual en la ventana y en las capturas |
| `src/ffi/pipeline_grafico.rs` · `malla_gpu.rs` · `imagen.rs` · `buffer.rs` | los recursos del visor, cada uno con su Drop |
| `src/ffi/visor.rs` | la escena de la ventana (el pipeline se hace para su formato) y las capturas fuera de pantalla |
| `src/escena.rs` | `Sombreadores` y `Constantes` (128 bytes), código puro |

Todo el `unsafe` vive en `src/ffi/`, cada bloque con su `// SAFETY:`; los tipos de puntero a función no lo necesitan (no ejecutan nada), y el guardián lo sabe.

## Sincronización

- Dos fotogramas en vuelo, cada uno con su valla y su semáforo de adquisición.
- Un semáforo de «listo para presentar» **por imagen**: la presentación lo usa hasta que esa imagen vuelve a adquirirse, así que reutilizarlo por fotograma provocaría avisos, y carreras de verdad.
- La swapchain se rehace sola al cambiar de tamaño o al volver de minimizada.

## Validación

Con `Opciones::validacion` (por defecto) y el SDK instalado, se activa `VK_LAYER_KHRONOS_validation` con un mensajero que cuenta cada aviso: [`avisos_de_validacion`] tiene que ser **0**.

Los avisos de tipo GENERAL (el cargador, y capas que otros programas meten en todas las apps Vulkan, como el overlay de GOG Galaxy) se enseñan como `[vulkan · sistema, no cuenta]`.

Las capas de validación ya cazaron dos fallos en el desarrollo: cargar funciones de la swapchain en un dispositivo sin ella, y un dispositivo que no se destruía cuando algo fallaba a medias.

## Tests

```bash
cargo test -p telar-gpu -- --nocapture
```

Las decisiones puras, el empaquetado de las constantes, y un contexto de Vulkan de verdad (sin ventana) con validación: **cero avisos**. En una máquina sin Vulkan, el test lo dice y no cuenta como fallo. Las capturas se prueban en `telar-app/tests/visor.rs`: allí están a la vez los shaders y la geometría.

## Calcular (Fase 2)

```rust
let gpu = Calculo::nuevo(&Opciones::default())?;          // sin ventana
let r = gpu.ejecutar(&spirv, 1 << 24, 256)?;               // kernel SPIR-V, elementos, hilos por grupo
r.datos;        // el buffer, ya en la CPU
r.nanosegundos; // solo el kernel, medido por la GPU
```

- **Los datos viven en memoria de la GPU** (`DEVICE_LOCAL`); al final se copian a memoria que la CPU lee (visible y coherente, con caché si la hay).
- **Los temporizadores miden solo el kernel**: dos marcas, antes y después del despacho, y `timestampPeriod` para pasarlo a nanosegundos.
- **Offsets calculados, no supuestos:** `timestampPeriod` y `maxComputeWorkGroupCount` se leen de `VkPhysicalDeviceProperties` por su desplazamiento (720 y 516), calculado campo a campo sobre el header con las reglas de alineación de C. El total, 824 bytes, coincide con el tamaño real.
- **`Recursos`** guarda todo lo que se crea (buffers, memoria, pipeline, descriptores, comandos, valla, consultas); su Drop lo suelta también si algo falla a medias.
- Si se piden más grupos de los que admite la GPU, es un error **antes** de tocarla.

`telar-gpu` no conoce `hilo-spirv` (están aislados): recibe el kernel como `&[u32]`, y los une `telar-app`.

## Dibujar (Fase 4)

```rust
gpu.preparar_visor(Sombreadores { vertices: &vs, fragmentos: &fs })?; // SPIR-V, entrada `main`
gpu.subir_malla(&malla.vertices, &malla.indices)?;  // [posición, normal] y triángulos
gpu.dibujar_escena(fondo, &Constantes { matriz, luz, color, cielo, suelo })?;
let pixeles = calculo.capturar(sombreadores, &vertices, &indices, (64, 64), fondo, &constantes)?; // RGBA8, sin ventana
```

- **Renderizado dinámico** (Vulkan 1.3, `dynamicRendering`): sin render pass ni framebuffers; el pipeline solo sabe los formatos.
- **Profundidad invertida e infinita** (D32, se limpia a 0, compara `GREATER`): la precisión está donde hace falta. La Y se da la vuelta en la proyección (`nudo::Camara`), no en el viewport.
- Caras **antihorarias** delante; se descartan las de detrás.
- **Constantes empujadas, 128 bytes:** la matriz, la luz y tres colores. Sin descriptores.
- La malla vive en **memoria de la GPU**: se sube con un buffer intermedio y una copia, con su barrera.
- La profundidad la comparten los dos fotogramas en vuelo: una barrera hace que cada uno espere a que el anterior termine de escribirla.
- El pipeline se rehace si cambia el formato de la ventana. El visor espera a la GPU antes de soltar nada.
