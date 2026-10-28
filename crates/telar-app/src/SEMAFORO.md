# 🟡 Semáforo · crates/telar-app/src

Estado: AMARILLO
Motivo: Une eventos, formato, render y GPU; cambios pueden alterar el comportamiento del producto.
CPU: Eventos/cámara, parámetros y envío; .tlar se exporta/inspecciona solo por solicitud explícita.
GPU: Trabajo delegado al backend: geometría y render residentes, interfaz 2D.
VRAM: Indirecta mediante telar-gpu: destinos, malla, atlas y buffers; no se atribuye memoria libre al perfil.
RAM: Estado UI, palabras SPIR-V y staging/lecturas de pruebas. .tlar tiene límite de archivo.
E/S: Ventana, entrada y presentación; E/S .tlar y capturas solo en sus comandos.
Frecuencia: Arranque/fotograma; exportación, inspección y diagnóstico fuera del bucle vivo.
Limites: 32 B de parámetros procedurales por paso y 128 B de constantes de dibujo; pico de proceso no medido.
Evidencia: tests/app.rs, archivo_tlar.rs, render_gpu.rs y docs/ARRANQUE-GPU.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Calidad seleccionable 1x/2x/4x: capacidades guardadas por formato; cambiar
el modo efectivo espera trabajo GPU y reemplaza adjuntos/pipelines gráficos.
Compute se conserva. Incremento lógico 2x: 12 B/píxel; 4x: 28 B/píxel
(RGBA8/D32). El pico durante el reemplazo no se midió.
Evidencia adicional: docs/CALIDAD-RENDER.md y tests/calidad_visor.rs de telar-app.

Escena v2: registro de 128 B; la UI serializa solo valores/posiciones y solicita
E/S a la app. Cámara/fase y CRC se validan antes de aplicar. Guardar/abrir es
síncrono y solo explícito; sin E/S de disco en cuadros estables. Reemplazo por
temporal en el mismo directorio; guardado no sube geometría ni añade buffers GPU.
La reapertura inicial reconstruye recursos; en vivo requiere igual capacidad.
Ver docs/ESCENAS.md y pruebas proyecto.rs, proyecto_render.rs y estado.rs.

Grafo de rejilla y números: hasta diez identidades y 24 caracteres de edición.
Validar/compilar SPIR-V y recortar triángulos UI consume CPU/RAM al editar;
la evaluación de puntos sigue en GPU. Recompilar espera y reconstruye el plan;
pico de VRAM no medido. Estado estable: diez buffers, dos grabaciones, dos
mapeos y seis pipelines compute en la ruta fusionada (ocho en la histórica).
La escena de datos v3 ocupa hasta 416 B; E/S solo explícita. Evidencia adicional:
docs/GRAFO-EDITABLE.md, tests/grafo_gpu.rs y tests/red.rs de sus crates.

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

Fuego procedural: --fuego envía constantes y tiempo; GPU integra 16..128 pasos por píxel. .tlar v6 de 48 B, captura PPM explícita. No solver físico. Ver docs/FUEGO-PROCEDURAL.md.

--humo compone parámetros y envíos; ocho controles .tlar v7, pausa/paso/reinicio y D para descarga explícita (786432 B a 128²). No hay lectura de campos en el bucle vivo ni solver CPU. La captura PPM es diagnóstica.

--humo usa presión adaptativa en GPU; D solicita solo 64 B de control. .tlar v8 de 56 B carga v7 con tolerancias por defecto. La CPU sigue registrando un plan completo acotado; no se atribuye autonomía total al backend.

--estudio organiza categorías y abre editores conocidos en ventanas independientes mediante Command, sin shell. No mata procesos ni transfiere escenas. Un registro local evita duplicar cada tipo de editor desde esa sesión.

--humo-3d compone MAC 32³ y visor orbitable 512²/64 pasos; campos y presión residen en GPU, CPU maneja cámara/constantes/envíos. D descarga 64 B; F5/F9 guarda/abre configuración v9 de 56 B y reinicia campos al abrir. Reserva lógica 2228228 B más gráficos/alineación; pico físico y FPS sin medir. No hay caché ni combustión. Evidencia adicional: docs/HUMO-3D.md y tests/volumen_gpu.rs.
