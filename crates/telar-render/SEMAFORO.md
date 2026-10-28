# 🟡 Semáforo · crates/telar-render

Estado: AMARILLO
Motivo: Matemáticas de material e iluminación; coste por fragmento y equivalencia visual aún parcial.
CPU: Emite shaders al preparar y empaqueta 128 bytes de constantes por dibujo; no ilumina píxeles en CPU.
GPU: GGX, luz de área y marcha de autosombra acotada (hasta 512 cargas de altura por fragmento con la configuración del visor), ambiente y tone mapping; vértices entregan posición/normal.
VRAM: No reserva directamente; usa malla y destinos que posee telar-gpu. Sin texturas ni acumulación propia todavía.
RAM: Código SPIR-V y parámetros; el tamaño crece con el programa emitido, no con cada píxel en CPU.
E/S: Sin E/S propia. Las capturas y pruebas leen resultados mediante el backend.
Frecuencia: Preparación, vértice y fragmento; las pruebas se ejecutan bajo demanda.
Limites: Rugosidad efectiva >=0.08; energía validada. Mediana de dibujo 0,834 ms en la escena RTX 3060 documentada; ABIERTO: otras escenas y pico VRAM; no es path tracing ni RenderMan.
Evidencia: crates/telar-render/README.md; tests/render.rs, crates/telar-app/tests/sombras_gpu.rs y docs/RENDER-AREA.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Intersecciones experimentales: emisor rayo–triángulo sin buffers propios ni
lecturas host. Solo los tests preparan casos constantes y descargan resultados:
16 B por consulta más guardas, aparte del staging/alineación del backend.
Aritmética GPU por par rayo–triángulo; coste temporal y pico de memoria abiertos.
Sin BVH ni integración al visor; umbral absoluto y aristas generales pendientes.
Ver también docs/INTERSECCIONES-GPU.md y crates/telar-app/tests/rayos_gpu.rs.

Render por lotes: preparación de escena/shader en CPU; intersecciones, sombra,
muestreo, media HDR y revisión sRGB en GPU. Reserva lógica 20 B/píxel de salida
más 20 B/píxel de staging, alineación/código adicionales; descarga final única.
Máximo 512 por eje, 1024 spp y 1048576 muestras totales, 32 triángulos.
RAM para resultados y PFM/PPM; E/S explícita por --render-rayos.
Sin BVH ni historial entre envíos; nuevas ejecuciones reconstruyen recursos.
No se afirma pico físico ni rendimiento universal; ver docs/RENDER-POR-LOTES.md
y crates/telar-app/tests/render_final.rs para evidencia RTX 3060.
