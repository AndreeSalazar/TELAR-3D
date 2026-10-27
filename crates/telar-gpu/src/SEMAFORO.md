# 🔴 Semáforo · crates/telar-gpu/src

Estado: ROJO
Motivo: Recursos Vulkan, sincronización y tiempos de vida de buffers/presentación.
CPU: Consultas al arrancar, preparación y envío por fotograma; descargas solo en diagnóstico.
GPU: Compute, despacho/dibujo indirectos, interfaz y rasterización; espera y barreras según el plan.
VRAM: Reserva real del backend: superficie de 10 buffers con arranque, atlas, UI, color y profundidad.
RAM: Parámetros, estructuras de recursos y staging; diagnósticos crean copias de lectura.
E/S: Carga de vulkan-1.dll y presentación; el kernel no accede al sistema de archivos.
Frecuencia: Preparación y fotograma; diagnóstico solo explícito.
Limites: Capacidad hasta 1024 por lado; control/cabecera 128 B cada uno. Pico total depende de resolución y asignaciones.
Evidencia: docs/ARRANQUE-GPU.md; crates/telar-app/tests/arranque_gpu.rs y render_gpu.rs.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

MSAA: consulta soporte por formato y conserva color/profundidad multimuestreados.
Coste lógico adicional 4x: 28 B/píxel (24,61 MiB a 1280x720 para RGBA8/D32),
sin contar alineación ni pico físico. Dibujo y resolución ocurren en GPU; la escena
medida pasa de 0,8330 a 1,4495 ms. DISTINTO, mayor coste por contorno más suave.
El host prepara y envía; no filtra píxeles en CPU durante el visor vivo. Ver docs/ANTIALIAS.md.

Calidad seleccionable 1x/2x/4x: capacidades guardadas por formato; cambiar
el modo efectivo espera trabajo GPU y reemplaza adjuntos/pipelines gráficos.
Compute se conserva. Incremento lógico 2x: 12 B/píxel; 4x: 28 B/píxel
(RGBA8/D32). El pico durante el reemplazo no se midió.
Evidencia adicional: docs/CALIDAD-RENDER.md y tests/calidad_visor.rs de telar-app.
