# hilo-spirv

**SPIR-V emitido a mano**, en `no_std`: el «ensamblador» de TELAR antes de tener el lenguaje HILO, como TALK-CPU antes de los compiladores. Una API en Rust que escribe las instrucciones palabra a palabra: sin glslc, sin shaderc, sin crates.

```rust
let mut m = Modulo::nuevo();                  // Capability Shader, Logical GLSL450
let u32_ = m.tipo_u32();                      // los tipos y constantes se declaran una sola vez
let mut f = m.funcion(void, tipo_main);
let i = f.cargar(u32_, puntero);
let h = f.binaria(op::BITWISE_XOR, u32_, a, b);
let main = f.terminar();
m.entrada_calculo(main, "main", &[id_global], [64, 1, 1]);
let palabras: Vec<u32> = m.ensamblar();       // listo para vkCreateShaderModule
```

Es del núcleo (regla `nucleo-sin-std`): solo `core` y `alloc`, así que el mismo código correrá algún día dentro de BMO-X.

## Cómo está hecho

| Archivo | Qué |
|---|---|
| `src/op.rs` | códigos de operación, capacidades, clases y decoraciones, **comprobados uno a uno contra `spirv.h`** del SDK de Vulkan 1.4.350 |
| `src/modulo.rs` | el constructor: secciones en el orden de la especificación (2.4), tipos y constantes sin repetir, entradas y modos, `GLSL.std.450` importado una vez |
| `src/funcion.rs` | el cuerpo de una función: bloques, cargas, aritmética, vectores (`construir`, `extraer`, `barajar`) e instrucciones de `GLSL.std.450` |
| `src/ejemplos.rs` | el primer kernel: llena un buffer con `hash_pcg(i)`, con su referencia en la CPU |
| `src/visor.rs` | los shaders del visor (Fase 4): vértices y fragmentos |
| `src/rejilla.rs` | productor de argumentos, generación y deformación de la rejilla residente |

## Los kernels residentes (Fase 4.5)

`rejilla::preparar`, `generar` y `deformar` emiten tres programas independientes.
El productor tiene un hilo; los otros usan grupos de 64 y comprueban el índice
contra el recuento activo. Cada hilo posee su vértice y, al generar, una celda de
índices como máximo. La deformación es `y = A sin(2x + fase) cos(2z)`; su normal
es la normalización de `(-2A cos(2x+fase) cos(2z), 1, 2A sin(2x+fase) sin(2z))`.
Es diferenciación analítica de esa superficie, sin reducción ni atómicos.

ABI del conjunto 0, todos buffers de almacenamiento:

| Enlace | Contenido |
|---|---|
| 0 | cuatro u32: lado pedido, bits f32 de fase y amplitud, capacidad validada por el ejecutor |
| 1 | f32[], posición xyz y normal xyz por vértice |
| 2 | u32[], seis índices por celda, caras hacia +Y |
| 3 | u32[16]: dispatch de deformación [0..3), draw [3..8), lado seguro [8], puntos [9], error [10], último lado generado [11], dispatch de generación [12..15), contador de generaciones [15] |

Un lado menor que 2 da salida vacía. Un lado mayor que capacidad da salida
vacía y error 1. El último lado válido permanece en GPU: si se pide otra vez,
el productor escribe cero grupos de generación y reutiliza posiciones x/z e
índices. Fase y amplitud siguen cambiando la deformación. Vacío y error no
invalidan la geometría guardada. El contador entero de generaciones avanza solo
cuando se genera; su desbordamiento a 32 bits es modular y no afecta la caché.

El control se inicializa a cero una vez con `vkCmdFillBuffer` en el ejecutor.
Las barreras entre kernels y antes del dibujo las graba el
ejecutor Vulkan. Los tres programas pasan `spirv-val`; las pruebas de integración
en `telar-app` comparan datos y píxeles. Esto aún no incorpora el lenguaje HILO.

## Los shaders del visor

```glsl
// vértices: 128 bytes de constantes (la matriz por columnas, la luz y tres colores)
gl_Position = c0*posicion.x + c1*posicion.y + c2*posicion.z + c3;   n = normal;
// fragmentos
vec3 N = normalize(n);
vec3 ambiente = mix(suelo, cielo, N.y*0.5 + 0.5);
color = vec4(base * (ambiente + max(dot(N, luz), 0)*0.85), 1);
```

Los dos pasan `spirv-val`. El contrato con `telar-gpu`: vértice = posición (ubicación 0) + normal (ubicación 1); la constante `TAMANO_CONSTANTES` es la misma en los dos lados.

## El primer kernel

```glsl
void main() {
    uint i = gl_GlobalInvocationID.x;
    if (i < datos.length()) datos[i] = hash_pcg(i);   // nunca fuera del buffer
}
```

- **230 palabras de SPIR-V 1.3**, escritas por el constructor. `cargo run -p hilo-spirv --example volcar -- hash.spv` lo vuelca a un archivo, para verlo con `spirv-dis`.
- **El hash** es el PCG de Jarzynski y Olano (JCGT, 2020): solo enteros de 32 bits, así que la GPU y la CPU dan los mismos bits.
- **En la RTX 3060** (`telar --calculo`), 16 777 216 números:
  - 0,196 ms en la GPU (solo el kernel);
  - 16,9 ms en la CPU (un hilo);
  - **iguales bit a bit**.

  Son unos 342 GB/s escritos, cerca de los 360 GB/s de la memoria de esa tarjeta: el kernel está limitado por la memoria, no por las cuentas.

## Verificado por otros

- **`spirv-val`** (el validador oficial de Khronos, del SDK) aprueba el kernel y los dos shaders del visor con `--target-env vulkan1.3`. En su primera ejecución cazó un fallo: `OpConstant` lleva primero el tipo y después el resultado, al revés que los tipos.
- **`hash_pcg`** se compara con una implementación independiente en JavaScript (`Math.imul`): 0 → 129 708 002, 1 → 2 831 084 092…
- **La GPU** se compara con la CPU bit a bit, en el test de `telar-app`.

```bash
cargo test -p hilo-spirv   # codificación, hash, y spirv-val si el SDK está instalado
```
