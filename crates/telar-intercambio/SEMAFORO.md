# Semáforo · crates/telar-intercambio

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
