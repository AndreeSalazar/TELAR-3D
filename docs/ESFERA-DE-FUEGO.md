# La esfera de fuego: la estrella del norte

Una bola de fuego **ultrarrealista**, simulada y renderizada entera en la GPU, que se pueda mirar en vivo y exportar en alta calidad. Cada pieza de TELAR existe para llegar aquí.

## Qué la hace realista (y no un efecto pintado)

| Lo que ves | De dónde sale |
|---|---|
| El color: rojo, naranja, amarillo, núcleo casi blanco | la **temperatura**, convertida en color con la ley de Planck: física, no una paleta |
| Los rizos y remolinos | advección con poca difusión (MacCormack), **confinamiento de vorticidad** y *curl noise* |
| Que la bola crezca y empuje | la **combustión** expande el gas (divergencia) |
| Que suba y se enrolle | la **flotación**: lo caliente sube y lo frío baja |
| El humo oscuro que la envuelve | el **hollín** que deja la combustión, que absorbe la luz |
| La profundidad y el volumen | **marcha de rayos** con sombras propias y dispersión |
| El brillo sin quemar la imagen | HDR, exposición y **tonemapping** |

## Los campos (todos viven en la GPU)

| Campo | Tipo | Para qué |
|---|---|---|
| `vel` | `f32` × 3, rejilla MAC | hacia dónde se mueve el gas |
| `combustible` | `f32` | lo que queda por quemar |
| `temperatura` | `f32` | el calor: flotación y color |
| `hollín` | `f32` (o `f16`, si se mide igual) | el humo: absorción |
| `divergencia` | `f32` | la expansión de la combustión |
| `presión` | `f32` | lo que mantiene el gas incompresible |

Cada campo que se advecta tiene **dos copias** (leer de una y escribir en la otra), así que no hay carreras entre hilos.

## Un paso de simulación, en orden

Cada paso es un kernel (o varios fusionados por HILO). La CPU no interviene.

1. **Emitir:** una esfera con ruido fBm deja combustible, temperatura y una velocidad hacia fuera. El ruido rompe la simetría, como en una explosión real.
2. **Advectar** la velocidad y los escalares (MacCormack, con un limitador para que no aparezcan valores falsos).
3. **Quemar:** donde la temperatura supera la de ignición, el combustible baja, y la temperatura, el hollín y la divergencia suben.
4. **Fuerzas:**
   - flotación;
   - confinamiento de vorticidad;
   - turbulencia con *curl noise*;
   - enfriamiento proporcional a `T⁴`;
   - disipación suave del hollín.
5. **Proyectar:** calcular la divergencia (descontando la expansión deseada), resolver la presión y restar su gradiente. Es el paso más caro: primero Jacobi con iteraciones fijas, multigrid después.
6. **Bordes:** abiertos, para que el humo salga sin rebotar.
7. **(Después) Ladrillos activos:** un kernel cuenta dónde hay algo y escribe él mismo cuánto trabajo lanzar en el paso siguiente. Reutiliza el ejecutor residente y el despacho indirecto de la Fase 4.5; aquí se añade la estructura dispersa.

## Un fotograma de render

1. **Tabla de color:** al arrancar, la GPU calcula una tabla «temperatura → RGB» con la ley de Planck y las funciones CIE, y la guarda en una textura 1D.
2. **Marcha de rayos** por píxel, con *jitter*:
   - **emisión:** el color de la temperatura, por la intensidad;
   - **absorción:** según el hollín (Beer–Lambert);
   - **dispersión:** Henyey–Greenstein, con una segunda marcha corta hacia la luz principal para las sombras.
3. **Acumulación temporal** (solo en vivo): mezclar con el fotograma anterior para quitar el ruido del *jitter*.
4. **Posproceso:** *bloom* desde los píxeles más calientes, exposición, tonemapping y un grano fino.

## Dos modos

| | En vivo | Final |
|---|---|---|
| Rejilla | 128³ (≈2 M de celdas) | 256³–384³ (17–57 M) |
| Presión | Jacobi, ~30 iteraciones | multigrid, hasta converger |
| Render | ~64 pasos por rayo + acumulación temporal | 256+ pasos, varias muestras por píxel |
| Objetivo | 60 fps en la RTX 3060 | calidad; segundos por fotograma está bien |

**Memoria (estimación, a medir):** a 256³, unos 1–1,5 GB con las dobles copias; cabe en la RTX 3060.

**Tiempo (estimación, a medir):** a 256³, la presión domina: decenas de ms por paso con Jacobi, mucho menos con multigrid. A 128³ es 8 veces menos trabajo.

Las cifras reales se medirán con temporizadores de la GPU y se publicarán con su etiqueta.

## Cómo se verifica

- **Cada kernel contra la CPU:** una rejilla pequeña (32³), la misma entrada y la implementación de referencia en `no_std`. Mismo resultado dentro de la tolerancia, o bit a bit donde se exige determinismo.
- **Física:**
  - tras proyectar, la divergencia debe quedar por debajo de ε;
  - sin fuerzas, la advección no puede crear masa.
- **Determinismo:** la misma semilla, N pasos y el mismo hash de todos los campos, en cada ejecución.
- **Contra Houdini:**
  - una bola de fuego de Pyro con parámetros parecidos;
  - captura con la misma cámara y tiempo por fotograma en la misma máquina;
  - etiqueta: IGUAL, MÁS RÁPIDO, DISTINTO (si el algoritmo es otro, se explica) o ABIERTO.

## Hitos de la esfera

1. **Humo 2D** (256²) en la GPU: advección, flotación y presión. Para entender cada paso viéndolo.
2. **Humo 3D** (128³) con marcha de rayos y absorción.
3. **Fuego:** combustión, divergencia y color de cuerpo negro.
4. **Detalle:** MacCormack, vorticidad y *curl noise*.
5. **Luz:** sombras propias, dispersión y *bloom*.
6. **Ladrillos activos:** estructura dispersa sobre el despacho indirecto demostrado desde la Fase 4.5.
7. **Final:** 256³ o más con multigrid, y comparación publicada contra Houdini.
8. **(P2) Chispas:** partículas que nacen donde más arde.

## Estado real: 28 de octubre de 2026

Ya existe una primera [esfera procedural](FUEGO-PROCEDURAL.md) con marcha de rayos GPU, campos analíticos y controles guardables. No completa los hitos físicos de esta página: no tiene advección, presión ni combustión. Las cifras anteriores son objetivos/estimaciones sin benchmark actual; la CPU aún prepara y envía el trabajo Vulkan. El plan para artistas está en [VFX-PARA-ARTISTAS.md](VFX-PARA-ARTISTAS.md).

## Hito inicial de humo 2D implementado

[HUMO-MAC-2D.md](HUMO-MAC-2D.md) añade campos persistentes, advección, fuerzas y proyección de presión. El editor usa 128²; el emisor admite hasta 256². Fronteras cerradas, Jacobi amortiguado y residual visible bajo petición. Sigue pendiente la versión 3D, presión convergida por tolerancia, combustión e integración con el fuego procedural. La advección actual es difusiva y no conservativa: la conservación estricta de masa del plan anterior continúa como objetivo, no como propiedad demostrada.
