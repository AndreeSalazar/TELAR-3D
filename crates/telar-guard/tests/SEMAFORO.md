# 🟡 Semáforo · crates/telar-guard/tests

Estado: AMARILLO
Motivo: Analiza archivos y bloquea verificaciones/commits; las reglas no demuestran semántica completa de shaders.
CPU: Recorrido/lectura del repo y reglas; verify lanza fmt, clippy y pruebas.
GPU: Ningún shader del guardián; verify puede ejecutar pruebas que sí usan GPU.
VRAM: Sin reserva directa; consumo indirecto de las pruebas invocadas.
RAM: Texto y estructuras del análisis, más procesos de herramientas; crece con el repo.
E/S: Fuentes, configuración de hooks y procesos de verificación; no opera por fotograma.
Frecuencia: Comando de guardián y hooks de verificación/commit. Esta carpeta se activa solo al lanzar sus pruebas/ejemplos.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: crates/telar-guard/tests/reglas.rs y docs/GUARDIANES.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.
