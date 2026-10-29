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

Trazadores MAC (28-10-2026): riesgo de transporte/interpolación y accesos a cola
residentes, con límites comprobados. CPU: UI/constantes y E/S solicitada; referencia
escalar solo tests. GPU: hasta 128 transportes por subpaso y splats hasta 512²;
sin descarga estable ni fuerzas sobre gas. VRAM lógica a 32³: 2359300 B para
campos/cola, más imágenes/pipelines no medidos. RAM: vectores/texto al exportar.
Lecturas explícitas: D 576 B, GEO 5700 B descargados; configuración física v11, sin caché.
Ritmo de trabajo: diagnóstico/transporte cada subpaso; E/S solo explícita. Límites:
vida fija 2s, 128 plazas, sin estelas, colisión solo de destino, no producción.
Prueba/captura reales y siguiente contrato: docs/TRAZADORES-MAC.md. Se conserva
el estado de riesgo de la carpeta; no se anuncia consumo cero.


Plano VFX (29-10-2026): se conserva el estado de riesgo anterior; FFI sigue ROJO.
CPU prepara SPIR-V, interfaz, constantes, envios y E/S explicita; GPU ejecuta
MAC/reaccion, secundarios y pixeles. Actualizar parametros VFX sube 112 B;
no descarga campos por cuadro estable. Reserva logica compartida 4063236 B
a 32 cubico con salida final; staging, imagenes, alineacion y pipelines aparte,
sin pico fisico ni tiempos GPU medidos. RAM: copias/serializacion de cache y HDR
solo por solicitud. Render lee 1310720 B; cache reactiva copia 1015808 palabras,
archivo 4063512 B. Tests usan GPU real y crean archivos temporales en target.
Frecuencia de trabajo: simulacion por subpaso, visor 384 cuadrado; HDR 256 cuadrado,
1..1024 muestras en envios acotados. E/S de cache/escena/secuencia es optativa,
sin reemplazar resultados; salida VFX por lotes sin trabajador asincrono.
Limites adicionales: 32 cubico, 128 secundarios, cuatro posiciones de historial,
dispersion simple, sin oclusion de estelas ni color de produccion; cache ligada
a GPU/driver/Vulkan, requiere enlaces duros. No equivale a coste cero ni a Pyro.
Contrato, fuentes, evidencia real y pendientes: docs/PLANO-VFX.md.


Composicion VFX (29-10-2026): actualiza el limite anterior de estelas separadas.
Belleza suma puntos/segmentos con transmision aproximada del humo y esfera;
el diagnostico Particulas mantiene inspeccion sin extincion. CPU conserva
preparacion/eventos/envios y E/S explicita; no mezcla pixeles ni lee por cuadro.
GPU evalua hasta 128 puntos y cuatro segmentos por punto/pixel; cobertura
mayor que 1e-5 activa ocho muestras de transmision por aporte. No hay reserva
VRAM nueva, se conserva el contrato de 4063236 B logicos mas recursos del
backend. RAM/E/S adicionales solo compilacion, pruebas y capturas explicitas;
los ensayos usan target y temporales del sistema. Ritmo: cada pixel en visor
o muestra HDR; tiempo GPU y pico fisico siguen sin medir. Riesgo: coste por
pixel, aproximacion de oclusion de ocho pasos y contribucion aditiva sin
iluminar el gas; no garantiza FPS ni calidad profesional. Se conserva el
estado declarado de la carpeta. Evidencia y limites: docs/VFX-COMPUESTO.md.
