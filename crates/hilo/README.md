# HILO: IR tipado y primer compilador por punto

`no_std + alloc`, sin crates externos, sin Vulkan y sin FFI. Usa `hilo-spirv`
para emitir palabras SPIR-V y `nudo::mate::raiz` para la referencia numérica CPU.
`telar-gpu` continúa aislado: recibe el código compilado desde `telar-app`.

Esta entrega incluye un IR lineal en SSA, validación, intérprete CPU y compilación
a SPIR-V. Todavía no hay analizador de texto HILO, control de flujo, bucles,
atómicos, accesos a vecinos, atributos arbitrarios ni fusión de operaciones.

```rust
let programa = hilo::ejemplos::relieve(1.3)?;
let spirv = programa.compilar(); // compilar una vez al preparar/editar
// telar-app conecta estos bytes a una etapa Puntos del plan residente.
# Ok::<(), hilo::Error>(())
```

## Modelo y contratos

Cada `Operacion` define un valor de tipo `Escalar` (f32) o `Vector` (vec3).
Los operandos `Id` solo pueden apuntar a instrucciones anteriores. El programa
es inmutable tras validarse; `Error` identifica la instrucción y el motivo.
Límite actual: 256 instrucciones por programa.

| Operaciones | Semántica |
|---|---|
| Escalar, Vector | constantes finitas |
| Posicion, Normal | entrada del punto actual al empezar la etapa |
| Fase, Amplitud | parámetros leídos desde el buffer vivo |
| Sumar, Restar | operandos del mismo tipo |
| Multiplicar | escalar×escalar, vector×escalar en ambos órdenes, o componente a componente |
| Extraer, Componer | componentes 0..=2 y composición de tres escalares |
| ProductoEscalar | vec3·vec3 → f32 |
| Normalizar | normalización con escala previa; el vector nulo produce [0,1,0] |

Las salidas son altura escalar y normal vectorial. El backend conserva posición
x/z, índices y control. Los accesos a datos los genera la envoltura, no el IR:
cada hilo comprueba su índice y solo escribe altura/normal de su propio punto.
No hay instrucciones que permitan carreras entre puntos, barreras divergentes
o sumas atómicas. Estas construcciones quedan fuera del IR inicial; aún no se
analizan programas arbitrarios que las contengan.

El kernel usa grupos de 64 y el ABI de cuatro buffers de la rejilla residente.
La CPU prepara el programa; no interpreta los puntos del visor en cada fotograma.
Cambiar fase/amplitud conserva el shader. Cambiar una constante del programa
requiere recompilar y preparar el plan.

## Semántica numérica y comprobación

El intérprete rechaza entradas o resultados no finitos. El comprobador de tipos
no demuestra que una cadena de operaciones no desborde f32. El kernel aún no
incluye diagnóstico numérico en vivo: la equivalencia se verifica para entradas
y resultados finitos, con tolerancia 2·10⁻⁵. No se promete identidad flotante
entre CPU y GPU ni entre dispositivos. Las pruebas exigen repetición bit a bit
en la misma GPU/configuración y emisión idéntica del compilador.

La normalización escala primero por la componente de mayor magnitud, evitando
elevar componentes enormes al cuadrado. Para el vector nulo se elige arriba
antes de normalizar. Los subnormales y modos de redondeo entre dispositivos
quedan fuera de la equivalencia verificada.

`ejemplos::relieve(k)` expresa `y' = k*y` y
`N' = normalize(k*Nx, Ny, k*Nz)` para una superficie de altura. Es la transformación
de la normal según las derivadas de esa superficie. No es una transformación
general de mallas ni demuestra que un programa arbitrario produzca una normal
geométricamente correcta.

| Archivo | Responsabilidad |
|---|---|
| `src/ir.rs` | operaciones, tipos, referencias SSA y errores |
| `src/interprete.rs` | evaluación CPU y semántica numérica |
| `src/spirv.rs` | traducción de operaciones a SPIR-V |
| `src/spirv/interfaz.rs` | tipos y buffers del ABI por punto |
| `src/ejemplos.rs` | programa de relieve escrito con el IR |

`tests/ir.rs` prueba tipos y matemática contra resultados independientes.
`tests/spirv.rs` exige el validador oficial de Khronos para el código generado.
`telar-app/tests/hilo_gpu.rs` compara seis programas CPU/GPU, parámetros en vivo,
vacío, grupos parciales, exceso de capacidad, caché y repetición de resultados.
El visor de un millón de puntos usa el relieve compilado desde HILO.

Etiqueta: **IGUAL** en los casos verificados dentro de tolerancia en Ryzen 5
5600X / RTX 3060 / Windows 11. Rendimiento comparativo y Houdini: **ABIERTO**.
Las bases de SSA y las fuentes SPIR-V están en `docs/MATEMATICAS.md`, sección 12.
