# Primera base de intersecciones GPU

Estado: **28 de octubre de 2026**. Entrega parcial de P3, aislada en
`telar-render::rayos`. No cambia el visor ni añade todavía un trazador de caminos.
La aplicación ya ofrece un ensayo por lotes con varios objetos, sombras y HDR:
ver [RENDER-POR-LOTES.md](RENDER-POR-LOTES.md).

## Contrato implementado

El emisor Rust produce operaciones SPIR-V que calculan en GPU la intersección
bilateral de un rayo con un triángulo. Usa Möller–Trumbore: dos aristas,
productos cruzados y escalares, inversión del determinante y prueba de
coordenadas baricéntricas. La implementación es propia, sobre la matemática
publicada en [Möller y Trumbore, 1997](https://my.eng.utah.edu/~cs6965/papers/MT97.pdf).

Entradas SSA: origen/dirección vec3, tres vértices vec3 y un intervalo finito
`0 <= tmin <= tmax`. Todos deben pertenecer a la función emisora y tener el
tipo declarado. No normaliza la dirección: en `O+tD`, t es distancia solamente
si D es unitario. Se aceptan ambas caras y extremos inclusivos del intervalo.

Salidas: booleano de impacto, t, u y v; pesos del triángulo `(1-u-v,u,v)`.
Si no hay impacto, los tres números de salida son cero. Se rechazan entradas
no finitas, intervalos inválidos y determinantes con valor absoluto <= 1e-7.
La división usa un divisor alternativo seguro cuando falla la prueba del
determinante. Los cálculos intermedios pueden desbordar con escalas extremas;
no se promete exactitud general por rechazar resultados no finitos.

La función emisora no conoce Vulkan, no reserva buffers y no lee datos en CPU.
Recibe valores SSA para poder emplear después geometría cargada desde VRAM.
El llamador debe validar sus accesos a buffers y sus dependencias de memoria.

## Evidencia y coste

`crates/telar-app/tests/rayos_gpu.rs` ejecuta el código en Vulkan real:

- 19 rayos en cuatro escenas: tres triángulos a distintas profundidades y
  posiciones, triángulo degenerado, empate y triángulo inclinado.
- Impactos por ambas caras, vértices/aristas sencillos, dirección no unitaria,
  rayo paralelo/nulo, segmento vacío/invertido, NaN e infinito.
- El test selecciona el impacto más cercano en GPU; en empate conserva el
  primer índice. Es una reducción lineal de prueba, no un BVH implementado.
- Compara con una referencia f64 independiente: intersección con plano y
  baricéntricas mediante matriz de Gram. Tolerancia absoluta 2e-5 en t/u/v,
  identidad exacta del triángulo. Los casos degenerados se rechazan.
- Valida SPIR-V con `spirv-val --target-env vulkan1.3`, grupos parcialmente
  activos, tres palabras sobrantes y buffers menores que un registro. Los bytes sin
  escritura no están inicializados: no se afirma integridad por compararlos con cero.

**IGUAL en esos casos:** NVIDIA GeForce RTX 3060 / Windows, 76 consultas,
cero avisos de validación TELAR. No demuestra exactitud en toda geometría,
igualdad entre fabricantes ni superioridad de velocidad.

CPU: emite el shader al preparar; en el test prepara constantes y comprueba la
descarga explícita. GPU: aritmética por cada par rayo–triángulo; no necesita
unidades de ray tracing. El ensayo incrusta geometría/rayos de prueba en el
programa; aún no prueba un almacén de mallas arbitrarias residente.
VRAM: el ensayo escribe 16 B por consulta, más 12 B de guarda; memoria física,
staging y alineación del backend son adicionales. La función por sí sola no
reserva VRAM. RAM/E/S: código SPIR-V, referencia y archivos temporales solo en
verificación. Tiempo GPU, ocupación y pico de memoria siguen **ABIERTOS**.

## Qué falta antes de producción

1. Robustez en aristas compartidas, escalas y errores de origen del rayo. Este
   prototipo usa un umbral absoluto; no garantiza una intersección *watertight*.
   Estudiar [Woop, Benthin y Wald](https://jcgt.org/published/0002/01/05/)
   y el [análisis de errores de PBRT](https://pbr-book.org/4ed/Shapes/Managing_Rounding_Error).
2. Mallas/instancias y materiales en buffers residentes, identidad de objetos
   y límites de acceso. Construcción/actualización de BVH y recorrido acotado
   con pruebas contra búsqueda exhaustiva.
3. Conectar sombras entre objetos al visor, incluyendo sesgo y rayos secundarios.
4. Acumulación HDR, contador de muestras e invalidación de cámara/escena/luz/material.
5. Transporte de luz, muestreo de luces y materiales, rebotes y control de ruido.

Más MSAA no sustituye estas piezas. Las peticiones de 32 MSAA siguen separadas
de las futuras 32/64/256 muestras por píxel de producción.
