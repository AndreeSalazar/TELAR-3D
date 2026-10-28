# 🟢 Semáforo · docs

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

El mapa Houdini y el plan de explosión enlazan el registro de Partio/BGEO en
referencias/. La anotación no instala herramientas, no cambia dependencias y no
certifica comparación externa; lectura de los documentos solo por consulta.

Escena v2: registro de 128 B; la UI serializa solo valores/posiciones y solicita
E/S a la app. Cámara/fase y CRC se validan antes de aplicar. Guardar/abrir es
síncrono y solo explícito; sin E/S de disco en cuadros estables. Reemplazo por
temporal en el mismo directorio; guardado no sube geometría ni añade buffers GPU.
La reapertura inicial reconstruye recursos; en vivo requiere igual capacidad.
Ver docs/ESCENAS.md y pruebas proyecto.rs, proyecto_render.rs y estado.rs.

La entrega de grafo agrega documentación de v3 (hasta 416 B de datos de escena),
edición numérica y contratos de ocho operaciones geométricas. El documento no
ejecuta el grafo; los costes de compilación/ejecución se describen por separado
en GRAFO-EDITABLE.md y los semáforos de UI, HILO y app.

HUMO-3D.md y PLAN-EXPLOSION-PRODUCCION.md documentan la base 32³ y los hitos pendientes, costes lógicos y evidencia numérica; no certifican equivalencia con Houdini.

TIEMPO-COLISION-3D.md documenta predictor CFL, límites de tiempo/esfera y formato v10; distingue colisión estática de mallas/movimiento pendientes. No ejecuta el solver.

PULSO-EXPANSIVO-3D.md separa fuente/expansión iniciales de combustión y render final pendientes; publica evidencia numérica y límites del dominio cerrado.
