# 🟡 Semáforo · crates/telar-ventana/tests

Estado: AMARILLO
Motivo: Manejadores y memoria nativa Win32/GDI; restauración y destrucción deben conservar su orden.
CPU: Eventos Win32, ventana y rasterización inicial del atlas de fuente en GDI.
GPU: No emite comandos Vulkan; la composición del escritorio es responsabilidad del sistema.
VRAM: No reserva VRAM de TELAR directamente; el atlas se entrega al backend GPU.
RAM: Bitmap del atlas, cobertura, eventos y objetos nativos; su vida se limita por RAII.
E/S: Entrada, ventana, fuente del sistema y llamadas Win32/GDI.
Frecuencia: Eventos; atlas al preparar la fuente, no rasterización de letras por fotograma. Esta carpeta se activa solo al lanzar sus pruebas/ejemplos.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: crates/telar-ventana/tests y contratos SAFETY de src/ffi/.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.
