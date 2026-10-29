# 🟡 Semáforo · crates/telar-guard/src/reglas

Estado: AMARILLO
Motivo: Analiza archivos y bloquea verificaciones/commits; las reglas no demuestran semántica completa de shaders.
CPU: Recorrido/lectura del repo y reglas; verify lanza fmt, clippy y pruebas.
GPU: Ningún shader del guardián; verify puede ejecutar pruebas que sí usan GPU.
VRAM: Sin reserva directa; consumo indirecto de las pruebas invocadas.
RAM: Texto y estructuras del análisis, más procesos de herramientas; crece con el repo.
E/S: Fuentes, configuración de hooks y procesos de verificación; no opera por fotograma.
Frecuencia: Comando de guardián y hooks de verificación/commit.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: crates/telar-guard/tests/reglas.rs y docs/GUARDIANES.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

El grupo no_std añade telar-intercambio: sus fuentes no pueden usar std y sus dependencias siguen vigiladas por fronteras/cero-deps. No se relaja ninguna regla para añadir el serializador. Ver cargo verify.
