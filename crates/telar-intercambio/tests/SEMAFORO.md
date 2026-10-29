# Semáforo · crates/telar-intercambio/tests

Estado: AMARILLO
Motivo: Contrato y análisis de archivos con límites explícitos; datos externos pueden ser inválidos.
CPU: Validación, conversión y formato ASCII solo al solicitar intercambio o durante pruebas.
GPU: No ejecuta shaders; el crate no conoce el backend ni la simulación.
VRAM: No reserva VRAM; una exportación host puede solicitar lectura explícita al backend.
RAM: Hasta 65536 puntos de 40 B, texto máximo 16 MiB y vector auxiliar de identificadores; pico físico no medido.
E/S: Ninguna dentro del crate no_std; la app se encarga de los archivos.
Frecuencia: Exportación/inspección explícita y ensayos; no por fotograma vivo.
Limites: Subconjunto GEO clásico V5, cinco atributos fijos, sin mallas, BGEO, compresión ni caché reanudable.
Evidencia: tests/geo.rs, docs/INTERCAMBIO-GEO.md y pruebas de exportación de telar-app; comparación Houdini ABIERTO.

Partículas GPU (28-10-2026): base independiente de 128 plazas; dos despachos,
6148 B lógicos compartidos y visor gaussiano hasta 384². CPU: constantes/eventos,
configuración y GEO explícito; referencia escalar solo tests. GPU: física y dibujo;
VRAM/RAM: costes lógicos y temporales descritos en docs/PARTICULAS-GPU.md, sin
medición del pico físico. E/S: v12 de 56 B y GEO de vivas; descarga explícita
5664 B al exportar, ninguna por cuadro estable. Sin acoplamiento al humo ni
compatibilidad externa certificada; perfil de ocho atributos separado del MAC.
Pruebas y captura reales: docs/PARTICULAS-GPU.md y telar-app/tests/particulas_gpu.rs.
El estado de riesgo de esta carpeta se conserva; no significa consumo cero.
