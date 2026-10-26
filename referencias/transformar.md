# transformar · ABIERTO

Todavía no comparado en Houdini Apprentice.

Prueba propuesta: una caja por defecto con estos parámetros:
- traslación (10, 0, 0);
- rotación (0, 0, 90);
- escala (2, 1, 1).

| Cifra | TELAR (tests) | Houdini (esperado: SRT, rotación XYZ) | Comprobado |
|---|---|---|---|
| El punto (1, 0, 0) va a | (10, 2, 0) | (10, 2, 0) | pendiente |
| Con rotación (90, 90, 0), el punto (0, 1, 0) va a | (1, 0, 0) | (1, 0, 0) | pendiente |
| Caja escalada ×2 y subida 5 | de (−1, 4, −1) a (1, 6, 1) | la misma | pendiente |

## Cómo comprobarlo

En Apprentice, **Box → Transform** con Transform Order «Scale Rot Trans» y Rotate Order «Rx Ry Rz» (los valores por defecto). Comparar la caja (*bounds*) y algún punto en el Geometry Spreadsheet.
