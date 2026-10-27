# Luz de área, autosombras y caché residente

Implementación propia del 27 de octubre de 2026. Esta etapa mejora el visor;
no integra RenderMan ni constituye todavía un render de producción completo.

## Qué ejecuta la GPU

El estudio usa una fuente cuadrada horizontal orientada hacia abajo, con cuatro
muestras estratificadas fijas. Cada muestra evalúa la BRDF GGX, coseno de emisión,
área y caída con distancia al cuadrado. La intensidad del bloque es radiancia
de la fuente en este modo, no la intensidad de la antigua luz direccional.
La configuración de fuente se compila al preparar; no hay controles de luz en UI aún.

La autosombra consulta **las alturas finales de los vértices residentes**, incluidos
los cambios de HILO. No replica una fórmula de onda en el shader. Avanza 32 pasos
por segmento hasta cada muestra e interpola los mismos dos triángulos de cada
celda. El relleno direccional y el ambiente siguen siendo aproximaciones sin sombras.

Contrato: rejilla regular x/z en [-2,2], posición/normal de seis f32 por vértice,
enlace 1 de vértices y enlace 3 de control, ambos de solo lectura en fragmentos.
El backend vincula la ranura correcta y publica compute → fragmentos/índices/indirecto.
Solo esos dos enlaces tienen visibilidad de fragmentos. Vulkan continúa aislado
del material y del compilador.

Es una **aproximación acotada de campo de alturas**: puede perder oclusores entre
pasos y tiene sesgo vertical de 0,015 unidades. Cuatro muestras pueden mostrar
transiciones discretas. No admite voladizos, otras mallas, transparencias ni sombras
de contacto exactas. No representa una estructura de aceleración general.

Referencia matemática de muestreo de área y emisión:
[PBRT, Area Lights](https://pbr-book.org/4ed/Light_Sources/Area_Lights).
La cuadratura y la marcha sobre la rejilla son nuestra implementación de vista previa.

## Estático y dinámico sin lecturas al host

`telar_kernel::compilar_con_cache` es una variante **optativa para planes inmutables**
cuyas operaciones por punto sean funciones de lado, fase, amplitud y los cuatro escalares. La GPU compara
sus bits y publica despacho cero para toda la cadena de puntos si nada cambió. Ver
[NODOS-PARAMETROS.md](NODOS-PARAMETROS.md).
La cámara y el material pueden cambiar sin deformar otra vez; el dibujo sí se repite.
No hay caché de píxeles ni de iluminación todavía.

La extensión ocupa control[32..48], 64 bytes adicionales por ranura. Conserva el
prefijo ABI v1 de 32 palabras y la cabecera TLAR v1: esa cabecera declara el mínimo,
no la longitud total reservada. El backend ahora reserva 192 bytes por control.
Si un ejecutor no ofrece 48 palabras de control y ocho de parámetros, esta variante
ejecuta sin caché y no lee la extensión. Fase, amplitud y lado están en [32..35], válida en 35, recalculadas
en 36 y omitidas en 37; 38..42 guarda los escalares propios y 42..48 queda reservado. Los contadores son modulares u32.

Regenerar topología invalida también deformación. Suspensión, vacío, exceso y
rechazo de arranque no publican una nueva clave. Cada juego en vuelo posee su
caché; al pausar hacen falta como máximo dos visitas para calentarlas ambas.
No usar esta variante con simulaciones acumulativas, tiempo oculto ni dependencias
externas: deben invalidarse explícitamente o usar el coordinador sin caché.
Cambiar el plan reconstruye sus recursos y por tanto reinicia su estado.

## Medición reproducible

Medición conservada del commit `a1c02e3`, anterior a la extensión de parámetros
propios. No mide el coordinador ampliado ni el relieve dinámico de esta entrega.

```
cargo run -p telar-app --release --example medir_render
```

Máquina: NVIDIA GeForce RTX 3060, Windows, Vulkan con validación y sincronización
activas. Captura 1280×720, rejilla 256×256, fase 0,6, amplitud 0,65, cámara a 7.
Dos calentamientos y nueve muestras intercaladas por variante; medianas de timestamps
GPU. No son FPS ni una comparación con RenderMan.

| Variante | Mediana ms | Mínimo ms | Máximo ms |
|---|---:|---:|---:|
| Área, cuatro muestras, sin sombra | 0,0476 | 0,0468 | 0,0492 |
| Área, una muestra, 32 pasos de sombra | 0,2313 | 0,2286 | 0,2339 |
| Área, cuatro muestras, 32 pasos de sombra | 0,8338 | 0,8228 | 0,8422 |

**DISTINTO:** aumenta el coste y cambia la iluminación. Las variantes de una y cuatro
muestras también difieren en cuadratura, no solo en cantidad de sombras. Los tiempos
incluyen dibujo, limpieza, barreras y overlay vacío; excluyen compilación, creación,
compute, envío CPU y descarga. La lectura espera una valla y es diagnóstica; nunca
decide trabajo en el visor vivo. Una cola sin timestamps devuelve `None` en la API.
Las consultas respetan bits válidos y periodo del dispositivo, según
[Khronos, timestamp queries](https://docs.vulkan.org/samples/latest/samples/api/timestamp_queries/README.html).

Una muestra adicional de compute: deformación dinámica 10.240 ns frente a despacho
vacío 2.048 ns. Es una observación, **ABIERTO** como benchmark estadístico. El coste
del coordinador y las barreras permanece. GPU publicó 1024 grupos frente a cero.
**IGUAL:** la geometría con caché coincide bit a bit con el plan sin caché en las
pruebas de cambios, repeticiones, vacío y recuperación. No se sube geometría desde CPU.

## Consumo y evidencia

La marcha de cuatro muestras × 32 pasos × cuatro alturas hace hasta 512 cargas f32
por fragmento: 2048 bytes lógicos, no tráfico físico medido gracias a las cachés.
No reserva otro mapa de sombras. Control añade 64 bytes por juego sobre el prefijo (128 en el visor).
Los buffers de geometría, índices, destinos y profundidad siguen en el backend.
Pico total de VRAM y coste en otras escenas/dispositivos: **ABIERTO**.

Pruebas: `sombras_gpu.rs` (plano sin autosombra, oclusión sin energía añadida,
determinismo y validación), `cache_deformacion_gpu.rs` (referencia sin caché,
invalidación y recuperación), validador oficial SPIR-V y prueba oculta de 120 cuadros.
Los tests anteriores de caché de topología se conservan en `cache_gpu.rs`.

## Siguiente etapa: render progresivo

Construir transporte de luz requiere otra ruta: BVH para geometría general,
intersecciones verificadas, estado de muestreo por píxel y acumulación HDR en VRAM.
Una generación de escena/cámara/material/luz invalidará esa acumulación desde GPU.
Después: rebotes acotados, muestreo de luz/BRDF y pruebas contra integrales conocidas.
La imagen de vista previa puede seguir interactiva mientras ese modo converge.
Todavía **no están implementados** el trazado de caminos, acumulación, GI ni denoiser.
