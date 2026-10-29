# 🟢 Semáforo · docs/capturas

Estado: VERDE
Motivo: Documentación, referencias o datos; no se ejecutan automáticamente.
CPU: Sin trabajo de producto directo; leer, comparar o visualizar consume recursos de la herramienta.
GPU: Ningún despacho propio; una captura es un resultado, no un programa activo.
VRAM: No reserva VRAM del producto; un visor externo puede utilizar su propia memoria.
RAM: Tamaño de los documentos y datos al abrirlos; no reside por fotograma en TELAR.
E/S: Lectura o escritura de documentación, capturas y ejemplos cuando se solicitan.
Frecuencia: Solo consulta, actualización o prueba explícita del contenido.
Limites: Los .tlar son datos de arranque; su CRC no autentica código. Referencias pendientes se etiquetan ABIERTAS.
Evidencia: docs/FORMATO-TLAR.md, docs/ARRANQUE-GPU.md y docs/SEMAFOROS.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

humo-mac-2d.png procede de la captura real de densidad GPU a 128², no de una simulación 3D ni de Houdini.

presion-adaptativa.png captura controles y franja del estado GPU después de 240 pasos a 128² en RTX 3060; no es una prueba de rendimiento.

estudio-categorias.png procede de la captura Vulkan de la UI nativa; las tarjetas Pendiente describen planes, no funciones ya implementadas.

humo-3d.png es captura GPU real de 180 pasos a 32³ en RTX 3060; PPM convertido a PNG sin modificar el contenido. No representa una explosión de producción ni medición de FPS.

humo-3d-colision.png procede de 240 cuadros GPU reales a 32³ con esfera estática y dos subpasos; conversión PPM a PNG sin retoque. La máscara es voxelizada, no superficie final de producción.

pulso-inicio.png y pulso-tarde.png son capturas reales del preset --explosion a 20/100 cuadros en RTX 3060. PPM convertido a PNG sin retoques; no equivalencia con una explosión cinematográfica.

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
