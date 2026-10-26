# caja · ABIERTO

Todavía no comparado en Houdini Apprentice.

| Cifra (valores por defecto) | TELAR (tests) | Houdini (según su documentación) | Comprobado |
|---|---|---|---|
| Puntos | 8 | 8 | pendiente |
| Primitivas | 6 | 6 | pendiente |
| Vértices | 24 | 24 | pendiente |
| Caja | de (−0,5, −0,5, −0,5) a (0,5, 0,5, 0,5) | la misma (tamaño 1, centrada) | pendiente |
| Centro (promedio de P) | (0, 0, 0) | (0, 0, 0) | pendiente |

## Cómo comprobarlo

En Apprentice, un nodo **Box** con sus valores por defecto y **Primitive Type: Polygon**. Mirar la información del nodo, anotar las cifras aquí y cambiar la etiqueta.

## Diferencia que hay que confirmar

El sentido de recorrido de los vértices de cada cara. TELAR usa el antihorario visto desde fuera, el de Vulkan; creo que Houdini usa el contrario. Si es así, la etiqueta será **DISTINTO** en eso, con el motivo, y lo demás **IGUAL**.
