# 🟡 Semáforo · crates/telar-app/examples

Estado: AMARILLO
Motivo: Une eventos, formato, render y GPU; cambios pueden alterar el comportamiento del producto.
CPU: Eventos/cámara, parámetros y envío; .tlar se exporta/inspecciona solo por solicitud explícita.
GPU: Trabajo delegado al backend: geometría y render residentes, interfaz 2D.
VRAM: Indirecta mediante telar-gpu: destinos, malla, atlas y buffers; no se atribuye memoria libre al perfil.
RAM: Estado UI, palabras SPIR-V y staging/lecturas de pruebas. .tlar tiene límite de archivo.
E/S: Ventana, entrada y presentación; E/S .tlar y capturas solo en sus comandos.
Frecuencia: Arranque/fotograma; exportación, inspección y diagnóstico fuera del bucle vivo. Esta carpeta se activa solo al lanzar sus pruebas/ejemplos.
Limites: 32 B de parámetros procedurales por paso y 128 B de constantes de dibujo; pico de proceso no medido.
Evidencia: tests/app.rs, archivo_tlar.rs, render_gpu.rs y docs/ARRANQUE-GPU.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Grafo de rejilla y números: hasta diez identidades y 24 caracteres de edición.
Validar/compilar SPIR-V y recortar triángulos UI consume CPU/RAM al editar;
la evaluación de puntos sigue en GPU. Recompilar espera y reconstruye el plan;
pico de VRAM no medido. Estado estable: diez buffers, dos grabaciones, dos
mapeos y seis pipelines compute en la ruta fusionada (ocho en la histórica).
La escena de datos v3 ocupa hasta 416 B; E/S solo explícita. Evidencia adicional:
docs/GRAFO-EDITABLE.md, tests/grafo_gpu.rs y tests/red.rs de sus crates.

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

capturar_presion.rs genera 240 pasos y captura la página de tolerancias con datos reales GPU; descarga/PPM de diagnóstico fuera del producto en vivo.

La captura de presión 2D usa el bloque gráfico explícito; los adaptadores compartidos también conocen el visor 3D, sin iniciarlo en este ejemplo. Compilación comprobada por cargo verify.

Partículas GPU (28-10-2026): base independiente de 128 plazas; dos despachos,
6148 B lógicos compartidos y visor gaussiano hasta 384². CPU: constantes/eventos,
configuración y GEO explícito; referencia escalar solo tests. GPU: física y dibujo;
VRAM/RAM: costes lógicos y temporales descritos en docs/PARTICULAS-GPU.md, sin
medición del pico físico. E/S: v12 de 56 B y GEO de vivas; descarga explícita
5664 B al exportar, ninguna por cuadro estable. Sin acoplamiento al humo ni
compatibilidad externa certificada; perfil de ocho atributos separado del MAC.
Pruebas y captura reales: docs/PARTICULAS-GPU.md y telar-app/tests/particulas_gpu.rs.
El estado de riesgo de esta carpeta se conserva; no significa consumo cero.
