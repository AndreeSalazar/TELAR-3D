# telar-nodos

La biblioteca de nodos de TELAR. **Una carpeta por nodo**, y cada una se explica y se demuestra sola (regla `nodo-completo`):

```
src/<nodo>/mod.rs        el código
src/<nodo>/libreto.md    qué hace, la matemática y de dónde sale
tests/<nodo>.rs          su test
referencias/<nodo>.md    su comparación con Houdini, con su etiqueta
```

| Nodo | Qué hace | Por defecto |
|---|---|---|
| [`caja`](src/caja/libreto.md) | un cubo alineado con los ejes | 8 puntos, 6 caras, 24 vértices |
| [`rejilla`](src/rejilla/libreto.md) | un plano de filas × columnas en el suelo (ZX) | 100 puntos, 81 caras |
| [`esfera`](src/esfera/libreto.md) | meridianos y paralelos, un punto en cada polo | 266 puntos, 288 caras |
| [`transformar`](src/transformar/libreto.md) | escala, gira y traslada (SRT, rotación XYZ) | sin cambios |
| [`unir`](src/unir/libreto.md) | junta sus entradas en una geometría | — |

Es del núcleo (`no_std`): solo usa `nudo` (regla `fronteras`), `core` y `alloc`.

## Lo que comprueban los tests

- **Cifras:** recuentos, caja y centro de cada nodo; las fórmulas de la esfera para varios tamaños.
- **Caras hacia fuera:** en la caja, la esfera y la caja transformada, la normal de cada cara (`(b − a) × (c − a)`) apunta hacia fuera. En la rejilla, todas hacia arriba.
- **La esfera es una esfera:** cada punto está a exactamente el radio del centro.
- **Transformar:** el orden SRT y la rotación XYZ, con puntos conocidos.
- **Unir:** los recuentos se suman, los índices se desplazan y los atributos que faltan se rellenan con ceros.
- **El grafo, con estos nodos (`tests/grafo.rs`):** mover la caja recocina solo `transformar` y `unir`; dar más detalle a la esfera, solo `esfera` y `unir`.

## Contra Houdini

Todas las referencias están en **ABIERTO**: todavía no se han comparado en Apprentice, y cada una explica cómo hacerlo. Las cifras que se esperan de la caja y la rejilla salen de los valores por defecto de la documentación de Houdini; las de la esfera, ni eso, y se dice.
