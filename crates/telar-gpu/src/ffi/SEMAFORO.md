# 🔴 Semáforo · crates/telar-gpu/src/ffi

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

Render de rejilla: fragmentos leen geometría/control del mismo juego residente.
Barreras compute → fragmentos y fragmentos → compute protegen la reutilización.
Control reserva 192 bytes por juego (64 adicionales); capturas de diagnóstico
usan dos timestamps y descargan píxeles fuera del bucle vivo. Ver docs/RENDER-AREA.md.

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

MSAA ampliado: peticiones 1..64 en potencias de dos, limitadas por la intersección
de formatos color/profundidad. La petición no obliga a reservar muestras que
la GPU no soporte. Para N efectivo > 1, sobrecoste lógico RGBA8/D32 de 8*N-4 B
por píxel; 8x añade 52,73 MiB a 1280x720. Pico físico y tiempo de niveles altos
no medidos. Evidencia: tests/calidad_visor.rs y docs/CALIDAD-RENDER.md.

Editor de objetos: dos cajas y luz con controles nativos, escena v5 de 184 B.
CPU prepara shaders al confirmar cambios; la vista traza 1 spp en GPU, limitada
a 1024×768, sin descarga viva ni reconstrucción UI en cuadros estables. Ediciones
esperan/reemplazan pipeline; pico transitorio no medido. La UI mantiene estado,
texto de hasta 24 caracteres y triángulos/atlas; usa buffers de overlay existentes.
E/S solo F5/F9/F6 o comandos explícitos; HDR final 20 B/píxel más staging,
código y alineación. Capturas diagnósticas esperan y descargan; fuera del visor.
Pruebas y límites: docs/EDITOR-OBJETOS.md, tests/objetos_editor.rs y ui/tests/objetos.rs.

Render final asíncrono: un trabajador y un canal de ocho estados, cancelación
atómica entre lotes. CPU prepara/envía/espera vallas fuera del bucle de ventana;
GPU conserva la salida, mismo pipeline/dos buffers/un comando por trabajo.
20 B/píxel de salida más staging equivalente; una descarga final o ninguna al
cancelar antes de copiar. Coexiste con el visor: pico físico no medido.
La E/S de render va en el trabajador; otras ediciones/guardados pueden esperar
a sus operaciones normales. No garantiza FPS ni cancelación de un despacho ya
enviado. Fuente de evidencia: docs/RENDER-ASINCRONO.md y tests/lotes.rs.

Visor progresivo: historial GPU de suma RGB f32/contador u32, 16 B/píxel más
4 B auxiliares, máximo 12 MiB +4 B lógicos a 1024×768. Compute añade una muestra
por cuadro hasta el objetivo; fragmentos solo leen la media. Una reserva compartida
entre fotogramas, con barreras; sin atómicos ni fragment stores. CPU prepara al
editar y envía constantes; no descarga historial en vivo. Exposición conserva datos,
las demás entradas de escena y tamaño reinician. Hay alineación, pipeline y
adjuntos adicionales; pico físico sin medir. Capturas/inspecciones diagnósticas
sí esperan y descargan. Pruebas y límites: docs/VISOR-PROGRESIVO.md.

campos.rs conserva un SSBO, seis pipelines en humo y barreras entre pasadas/cuadros. Constantes compute de 128 B. Riesgo crítico FFI de vida útil y sincronización; validación de humo_gpu.rs y docs/HUMO-MAC-2D.md.

Transferencias por rango validan offsets, tamaño y barreras; humo_adaptativo_gpu.rs prueba el resumen de 64 B y rechazo de rangos vacíos/fuera del buffer. FFI sigue ROJO por sincronización y vida útil.

Captura permite escena opcional; el camino UI-only reutiliza los destinos/barreras de captura existentes. FFI sigue ROJO: vida de atlas, descriptores, destinos y descarga se valida en pruebas Vulkan.
