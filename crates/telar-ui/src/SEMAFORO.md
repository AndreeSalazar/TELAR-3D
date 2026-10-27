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
