# esfera · ABIERTO

Todavía no comparado, y aquí hay más dudas que en los demás: **no sé con seguridad** los valores por defecto del Sphere de Houdini con tipo «Polygon Mesh», ni cómo resuelve los polos.

| Cifra | TELAR (tests) | Houdini | Comprobado |
|---|---|---|---|
| Puntos | `2 + (F − 2) · C` (266 con 13 × 24) | por mirar | pendiente |
| Primitivas | `2C + (F − 3) · C` (288) | por mirar | pendiente |
| Vértices | `6C + 4 · (F − 3) · C` (1104) | por mirar | pendiente |
| Caja (radio 1) | de (−1, −1, −1) a (1, 1, 1) | (−1, −1, −1) a (1, 1, 1) | pendiente |
| Distancia de cada punto al centro | exactamente el radio (±1e-5) | el radio | pendiente |

## Cómo comprobarlo

1. En Apprentice, un **Sphere** con **Primitive Type: Polygon Mesh**.
2. Anotar sus filas y columnas por defecto.
3. Poner las mismas en TELAR y comparar puntos, primitivas y vértices.
4. Si los polos se resuelven distinto (por ejemplo, un punto por columna en el polo en vez de uno solo), la etiqueta será **DISTINTO**, con ese motivo.
