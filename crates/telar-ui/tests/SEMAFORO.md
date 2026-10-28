# 🟡 Semáforo · crates/telar-ui/tests

Estado: AMARILLO
Motivo: Layout, validación de estilos y geometría de interfaz; riesgo de entradas y desbordes visuales.
CPU: Analiza CSS al cargar y compone triángulos cuando cambia estado/tamaño; calcula cajas y texto.
GPU: No ejecuta GPU directamente; los triángulos se dibujan por telar-gpu.
VRAM: Sin reserva directa; el backend conserva dos buffers UI y un atlas residente.
RAM: Reglas, métricas y Vec de vértices UI; geometría limitada por capacidad del backend.
E/S: No hay E/S en estos crates no_std; la app carga el CSS y la fuente.
Frecuencia: Carga/cambio de interfaz; la imagen se dibuja en cada fotograma. Esta carpeta se activa solo al lanzar sus pruebas/ejemplos.
Limites: No es CSS web completo; fuentes latinas, UI hasta 120000 vértices y atlas limitado.
Evidencia: crates/telar-ui/tests, crates/telar-estilo/tests y docs/SEMAFOROS.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Cada nodo conserva sus parámetros y el inspector expone solo los controles propios.
No calcula geometría 3D; envía valores a la app. Evidencia: tests/parametros.rs
y docs/NODOS-PARAMETROS.md.

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
E/S solo F5/F9/F6 o comandos explícitos; HDR síncrono 20 B/píxel más staging,
código y alineación. Capturas diagnósticas esperan y descargan; fuera del visor.
Pruebas y límites: docs/EDITOR-OBJETOS.md, tests/objetos_editor.rs y ui/tests/objetos.rs.

fuego.rs comprueba dominios de parámetros y confirmación numérica 32 + Enter; pruebas CPU sin reservas GPU.

humo.rs verifica dominios físicos del inspector y confirmación Enter con dt decimal e iteraciones pares.

humo.rs añade dominios de tolerancias y confirmación decimal con Enter.

catalogo.rs verifica hit testing, capacidades pendientes, acciones de una sola lectura, CSS transaccional y geometría dentro de ventana en cuatro resoluciones.

La prueba de humo añade Enter/decimal con coma para tiempo y esfera, rechazo transaccional de subpasos fraccionarios y dominios; no ejecuta shaders.

Humo prueba los cuatro controles de pulso con Enter y decimal con coma; rechaza valores fuera de rango/NaN sin cambiar configuración. No ejecuta simulación en CPU.
