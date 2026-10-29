# telar-intercambio

Contrato propio de muestras puntuales y subconjunto GEO clásico V5, no_std + alloc.
Sin bibliotecas externas, sistema de archivos, Vulkan ni simuladores.

`Punto` guarda posición, velocidad, densidad, calor, id y tiempo simulado.
`codificar_geo`/`decodificar_geo` validan esquema, valores, tamaños e identidades.
Máximo 65536 puntos y 16 MiB de texto. El lector acepta únicamente este esquema,
no un archivo GEO arbitrario. Comparación en Houdini: ABIERTO.

Uso, fuentes y límites: [INTERCAMBIO-GEO.md](../../docs/INTERCAMBIO-GEO.md).
