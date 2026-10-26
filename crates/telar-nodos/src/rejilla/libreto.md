# rejilla

Un plano de **filas × columnas** puntos en el suelo (plano ZX), hecho de cuadriláteros: **(filas − 1) × (columnas − 1)** caras. La normal de cada cara mira hacia arriba (+Y).

| Parámetro | Por defecto | Qué hace |
|---|---|---|
| `tamano` | `[10, 10, 0]` | el ancho en X y el fondo en Z |
| `filas` | `10` | puntos a lo largo de Z (al menos 2) |
| `columnas` | `10` | puntos a lo largo de X (al menos 2) |
| `centro` | `[0, 0, 0]` | dónde queda su centro |

Con los valores por defecto: **100 puntos, 81 caras, 324 vértices**, de −5 a 5 en X y en Z.

## Cómo se construye

- El punto de la fila `f` y la columna `c` está en `x = −ancho/2 + c · ancho/(columnas − 1)` y `z = −fondo/2 + f · fondo/(filas − 1)`.
- Su número es `f · columnas + c`.
- Cada cara une `(f, c)`, `(f+1, c)`, `(f+1, c+1)` y `(f, c+1)`: antihorario visto desde arriba.

## Fuente

El comportamiento, del nodo Grid de Houdini (plano ZX, tipo «Polygon»), según su documentación: los mismos parámetros y los mismos valores por defecto.

## Diferencias con Houdini

El orden de los vértices de cada cara, como en `caja`. Pendiente de comprobar en Apprentice (`referencias/rejilla.md`).
