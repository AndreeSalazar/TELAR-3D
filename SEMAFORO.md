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
