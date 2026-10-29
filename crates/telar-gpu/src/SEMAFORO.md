# 🔴 Semáforo · crates/telar-gpu/src

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

Campos persistentes: plan de hasta 16 programas/256 pasadas, estado lógico <=64 MiB; a 128²/12 planos son 786436 B con dummy. CPU registra el orden, GPU calcula celdas. Cargas, reinicio y descarga son explícitos. Ver docs/HUMO-MAC-2D.md.

Plan de campos ampliado hasta 512 huecos; descarga explícita de rangos permite resumen de 64 B. El estado adaptativo 128² ocupa 983044 B lógicos con dummy. No evita enviar los huecos sobrantes ni promete mayor velocidad.

API de dibujo/captura solo UI permite el centro sin visor ni buffers de simulación. Conserva coste de destinos, profundidad, atlas y buffers UI; captura explícita descarga RGBA8.

PlanCampos admite 1..20 palabras por elemento con límite explícito de 64 MiB. El humo 3D usa 17, a 32³ reserva lógica de estado 2228224 B más palabra auxiliar y recursos gráficos. El backend sigue opaco a física/UI; no añade dependencia ni FFI. Ver docs/HUMO-3D.md y tests/volumen_gpu.rs de la app.

El límite de secuencia de campos sube de 512 a 2048 posiciones (preparación y cambio de plan), manteniendo máximo 16 programas y 64 MiB. Permite cuatro subpasos 3D acotados; no añade buffers ni operaciones físicas al backend. Coste CPU de registro y GPU de pasadas crece con el plan.


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
