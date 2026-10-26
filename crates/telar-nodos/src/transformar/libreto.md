# transformar

Escala, gira y traslada los puntos de su entrada (**una**). Las caras y los atributos no cambian.

| Parámetro | Por defecto | Qué hace |
|---|---|---|
| `traslacion` | `[0, 0, 0]` | cuánto se mueve |
| `rotacion` | `[0, 0, 0]` | grados alrededor de X, Y y Z |
| `escala` | `[1, 1, 1]` | cuánto se estira en cada eje |
| `escala_uniforme` | `1` | una escala más, igual en los tres ejes |

## La matemática

En este orden (el **SRT** de Houdini, con la rotación en orden **XYZ**):

1. **Escala:** `p · escala · escala_uniforme`, componente a componente.
2. **Rotación** alrededor de X, luego de Y, luego de Z, cada una con la regla de la mano derecha:
   ```
   X: (x, y·cos − z·sin, y·sin + z·cos)
   Y: (x·cos + z·sin, y, −x·sin + z·cos)
   Z: (x·cos − y·sin, x·sin + y·cos, z)
   ```
3. **Traslación:** `+ traslacion`.

`sin` y `cos` son los propios de `nudo::mate`.

## Fuente

El orden SRT y XYZ es el que el nodo Transform de Houdini usa por defecto, según su documentación. Las matrices de rotación son las de cualquier libro de álgebra lineal.

## Diferencias con Houdini

- Todavía no tiene pivote ni la opción de cambiar el orden.
- Si la geometría tuviera normales (`N`), habría que girarlas también. Hoy ningún nodo las crea; cuando lleguen, transformar las girará (y quedará en este libreto).
