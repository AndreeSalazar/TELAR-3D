# nudo

El núcleo de TELAR: **la geometría y el grafo que cocina solo lo que cambia**. En `no_std`: solo `core` y `alloc`, también para `sin`, `cos` y `sqrt`.

```rust
let mut grafo = Grafo::nuevo();
let caja = grafo.anadir(Caja::default(), &[])?;            // nodos de telar-nodos
let mover = grafo.anadir(Transformar::default(), &[caja])?;
grafo.cocinar(mover)?;                                       // cocina los dos
grafo.poner(mover, "traslacion", Valor::Vector([0.0, 5.0, 0.0]))?;
grafo.cocinar(mover)?;                                       // solo transformar: la caja no cambió
```

## La geometría (`geometria.rs`)

El modelo de Houdini, según su documentación:

| Pieza | Qué es |
|---|---|
| **puntos** | posiciones (`P`) |
| **vértices** | cada uno apunta a un punto; una esquina compartida es un punto con varios vértices |
| **primitivas** | cada una, un rango de vértices en orden (un polígono) |
| **atributos** | datos con nombre por punto, vértice, primitiva o para toda la geometría (*detail*): reales, vectores o enteros |

`Geometria::comprobar` verifica que es coherente: cada vértice apunta a un punto que existe, cada primitiva cae dentro de los vértices y cada atributo tiene un valor por elemento.

## El grafo (`grafo.rs`)

- **Sin ciclos por construcción:** un nodo solo puede tener como entradas nodos que ya existen.
- **Marcas de sucio:** cambiar un parámetro ensucia ese nodo y todo lo que depende de él. Lo demás conserva su resultado, y un valor igual al que ya tenía no ensucia nada.
- **Cocinar** recorre hacia atrás desde el nodo pedido y solo recalcula lo sucio.
- **Un contador** (`cocinados`) lo demuestra en los tests.
- **Un error** dice qué nodo falló («fuente (nodo 0): x negativo») y no rompe nada: arreglado el parámetro, todo vuelve a cocinar.

## La matemática propia (`mate.rs`)

En `core` no hay `sin`, `cos` ni `sqrt`, así que se escriben aquí, solo con +, −, × y ÷ (IEEE 754 las define exactas, así que el resultado es el mismo en cualquier máquina):

- **`sin`/`cos`:** el ángulo se reduce a [−π/4, π/4] y se suma la serie de Taylor en `f64`. Contra los de `std`, error menor que 2·10⁻⁷ en 400 000 ángulos entre −1000 y 1000.
- **`raiz`:** Newton en `f64` desde una aproximación sacada de los bits. Contra `std`, idéntica en casi todos los casos y nunca a más de una unidad del último bit.

## Las cifras (`estadisticas.rs`)

Puntos, vértices, primitivas, la caja y el promedio de `P` (sumado en `f64`): lo que se compara con Houdini en `referencias/`.

## Tests

```bash
cargo test -p nudo
```

La matemática contra `std`, y el grafo con un contador, que exige que se cocine exactamente lo que cambió.
