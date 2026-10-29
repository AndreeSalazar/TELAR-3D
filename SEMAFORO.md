# 🔴 Semáforo · .

Estado: ROJO
Motivo: Incluye descendientes críticos; Agrupa ejecución, memoria y fronteras críticas; el color refleja el máximo de los hijos.
CPU: Preparación, herramientas y costes de descendientes; no implica trabajo por existir la carpeta.
GPU: Consumo indirecto según descendientes; consultar los informes de fuentes y pruebas.
VRAM: Sin reserva propia por la carpeta; las reservas pertenecen al backend y a sus descendientes.
RAM: Archivos, estructuras y compilación; pico no medido en esta agrupación.
E/S: Lectura de fuentes y herramientas de desarrollo; rutas de producto indicadas en descendientes.
Frecuencia: Al preparar, compilar o ejecutar las piezas contenidas; consultar hojas hijas.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: docs/SEMAFOROS.md y contratos/pruebas de las carpetas descendientes.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

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
