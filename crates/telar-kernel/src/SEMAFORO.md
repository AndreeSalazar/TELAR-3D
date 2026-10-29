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
