# unir

Junta sus entradas (tantas como tenga, en orden) en una sola geometría.

No tiene parámetros.

## Cómo se junta

Al añadir una geometría B detrás de A (con `nA` puntos y `vA` vértices):

- **Los puntos de B** van detrás de los de A.
- **Los vértices de B** apuntan a `punto + nA`, porque sus puntos se desplazaron.
- **Las primitivas de B** van detrás, con sus vértices desplazados `vA`.
- **Los atributos:**
  - si los tienen las dos, se juntan, y tienen que ser del mismo tipo;
  - si solo lo tiene una, en la otra se rellena con **ceros** (como hace Houdini con los valores por defecto);
  - los de toda la geometría (*detail*) se quedan con el valor de la primera entrada que lo tenga.

Al final se comprueba que el resultado es coherente (`Geometria::comprobar`).

## Fuente

El comportamiento, del nodo Merge de Houdini, según su documentación.

## Las cifras

Los recuentos se suman: puntos, vértices y primitivas de la unión = la suma de los de cada entrada. Lo exige el test.
