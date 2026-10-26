# esfera

Una esfera de **meridianos y paralelos**, como un globo terráqueo, con un punto en cada polo:
- triángulos alrededor de los polos;
- cuadriláteros en el resto.

| Parámetro | Por defecto | Qué hace |
|---|---|---|
| `radio` | `1` | el radio |
| `filas` | `13` | filas de puntos de polo a polo, polos incluidos (al menos 3) |
| `columnas` | `24` | puntos en cada paralelo (al menos 3) |
| `centro` | `[0, 0, 0]` | dónde queda su centro |

Con `filas = F` y `columnas = C`:

| | Cuántos | Con los valores por defecto |
|---|---|---|
| Puntos | `2 + (F − 2) · C` | 266 |
| Caras | `2C + (F − 3) · C` | 288 |
| Vértices | `6C + 4 · (F − 3) · C` | 1104 |

## La matemática

Coordenadas esféricas:
- θ es la latitud, de 0 en el polo norte a π en el sur, en `F − 1` pasos iguales;
- φ es la longitud, de 0 a 2π, en `C` pasos.

```
x = r · sin θ · cos φ        y = r · cos θ        z = r · sin θ · sin φ
```

`sin` y `cos` son los de `nudo::mate`: propios, en `no_std`, deterministas.

Las caras van en sentido antihorario visto desde fuera. Se dedujo con el producto vectorial (ver los comentarios del código) y lo comprueba el test, cara por cara: la normal de cada una apunta hacia fuera.

## Fuente

Las coordenadas esféricas son de cualquier libro de geometría. El nodo se inspira en el Sphere de Houdini con tipo «Polygon Mesh».

## Diferencias con Houdini

**ABIERTO.** No sé con seguridad los valores por defecto ni la disposición exacta de los polos del Sphere de Houdini. Hay que mirarlo en Apprentice y anotarlo en `referencias/esfera.md`.
