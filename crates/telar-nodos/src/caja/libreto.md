# caja

Un cubo alineado con los ejes: **8 puntos, 6 caras cuadradas, 24 vértices**.

| Parámetro | Por defecto | Qué hace |
|---|---|---|
| `tamano` | `[1, 1, 1]` | el lado en X, Y y Z |
| `centro` | `[0, 0, 0]` | dónde queda su centro |

## Cómo se construye

- **Los puntos:** cada esquina se numera con un bit por eje, `x + 2y + 4z` (0 = lado negativo). Su posición es `centro + (±½) · tamaño` en cada eje.
- **Las caras:** cada una lista sus 4 esquinas en sentido **antihorario visto desde fuera**. Así su normal, por la regla de la mano derecha, `(b − a) × (c − a)`, mira hacia fuera. Es el convenio de Vulkan, donde se dibujará.
- **Los vértices:** cada esquina está en 3 caras, así que hay 3 vértices por punto (24 en total).

## Fuente

El comportamiento, del nodo Box de Houdini (tipo de primitiva «Polygon»), según su documentación. La matemática es la de cualquier cubo; no hay más.

## Diferencias con Houdini

- **El orden de los vértices:** Houdini recorre sus polígonos en el sentido contrario (horario visto desde fuera). TELAR usa el de Vulkan. Pendiente de comprobar en Apprentice (`referencias/caja.md`).
- **El orden de los puntos** puede ser otro: no cambia ni las caras ni las cifras.
