# 🟡 Semáforo · crates/telar-render/src

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

Editor de objetos: dos cajas y luz con controles nativos, escena v5 de 184 B.
CPU prepara shaders al confirmar cambios; la vista traza 1 spp en GPU, limitada
a 1024×768, sin descarga viva ni reconstrucción UI en cuadros estables. Ediciones
esperan/reemplazan pipeline; pico transitorio no medido. La UI mantiene estado,
texto de hasta 24 caracteres y triángulos/atlas; usa buffers de overlay existentes.
E/S solo F5/F9/F6 o comandos explícitos; HDR síncrono 20 B/píxel más staging,
código y alineación. Capturas diagnósticas esperan y descargan; fuera del visor.
Pruebas y límites: docs/EDITOR-OBJETOS.md, tests/objetos_editor.rs y ui/tests/objetos.rs.

Visor progresivo: historial GPU de suma RGB f32/contador u32, 16 B/píxel más
4 B auxiliares, máximo 12 MiB +4 B lógicos a 1024×768. Compute añade una muestra
por cuadro hasta el objetivo; fragmentos solo leen la media. Una reserva compartida
entre fotogramas, con barreras; sin atómicos ni fragment stores. CPU prepara al
editar y envía constantes; no descarga historial en vivo. Exposición conserva datos,
las demás entradas de escena y tamaño reinician. Hay alineación, pipeline y
adjuntos adicionales; pico físico sin medir. Capturas/inspecciones diagnósticas
sí esperan y descargan. Pruebas y límites: docs/VISOR-PROGRESIVO.md.

fuego.rs emite marcha volumétrica procedural 16..128 pasos/píxel, sin buffers de campos ni atomics; coste proporcional a píxeles×pasos. No simula fluidos. Ver docs/FUEGO-PROCEDURAL.md.

humo.rs lee campos MAC residentes y los interpola para el visor; no descarga estado ni integra volumen 3D. Coste de fragmentos y cargas bilineales distinto al fuego procedural. Ver docs/HUMO-MAC-2D.md.

fragmentos_adaptativo lee control residente del plano 14 para franja de estado, sin readback de UI. La ruta fija de doce planos conserva su shader separado. Capturas de tres estados verificadas en Vulkan.

Visor de campos 3D: marcha de rayos de 16..128 muestras con trilineal, emisión/absorción diagnóstica; la app usa 64 y visor máximo 512². Lee los 17 planos del solver; no añade historial ni descargas en vivo. No incluye sombras/dispersión de producción. La velocidad requiere tres muestras por paso; coste GPU no medido. Prueba real: telar-app/tests/volumen_gpu.rs.

El visor 3D dibuja también la máscara esférica voxelizada en azul, evaluada en shader; sin buffer adicional. No implica malla suavizada, sombras ni luz de producción. Aumenta aritmética por muestra; coste no medido. Captura real: docs/capturas/humo-3d-colision.png.

La vista Calor 3D usa paleta artística por magnitud normalizada; no es radiancia de cuerpo negro. Sigue sin sombras/dispersión. Añade pocas operaciones escalares por muestra, sin buffers nuevos; capturas pulso-inicio/tarde en docs/capturas.

Partículas GPU (28-10-2026): base independiente de 128 plazas; dos despachos,
6148 B lógicos compartidos y visor gaussiano hasta 384². CPU: constantes/eventos,
configuración y GEO explícito; referencia escalar solo tests. GPU: física y dibujo;
VRAM/RAM: costes lógicos y temporales descritos en docs/PARTICULAS-GPU.md, sin
medición del pico físico. E/S: v12 de 56 B y GEO de vivas; descarga explícita
5664 B al exportar, ninguna por cuadro estable. Sin acoplamiento al humo ni
compatibilidad externa certificada; perfil de ocho atributos separado del MAC.
Pruebas y captura reales: docs/PARTICULAS-GPU.md y telar-app/tests/particulas_gpu.rs.
El estado de riesgo de esta carpeta se conserva; no significa consumo cero.
