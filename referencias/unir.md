# unir · ABIERTO

Todavía no comparado en Houdini Apprentice.

| Cifra | TELAR (tests) | Houdini (Merge, según su documentación) | Comprobado |
|---|---|---|---|
| Recuentos | la suma de los de cada entrada | la suma | pendiente |
| Caja (8 puntos) + esfera (266) | 274 puntos, 294 primitivas | 274 y 294 | pendiente |
| Atributo que solo tiene una entrada | ceros en la otra | el valor por defecto del atributo (0) | pendiente |

## Cómo comprobarlo

En Apprentice:
1. **Box** y **Sphere** (con las filas y columnas de `esfera.md`) unidos en un **Merge**; comparar los recuentos.
2. Un **Attribute Create** en una sola de las entradas; mirar en el Geometry Spreadsheet que la otra queda a 0.
