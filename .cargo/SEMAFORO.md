# 🟡 Semáforo · .cargo

Estado: AMARILLO
Motivo: Configuración y hooks que pueden lanzar compilación y verificación.
CPU: Cargo, guardián, análisis y pruebas cuando se invocan; sin cálculo por vértice en vivo.
GPU: Las verificaciones pueden lanzar pruebas GPU; la configuración no despacha shaders por sí sola.
VRAM: Indirecta: las pruebas reservan sus buffers y destinos; no hay reserva propia del archivo de configuración.
RAM: Procesos de compilación y pruebas; pico no medido.
E/S: Lectura de configuración, fuentes y binarios de pruebas; los hooks operan al verificar/commitear.
Frecuencia: Desarrollo y verificación; no se ejecuta como parte del sombreado de un fotograma.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: crates/telar-guard/src/verificar.rs y crates/telar-guard/tests/piezas.rs.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.
