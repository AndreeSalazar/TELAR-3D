# 🟡 Semáforo · crates/telar-ui/src

Estado: AMARILLO
Motivo: Layout, validación de estilos y geometría de interfaz; riesgo de entradas y desbordes visuales.
CPU: Analiza CSS al cargar y compone triángulos cuando cambia estado/tamaño; calcula cajas y texto.
GPU: No ejecuta GPU directamente; los triángulos se dibujan por telar-gpu.
VRAM: Sin reserva directa; el backend conserva dos buffers UI y un atlas residente.
RAM: Reglas, métricas y Vec de vértices UI; geometría limitada por capacidad del backend.
E/S: No hay E/S en estos crates no_std; la app carga el CSS y la fuente.
Frecuencia: Carga/cambio de interfaz; la imagen se dibuja en cada fotograma.
Limites: No es CSS web completo; fuentes latinas, UI hasta 120000 vértices y atlas limitado.
Evidencia: crates/telar-ui/tests, crates/telar-estilo/tests y docs/SEMAFOROS.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Cada nodo conserva sus parámetros y el inspector expone solo los controles propios.
No calcula geometría 3D; envía valores a la app. Evidencia: tests/parametros.rs
y docs/NODOS-PARAMETROS.md.

Calidad seleccionable 1x/2x/4x: capacidades guardadas por formato; cambiar
el modo efectivo espera trabajo GPU y reemplaza adjuntos/pipelines gráficos.
Compute se conserva. Incremento lógico 2x: 12 B/píxel; 4x: 28 B/píxel
(RGBA8/D32). El pico durante el reemplazo no se midió.
Evidencia adicional: docs/CALIDAD-RENDER.md y tests/calidad_visor.rs de telar-app.

Escena v2: registro de 128 B; la UI serializa solo valores/posiciones y solicita
E/S a la app. Cámara/fase y CRC se validan antes de aplicar. Guardar/abrir es
síncrono y solo explícito; sin E/S de disco en cuadros estables. Reemplazo por
temporal en el mismo directorio; guardado no sube geometría ni añade buffers GPU.
La reapertura inicial reconstruye recursos; en vivo requiere igual capacidad.
Ver docs/ESCENAS.md y pruebas proyecto.rs, proyecto_render.rs y estado.rs.

Números y red: hasta diez identidades y 24 caracteres por edición. Validación,
serialización de datos y recorte de triángulos UI consumen CPU/RAM al editar.
No evalúa puntos 3D ni reserva recursos GPU; la app recibe peticiones de recompilar
y de E/S. La extensión de escena ocupa hasta 72 palabras; sin código ejecutable.
Ver docs/GRAFO-EDITABLE.md y tests/numeros.rs, red.rs y estado.rs.

Editor de objetos: dos cajas y luz con controles nativos, escena v5 de 184 B.
CPU prepara shaders al confirmar cambios; la vista traza 1 spp en GPU, limitada
a 1024×768, sin descarga viva ni reconstrucción UI en cuadros estables. Ediciones
esperan/reemplazan pipeline; pico transitorio no medido. La UI mantiene estado,
texto de hasta 24 caracteres y triángulos/atlas; usa buffers de overlay existentes.
E/S solo F5/F9/F6 o comandos explícitos; HDR final 20 B/píxel más staging,
código y alineación. Capturas diagnósticas esperan y descargan; fuera del visor.
Pruebas y límites: docs/EDITOR-OBJETOS.md, tests/objetos_editor.rs y ui/tests/objetos.rs.

Render final asíncrono: un trabajador y un canal de ocho estados, cancelación
atómica entre lotes. CPU prepara/envía/espera vallas fuera del bucle de ventana;
GPU conserva la salida, mismo pipeline/dos buffers/un comando por trabajo.
20 B/píxel de salida más staging equivalente; una descarga final o ninguna al
cancelar antes de copiar. Coexiste con el visor: pico físico no medido.
La E/S de render va en el trabajador; otras ediciones/guardados pueden esperar
a sus operaciones normales. No garantiza FPS ni cancelación de un despacho ya
enviado. Fuente de evidencia: docs/RENDER-ASINCRONO.md y tests/lotes.rs.

Visor progresivo: historial GPU de suma RGB f32/contador u32, 16 B/píxel más
4 B auxiliares, máximo 12 MiB +4 B lógicos a 1024×768. Compute añade una muestra
por cuadro hasta el objetivo; fragmentos solo leen la media. Una reserva compartida
entre fotogramas, con barreras; sin atómicos ni fragment stores. CPU prepara al
editar y envía constantes; no descarga historial en vivo. Exposición conserva datos,
las demás entradas de escena y tamaño reinician. Hay alineación, pipeline y
adjuntos adicionales; pico físico sin medir. Capturas/inspecciones diagnósticas
sí esperan y descargan. Pruebas y límites: docs/VISOR-PROGRESIVO.md.

