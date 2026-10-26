# La matemática necesaria y de dónde sale

Todo lo que hace falta para TELAR es **público**: artículos, libros y especificaciones. Cada nodo cita en su `libreto.md` la fuente exacta de su matemática. Se marca **★** lo imprescindible para la esfera de fuego.

## 1. Álgebra y transformaciones (P0)

- Vectores, matrices 4×4 homogéneas, producto escalar y vectorial: la base de todo.
- **Cuaterniones** para rotar sin bloqueo de ejes: Shoemake, *Animating Rotation with Quaternion Curves* (SIGGRAPH 1985).
- Espacios de coordenadas: objeto → mundo → cámara → recorte. Proyección en perspectiva con profundidad invertida (más precisión lejos).

## 2. Geometría (P0)

- Puntos, vértices, primitivas y atributos: el modelo de Houdini, según su documentación.
- Normales por cara y por punto (promedio ponderado por ángulo).
- **Marching Cubes**, para convertir volúmenes en malla: Lorensen y Cline (SIGGRAPH 1987).
- Estructura *half-edge* para editar mallas.

## 3. Azar determinista (P0) ★

- La GPU no tiene `rand()`: se usan **funciones hash** puras, el mismo número para la misma semilla, en cualquier hilo y en cualquier orden.
  - Jarzynski y Olano, *Hash Functions for GPU Rendering* (JCGT, 2020): compara PCG, xxHash y otros, en calidad y velocidad.
  - O'Neill, *PCG: A Family of Simple Fast Space-Efficient Statistically Good Algorithms for Random Number Generation* (2014).

## 4. Ruido (P1–P2) ★

- **Perlin:** *An Image Synthesizer* (SIGGRAPH 1985) e *Improving Noise* (SIGGRAPH 2002).
- **Simplex:** Gustavson, *Simplex Noise Demystified* (2005).
- **fBm:** suma de octavas, `Σ aⁱ · ruido(2ⁱ·x)`, para el detalle fractal de las llamas.
- **Curl noise:** Bridson, Hourihan y Nordenstam, *Curl-Noise for Procedural Fluid Flow* (SIGGRAPH 2007). Da turbulencia sin divergencia: el fuego se riza sin «crear» ni «perder» humo.

## 5. Fluidos: humo y fuego (P1) ★

Ecuaciones de Navier–Stokes para un fluido incompresible:

```
∂u/∂t + (u·∇)u = −∇p/ρ + f        (momento)
∇·u = 0                            (incompresible)
```

- **Rejilla MAC** (velocidades en las caras de cada celda): Harlow y Welch (1965). El estándar para que la presión sea estable.
- **Advección semilagrangiana** (seguir hacia atrás cada celda): Stam, *Stable Fluids* (SIGGRAPH 1999). Incondicionalmente estable.
- **MacCormack / BFECC**, para no difuminar el detalle: Selle et al., *An Unconditionally Stable MacCormack Method* (2008).
- **Flotación y confinamiento de vorticidad**, los remolinos que se pierden en la rejilla: Fedkiw, Stam y Jensen, *Visual Simulation of Smoke* (SIGGRAPH 2001).
  - Flotación: `f = (−α·ρ_humo + β·(T − T_amb)) · ŷ`.
- **Proyección de presión:** resolver `∇²p = ∇·u / Δt` y restar `∇p`. En la GPU: Jacobi o *red-black* Gauss-Seidel, y **multigrid** después (Briggs, *A Multigrid Tutorial*).
- **El libro de referencia:** Bridson, *Fluid Simulation for Computer Graphics* (2.ª ed., 2015).

## 6. Combustión (P1) ★

- Nguyen, Fedkiw y Jensen, *Physically Based Modeling and Animation of Fire* (SIGGRAPH 2002).
- Modelo práctico (como el de los solvers de producción):
  - el **combustible** arde si la temperatura supera la de ignición;
  - al arder, el combustible baja, y la **temperatura**, el **hollín** y la **divergencia** suben (el gas se expande y empuja hacia fuera: la «bola» de la explosión);
  - **enfriamiento** radiativo, proporcional a `T⁴` (ley de Stefan–Boltzmann).

## 7. Render volumétrico (P1) ★

- **Transporte radiativo:** la luz que llega por un rayo se absorbe, se dispersa y se emite.
- **Beer–Lambert:** transmitancia `T = exp(−∫ σ_t ds)`.
- **Función de fase de Henyey–Greenstein** (1941), para que el humo disperse la luz hacia delante o hacia atrás.
- **Marcha de rayos** con sombras (una segunda marcha hacia la luz), y *jitter* por píxel para evitar bandas.
- **Fuentes:**
  - Fong, Wrenninge, Kulla y Habel, *Production Volume Rendering* (curso de SIGGRAPH 2017);
  - Novák et al., *Monte Carlo Methods for Volumetric Light Transport Simulation* (Eurographics 2018);
  - Pharr, Jakob y Humphreys, *Physically Based Rendering* (4.ª ed., gratis en pbr-book.org).

## 8. El color del fuego (P1) ★

- **Ley de Planck:** cuánta luz emite un cuerpo negro a temperatura `T` en cada longitud de onda. Así el fuego es rojo a 1000 K y amarillo blanquecino a 2000 K **porque sí, no porque lo pintemos**.
- **Funciones de igualación del color CIE 1931**, de espectro a XYZ: aproximación analítica de Wyman, Sloan y Shirley, *Simple Analytic Approximations to the CIE XYZ Color Matching Functions* (JCGT, 2013).
- XYZ → sRGB lineal (matriz estándar), y después **exposición y tonemapping**: Reinhard (2002) o una aproximación ACES. Así el núcleo blanco no satura y las llamas conservan el color.

## 9. Partículas y restricciones (P2–P3)

- **PBD:** Müller et al., *Position Based Dynamics* (2007).
- **XPBD:** Macklin, Müller y Chentanez, *XPBD: Position-Based Simulation of Compliant Constrained Dynamics* (2016). La base de un Vellum propio.
- **Fluidos con partículas:** Macklin y Müller, *Position Based Fluids* (2013).
- **FLIP:** Zhu y Bridson, *Animating Sand as a Fluid* (SIGGRAPH 2005).
- **Poisson disk:** Bridson, *Fast Poisson Disk Sampling in Arbitrary Dimensions* (SIGGRAPH 2007).

## 10. Volúmenes dispersos (P1–P2)

- Museth, *VDB: High-Resolution Sparse Volumes with Dynamic Topology* (ACM TOG, 2013).
- Museth, *NanoVDB* (2021): la versión para la GPU.
- Se implementa la idea (árbol de ladrillos), no su código.

## 11. Algoritmos paralelos en la GPU (P0) ★

- **Suma prefija (scan):** Blelloch (1990); Harris, Sengupta y Owens, *GPU Gems 3*, cap. 39. Sirve para compactar ladrillos activos y para contar de forma determinista.
- **Ordenación radix:** Merrill y Grimshaw, *High Performance and Scalable Radix Sorting* (2011).
- **Reducciones en árbol con orden fijo:** el mismo resultado bit a bit en cada ejecución.

## 12. El compilador de HILO (P0)

- **SSA:** Cytron et al., *Efficiently Computing Static Single Assignment Form* (TOPLAS, 1991).
- Cooper y Torczon, *Engineering a Compiler*.
- **Especificaciones de Khronos:** SPIR-V (el formato binario que emitimos) y Vulkan (el modelo de memoria, las barreras y la sincronización).
