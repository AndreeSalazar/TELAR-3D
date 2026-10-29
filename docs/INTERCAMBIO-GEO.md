# Intercambio de muestras puntuales: primera aplicación del estudio de Partio

TELAR dispone de `telar-intercambio`, un crate propio **no_std + alloc**, sin
dependencias externas. Define muestras tipadas y un lector/escritor de un
subconjunto textual **PGEOMETRY V5**. La app exporta campos físicos del laboratorio
3D como puntos y permite inspeccionarlos sin crear un contexto Vulkan.

La implementación es propia a partir del
[formato clásico documentado por SideFX](https://www.sidefx.com/docs/houdini/io/formats/geo.html).
Partio motivó el estudio de atributos/intercambio; no se ha instalado, enlazado ni
copiado su código. Las fuentes y sus límites están en
[el registro de investigación](../referencias/partio-houdini-intercambio.md).

## Uso

```powershell
cargo build -p telar-app --release
.\target\release\telar.exe --explosion --pasos 20 --exportar-geo target/pulso.geo
.\target\release\telar.exe --inspeccionar-geo target/pulso.geo
.\target\release\telar.exe --inspeccionar-geo docs/ejemplos/muestras-telar.geo
```

`--exportar-geo` requiere `--humo-3d`, `--explosion` o `--particulas`. Ejecuta oculto hasta el
número de cuadros solicitado (240 por defecto), exporta y termina. `--segundos`
puede terminar antes; `--prueba` conserva su límite de 120 cuadros. La inspección
es un comando independiente; no importa la nube a un grafo ni reproduce simulación.

En el laboratorio visible, **F8** exporta junto al proyecto activo: `explosion.geo`,
`humo3d.geo` o el nombre de la escena abierta con extensión `.geo`. La lectura
espera al trabajo GPU enviado; mientras exporta, el hilo de interfaz no envía
nuevos cuadros. Al terminar continúa la animación previa. No es E/S asíncrona.

El destino debe ser nuevo: no sobrescribe archivos. No crea carpetas padre.
Un error de escritura puede dejar un archivo incompleto recién creado; el lector
lo rechaza y no se anuncia exportación correcta. No es todavía salida transaccional
de producción. El comando no acepta `.bgeo`, `.bgeo.sc` ni compresión.

## Datos que se intercambian

| Campo | Representación | Significado en la exportación MAC |
|---|---|---|
| P | Tres f32 y peso fijo 1 | Centro de celda, dominio unitario, Y arriba |
| v | Tupla de tres float | Promedio de caras MAC opuestas; unidades de dominio/s |
| density | Float escalar | Densidad normalizada, no negativa |
| heat | Float escalar | Calor normalizado, no kelvin |
| id | Int32 no negativo | Índice de celda `x + N*y + N²*z`, único |
| sim_time | Float escalar | Reloj simulado GPU en segundos, repetido por muestra |

P es la posición implícita del formato; los otros cinco son atributos de punto.
Se exportan **todas las celdas**, también las de densidad cero, en orden X/Y/Z.
No se fabrican nacimientos, edad o vida de partículas. La velocidad de centro es
`0.5*(cara_positiva + cara_negativa)`; la cara exterior negativa vale cero en el
dominio cerrado. Los escalares se copian sin remuestreo y no se aplica la cámara.

El archivo contiene cero primitivas y cero grupos, diccionario fijo de atributos
y bloque extra vacío. Los números f32 se escriben en notación científica con
nueve cifras significativas y decimal con punto. El lector conserva sus valores,
orden e identificadores; no admite esquemas arbitrarios, atributos adicionales,
mallas, pesos distintos de 1 ni todas las variantes de GEO.

No se exportan las caras originales, presión, residual, estado del solver,
transformaciones arbitrarias ni metadatos suficientes para reanudar el volumen.
Por tanto, es **una nube de muestras de campos**, no una caché MAC ni partículas
simuladas. Los `.tlar` continúan guardando configuración por separado.

## Límites y coste

Máximo **65 536 puntos** y **16 MiB de texto**. Se validan finitud, magnitud
absoluta <=1e20, escalares no negativos e id <=2147483647; los ids deben ser únicos.
La cabecera se valida antes de reservar por recuento; tokens de hasta 64 bytes.
Se rechazan esquemas no soportados, datos truncados y contenido final inesperado.

El backend y el kernel no dependen del crate de intercambio. La app solicita
explícitamente 64 B de control y los primeros cinco planos físicos, comprueba
estado no inválido y convierte a puntos. Con presión en límite de presupuesto
se permite exportar la aproximación existente; el archivo no certifica convergencia.
Durante esas dos lecturas no se envía otro cuadro desde esa ventana.

A 32³: **655 424 B descargados** (64 + 20*N³), vector de puntos de
**1 310 720 B** (40*N³), vector de ids auxiliar durante validación y texto ASCII.
Hay staging, alineación y capacidad de asignaciones adicionales; 16 MiB limita
longitud serializada, no pico físico total. La validación de ids ordena una copia,
con coste O(n log n). La conversión MAC y el formato consumen CPU únicamente
cuando se solicita exportar. La simulación y el dibujo siguen en GPU.

La operación no añade lecturas en cuadros estables ni buffers permanentes GPU.
No se han medido FPS, latencia de exportación o pico físico de RAM/VRAM.

## Evidencia del 28 de octubre de 2026

- El crate prueba una [nube mínima propia](ejemplos/muestras-telar.geo), precisión
  f32 (incluidos subnormales/cero con signo), orden, repetibilidad y nube vacía.
- Rechaza variantes, cambios de tipos/aridad, ids duplicados, NaN/inf,
  truncamientos, archivos/tokens/recuentos fuera de límites y datos sobrantes.
- La conversión MAC contrasta velocidades de centro y bordes con caras fabricadas
  de valores conocidos; no las confunde con la velocidad de una sola cara.
- La prueba Vulkan a 9³ exporta un pulso real, vuelve a leerlo, compara todos los
  atributos y comprueba campos GPU idénticos antes/después. Intentar reemplazar
  el archivo falla sin alterar sus bytes.

Ejecución real RTX 3060 / Windows 11: `--explosion --pasos 20` produjo
32 768 puntos y **4 549 260 B** de GEO. La inspección mostró posiciones entre
0.015625 y 0.984375 por eje, densidad máxima 0.75175583, calor máximo 3.7303145
y tiempo 0.3199999 s. Cero avisos de validación Vulkan del producto al exportar.
El inspector se ejecutó sin iniciar Vulkan.
La prueba nativa de humo 3D con 120 cuadros, dos redimensionados y exportación
también terminó sin avisos; su archivo se volvió a inspeccionar correctamente.

**Comparación Houdini/Partio: ABIERTO.** Las pruebas propias no sustituyen abrir
el archivo con un lector externo. No se ha ejecutado Houdini ni Partio para
certificar interoperabilidad o rendimiento. El siguiente ensayo externo debe
registrar aplicación, versión/revisión, tipo y valores de atributos y tolerancias.

Siguiente ampliación: atributos generales y conversores
externos comprobados; cachés reanudables de campos requieren un contrato distinto.

## Segundo perfil fijo: partículas vivas

`--particulas` y F8 exportan partículas simuladas, con los cinco atributos base
más `age`, `life`, `birth_time` (float escalares, segundos; defaults 0, 1, 0).
No altera el perfil MAC anterior: cada lector de núcleo exige su propio esquema;
el inspector host distingue ambos. Puntos con vida>0, 0<=age<life y
0<=birth_time<=sim_time; id único = generación*128+plaza. density=1 y
heat=clamp(1-age/life), máscara visual sin unidades térmicas.

[Ejemplo propio](ejemplos/particulas-telar.geo), `Particula` de 52 B y APIs
`codificar_particulas_geo`/`decodificar_particulas_geo`. Comparten límites de
65536 puntos/16 MiB; validación añade copia temporal de puntos e ids.
Solo se exportan vivas: no constituye caché reanudable. Coste concreto del
laboratorio y evidencia GPU en [PARTICULAS-GPU.md](PARTICULAS-GPU.md).
Compatibilidad con lectores Houdini/Partio sigue **ABIERTO**.
