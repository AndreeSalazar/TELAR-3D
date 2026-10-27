# 🔴 Semáforo · crates/hilo/src

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

Seno/Coseno escalares: validación y emisión del IR en CPU al preparar;
intérprete CPU solo de referencia. El SPIR-V evalúa trigonometría por punto en GPU,
sin añadir descriptores ni reservas propias. Hasta 256 instrucciones IR.
Ver docs/GRAFO-EDITABLE.md y telar-app/tests/grafo_gpu.rs.
