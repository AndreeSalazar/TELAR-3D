# 🟡 Semáforo · crates/telar-estilo/src

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
