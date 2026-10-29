# 🔴 Semáforo · crates/telar-kernel/src

Estado: ROJO
Motivo: Emisión de programas y contratos de memoria/argumentos que ejecutará la GPU.
CPU: Construcción y validación de SPIR-V al preparar; .tlar usa lectura/CRC fuera del bucle de fotogramas.
GPU: Los programas generados ejecutan operaciones, coordinación y accesos SSBO según su contrato.
VRAM: Sin reserva propia del emisor; el backend reserva control, cabecera y datos que usarán los shaders.
RAM: Vec de palabras e IR; .tlar limita módulos a 1048576 palabras; pico depende de copias y programa.
E/S: Ninguna E/S directa del núcleo no_std; la app gestiona .tlar y los tests usan archivos temporales.
Frecuencia: Emisión al preparar; el programa generado ejecuta por paso o elemento según dominio.
Limites: Buzón de una entrada, productores serializados, región/ABI verificados; cola y arena generales pendientes.
Evidencia: docs/ARRANQUE-GPU.md, docs/FORMATO-TLAR.md y pruebas de kernel/hilo/SPIR-V.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

Extensión de caché optativa: 64 bytes adicionales de control por ranura; omite
la cadena de puntos con clave idéntica. Solo planes fijos/puramente funcionales.
Evidencia y medición: docs/RENDER-AREA.md.

Humo MAC 2D: emite seis kernels SPIR-V, propiedad de celda por plano y sin atómicos. CPU no simula en vivo; humo_referencia.rs es referencia escalar para pruebas. Presión aproximada, advección difusiva. Ver docs/HUMO-MAC-2D.md.

Presión adaptativa: residual máximo y reducción jerárquica sin atómicos; decisión GPU con tolerancia/presupuesto, 14 programas y hasta 327 pasadas. Estado de quince planos. Referencia CPU acepta cero iteraciones para verificar la estimación inicial. Ver docs/PRESION-ADAPTATIVA.md.

Humo MAC 3D: 17 planos a 4..64³, catorce programas y 7+5*(iter/2) posiciones por paso. Advección, fuerzas, presión y control en GPU; referencia escalar solo de ensayo. 68*N³+4 B lógicos compartidos, sin atómicos ni descargas vivas. El plan host sigue enviando las pasadas acotadas. Pruebas: telar-app/tests/volumen_gpu.rs; límites en docs/HUMO-3D.md.

Tiempo/esfera 3D: dos programas adicionales de reducción/decisión dt y stencil de caras fluidas, 16 programas únicos. 1..4 subpasos; máximo 1316 posiciones. Misma reserva 17 planos; aumenta el cómputo por máscara matemática y subpasos, sin benchmark. Reloj y predictor en control, sin lectura host viva. Prueba ampliada: telar-app/tests/volumen_gpu.rs; docs/TIEMPO-COLISION-3D.md.

Pulso: reducción determinista de máscara/cuenta fluida, fuente finita y objetivo compensado en caja cerrada. Control 16 congela el reloj antes de las pasadas, evitando lectura/escritura concurrente en diagnóstico. Diecisiete planos y dieciséis programas; parciales temporales de seis palabras por grupo caben en reserva. Más aritmética GPU, coste no medido. Referencia y pruebas en telar-app/tests/volumen_gpu.rs; docs/PULSO-EXPANSIVO-3D.md.

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
