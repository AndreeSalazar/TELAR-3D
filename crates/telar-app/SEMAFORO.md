# 🟡 Semáforo · crates/telar-app

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
