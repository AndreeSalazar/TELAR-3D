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

Humo 3D conserva 17 planos, añade cuatro constantes de tiempo/esfera y .tlar v10 de 72 B. CPU recompone el plan solo al cambiar presupuesto/subpasos; GPU calcula dt y reloj. D sigue descargando 64 B por solicitud. Captura y límites: docs/TIEMPO-COLISION-3D.md.

--explosion configura el laboratorio 3D y la pestaña Pulso; CPU solo compone constantes, plan y eventos. El GPU decide emisión por reloj congelado. .tlar v11 de 88 B migra v9/v10 sin pulso. D permanece en 64 B; no caché de campos. Pruebas/formato y capturas descritos en docs/PULSO-EXPANSIVO-3D.md.

F8/--exportar-geo solicita 64 B de control + cinco planos físicos; a 32³ descarga 655424 B. CPU centra velocidades MAC, valida y serializa hasta 65536 muestras/16 MiB de texto. No hay lecturas adicionales en cuadros estables. La escritura es síncrona, destino exclusivo nuevo; error puede dejar archivo incompleto. --inspeccionar-geo usa solo CPU/RAM/E/S, sin contexto Vulkan. Ver docs/INTERCAMBIO-GEO.md y tests/volumen_gpu.rs.

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


Monitor GPU vivo (29-10-2026): actualiza el contrato anterior de cero lecturas
vivas solo cuando el monitor esta activo. CPU recoge cuatro timestamps y 16
palabras de control tras la valla habitual del juego, compone panel a 2 Hz,
consulta heaps al driver y conserva hasta 120 cuadros/4096 filas CSV en RAM.
GPU graba marcas y copia 64 B por cuadro observado de humo/VFX; usa barreras
explicitas, sin envio separado ni espera global nueva por cuadro. VRAM/RAM:
dos buffers coherentes de 64 B (128 B logicos) y ocho queries, mas alineacion
y memoria interna del driver no medidas; buffers se conservan al ocultar.
El uso/presupuesto de heap son estimaciones de proceso, no toda la VRAM libre.
No hay E/S de disco por cuadro; CSV/TXT exclusivos al cerrar si se solicitan.
Ritmo: marcas/snapshot por cuadro; texto y presupuesto cada 500 ms; datos
GPU diferidos hasta dos cuadros. Temperatura, potencia, frecuencia y ocupacion
global no disponibles. Observar tiene coste y puede alterar tiempos; no se
anuncia velocidad universal ni consumo cero. Semaforo de memoria 70/90% y
estado numerico; FFI conserva ROJO y los demas estados previos se mantienen.
Comprobacion de esta entrega: docs/MONITOR-GPU.md y telar-app/tests/telemetria_gpu.rs.


Desglose host (29-10-2026): reloj monotono opcional en preparacion, valla,
recogida, adquisicion, grabacion, envio y presentacion; medias de hasta 120
registros en RAM. Son tiempos de pared, no porcentaje CPU. Movimiento libre,
orbita y rueda conservan triangulos UI; un deslizador activo si los invalida.
GPU/VRAM: no hay buffers, transferencias ni esperas nuevas; permanece el coste
del monitor anterior. E/S solo informe CSV/TXT solicitado (18 columnas CSV),
sin disco por cuadro. Ritmo de medicion: cada cuadro con monitor; refresco 2 Hz.
Limitacion adicional: aun se registra el plan compute por cuadro; no se afirma
soberania total ni velocidad universal. Estados anteriores, incluido FFI ROJO,
se conservan. Contrato, pruebas y siguiente paso: docs/HOST-ENVIO.md.
