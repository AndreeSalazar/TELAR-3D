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