EditorFuego expone ocho parámetros y vistas de campos procedurales. Valida números/Enter; la CPU recompone UI con cambios de tiempo. Sin simulación física ni lectura de VRAM.

EditorHumo valida dt fraccionario e iteraciones pares y expone vistas de campos. Conserva parámetros independientes del backend; no accede a GPU ni hace física.

Humo expone diez parámetros, incluidos dos tolerancias y presupuesto par. Confirma números y conserva campos; el estado de convergencia visual se dibuja desde GPU, no por sondeo CPU.

Catálogo no_std de ocho categorías con destinos tipados y funciones pendientes sin apertura. Composición CSS y texto ajustado por palabras, solo al cambiar vista. No ejecuta procesos ni simulaciones.

EditorHumo distingue 2D/3D con un indicador de presentación; la física y el render permanecen fuera de UI. El catálogo añade destino Humo3d. No reserva campos; triangulación CPU solo al cambiar UI. Pruebas de catálogo y de geometría del editor en cargo verify.

VolumenHumo añade cuatro parámetros y pestaña Tiempo/col., sin recursos GPU. Enter y sliders validan enteros/rangos antes de aplicar; subpasos predeterminados 2. Prueba: tests/humo.rs.

ExplosionHumo añade cuatro valores y cuarta pestaña Pulso, sin recursos GPU. Dominios/Enter/decimal con coma son transaccionales; duración cero conserva emisión continua. Pruebas en tests/humo.rs.

La cabecera 3D indica F8: GEO; la UI no adquiere dependencia de intercambio ni realiza E/S. La app resuelve la acción explícita.

Partículas GPU (28-10-2026): base independiente de 128 plazas; dos despachos,
6148 B lógicos compartidos y visor gaussiano hasta 384². CPU: constantes/eventos,
configuración y GEO explícito; referencia escalar solo tests. GPU: física y dibujo;
VRAM/RAM: costes lógicos y temporales descritos en docs/PARTICULAS-GPU.md, sin
medición del pico físico. E/S: v12 de 56 B y GEO de vivas; descarga explícita
5664 B al exportar, ninguna por cuadro estable. Sin acoplamiento al humo ni
compatibilidad externa certificada; perfil de ocho atributos separado del MAC.
Pruebas y captura reales: docs/PARTICULAS-GPU.md y telar-app/tests/particulas_gpu.rs.
El estado de riesgo de esta carpeta se conserva; no significa consumo cero.

Trazadores MAC (28-10-2026): riesgo de transporte/interpolación y accesos a cola
residentes, con límites comprobados. CPU: UI/constantes y E/S solicitada; referencia
escalar solo tests. GPU: hasta 128 transportes por subpaso y splats hasta 512²;
sin descarga estable ni fuerzas sobre gas. VRAM lógica a 32³: 2359300 B para
campos/cola, más imágenes/pipelines no medidos. RAM: vectores/texto al exportar.
Lecturas explícitas: D 576 B, GEO 5700 B descargados; configuración física v11, sin caché.
Ritmo de trabajo: diagnóstico/transporte cada subpaso; E/S solo explícita. Límites:
vida fija 2s, 128 plazas, sin estelas, colisión solo de destino, no producción.
Prueba/captura reales y siguiente contrato: docs/TRAZADORES-MAC.md. Se conserva
el estado de riesgo de la carpeta; no se anuncia consumo cero.


Plano VFX (29-10-2026): se conserva el estado de riesgo anterior; FFI sigue ROJO.
CPU prepara SPIR-V, interfaz, constantes, envios y E/S explicita; GPU ejecuta
MAC/reaccion, secundarios y pixeles. Actualizar parametros VFX sube 112 B;
no descarga campos por cuadro estable. Reserva logica compartida 4063236 B
a 32 cubico con salida final; staging, imagenes, alineacion y pipelines aparte,
sin pico fisico ni tiempos GPU medidos. RAM: copias/serializacion de cache y HDR
solo por solicitud. Render lee 1310720 B; cache reactiva copia 1015808 palabras,
archivo 4063512 B. Tests usan GPU real y crean archivos temporales en target.
Frecuencia de trabajo: simulacion por subpaso, visor 384 cuadrado; HDR 256 cuadrado,
1..1024 muestras en envios acotados. E/S de cache/escena/secuencia es optativa,
sin reemplazar resultados; salida VFX por lotes sin trabajador asincrono.
Limites adicionales: 32 cubico, 128 secundarios, cuatro posiciones de historial,
dispersion simple, sin oclusion de estelas ni color de produccion; cache ligada
a GPU/driver/Vulkan, requiere enlaces duros. No equivale a coste cero ni a Pyro.
Contrato, fuentes, evidencia real y pendientes: docs/PLANO-VFX.md.
