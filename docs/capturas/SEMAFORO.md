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
