# 🟡 Semáforo · crates/telar-app/tests

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

fuego_gpu.rs valida SPIR-V y capturas reales 96×96, repetibilidad, campos y cambio de calidad. Solo las pruebas descargan imágenes; no es benchmark contra Pyro.

humo_gpu.rs contrasta doce planos con referencia a 17², proyección, pausa, cambio de secuencia y reinicio; descargas explícitas de prueba. No es comparación de rendimiento contra Houdini.

humo_adaptativo_gpu.rs verifica convergencia, límite, datos no finitos, resumen, franjas GPU y equivalencia con referencia a la cuenta efectiva; no es benchmark de velocidad.

volumen_gpu.rs valida catorce kernels SPIR-V y visor, compara 14 planos contra referencia a 9³, comprueba presupuesto, fuente, repetibilidad, cámara/pausa y reinicio con descargas explícitas. No es benchmark. Configuración v9 verifica corrupción y rechazo cruzado 2D/3D.

volumen_gpu.rs amplía referencia a dos subpasos con fuerzas y esfera: caras normales/celdas sólidas nulas, recorte CFL, reloj y repetibilidad. Valida 16 shaders; no benchmark. Formato v10 prueba corrupción y migración de v9.

El ensayo de volumen compara pulso activo, cruce de duración y fin de emisión a 9³ con y sin esfera; suma global de RHS <0.0001, error por valor <0.0003, reloj y repetibilidad. Valida los shaders y mantiene regresiones. V11 verifica CRC y migración v9/v10. Sin benchmark de rendimiento.

La prueba de volumen exporta un estado real a GEO a 9³, lee y compara todos los atributos, comprueba campos GPU intactos y rechazo de sobrescritura sin cambiar bytes. Puntos MAC verifica promedio de caras con valores fabricados. Descargas/E/S solo en ensayos; no comparación externa Houdini.

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


Esfera VFX (29-10-2026): la superficie y las sombras de esfera usan cuadraticas
analiticas GPU; la fisica conserva la mascara voxelizada de 32 cubico. CPU
conserva eventos/parametros/envios y no intersecta pixeles en el producto.
GPU recorta el volumen, sombrea la superficie y bloquea segmentos hacia luz
y secundarios; sustituye las pruebas voxelizadas de sombra por interseccion.
No hay buffers ni transferencias por cuadro adicionales; reserva logica VFX
4063236 B mas staging/imagenes/alineacion/pipelines, sin pico ni tiempos medidos.
RAM y E/S: escena propia de 200 B, pruebas y capturas explicitas; sin disco
por cuadro estable. Ritmo: cada rayo primario y segmento de visibilidad.
Restricciones de esta entrega: esfera unica, material mate fijo, ambiente artistico, sesgos numericos,
diferencia entre superficie continua y mascara fisica de una celda de orden;
sin mallas generales ni GI. Estado de riesgo anterior conservado.
Evidencia y contrato: docs/VFX-ESFERA.md, pruebas vfx_compuesto_gpu.rs y vfx_gpu.rs.


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


Compute reutilizable (29-10-2026): dos secundarios, un pool y dos claves de
32 palabras/version en RAM; memoria interna y residencia del driver sin medir.
CPU compara clave/plan y ejecuta el secundario; reconstruye solo al invalidar.
GPU conserva orden/despachos/barreras y estado de simulacion, sin buffers ni
transferencias adicionales por cuadro. No espera global nueva; reutiliza la
valla de cada juego. E/S solo informe solicitado y evidencias de desarrollo;
CSV vigente de 21 columnas. Ritmo: comprobacion por cuadro, grabacion inicial
o al cambiar fisica/plan/mascara; camara MAC 3D no invalida compute.
Precaucion: capturas/lotes siguen directos; cambios fisicos continuos aun
pagan grabacion, y el driver puede cobrar por ejecutar secundarios. FFI
conserva ROJO y los otros estados previos se mantienen. Pruebas de bytes,
contadores y sincronizacion: docs/COMPUTE-REUTILIZABLE.md.


Presion indirecta (29-10-2026): argumentos u32 acotados publicados por la
GPU en auxiliar binding 0 ampliado de 4 a 64 B (+60 B logicos). Los campos
fisicos no cambian de planos; VFX suma 4063296 B logicos mas recursos del
driver sin pico medido. CPU conserva plan/envio; no lee residual para decidir
grupo. GPU evita invocaciones de Jacobi/residual/reduccion con x=0; mantiene
ranuras/decisor pequeno y coste de leer argumentos. Productores sincronizados
hacia INDIRECT_COMMAND_READ y contra lecturas previas; FFI sigue ROJO.
No hay descarga nueva por cuadro; D/captura 3D lee 64 B explicitos adicionales,
640 B con trazadores. Monitor GPU pasa a medias de hasta 120 muestras.
Archivos de esta entrega: cache ABI revision 3, rechaza previas sin sobrescribir; .tlar sin cambio.
Ritmo: publicacion por decision; monitor 2 Hz. Beneficio dependiente de escena
y hardware; fallback --sin-indirectos. No es solver sparse ni compactacion
espacial. Se conservan los estados anteriores de riesgo. Pruebas, costes y
mediciones completas: docs/PRESION-INDIRECTA.md.
