# 🟡 Semáforo · crates/telar-render/tests

Estado: AMARILLO
Motivo: Matemáticas de material e iluminación; coste por fragmento y equivalencia visual aún parcial.
CPU: Emite shaders al preparar y empaqueta 128 bytes de constantes por dibujo; no ilumina píxeles en CPU.
GPU: GGX, luz de área y marcha de autosombra acotada (hasta 512 cargas de altura por fragmento con la configuración del visor), ambiente y tone mapping; vértices entregan posición/normal.
VRAM: No reserva directamente; usa malla y destinos que posee telar-gpu. Sin texturas ni acumulación propia todavía.
RAM: Código SPIR-V y parámetros; el tamaño crece con el programa emitido, no con cada píxel en CPU.
E/S: Sin E/S propia. Las capturas y pruebas leen resultados mediante el backend.
Frecuencia: Preparación, vértice y fragmento; las pruebas se ejecutan bajo demanda. Esta carpeta se activa solo al lanzar sus pruebas/ejemplos.
Limites: Rugosidad efectiva >=0.08; energía validada. Mediana de dibujo 0,834 ms en la escena RTX 3060 documentada; ABIERTO: otras escenas y pico VRAM; no es path tracing ni RenderMan.
Evidencia: crates/telar-render/README.md; tests/render.rs, crates/telar-app/tests/sombras_gpu.rs y docs/RENDER-AREA.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.
