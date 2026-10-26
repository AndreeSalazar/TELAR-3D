# TELAR 3D

> Los hilos pasan por los nudos, y el telar teje mundos 3D.

Un sistema 3D procedural **propio**, basado en nodos, inspirado en lo que enseña Houdini Apprentice. No para competir en el mercado, sino para **encararlo de verdad con programas**: entender cada punto, cada atributo y cada ecuación, y construirlo en Rust con control total.

**La estrella del norte:** una **esfera de fuego ultrarrealista**, simulada y renderizada entera en la GPU. Esta vez no manda la CPU: **manda la GPU**.

| Pieza | Nombre | Qué es |
|---|---|---|
| El programa | **TELAR** | el telar procedural: no se modela a mano, se teje con reglas |
| El núcleo | **NUDO** | geometría, atributos y el grafo que cocina; `no_std` |
| El lenguaje | **HILO** | el lenguaje de los nodos; se compila a SPIR-V y corre en los hilos de la GPU |

## Por qué así

- **Rust:** seguridad de memoria por defecto, y `unsafe` solo donde se habla con el sistema (Vulkan, Win32), marcado y vigilado.
- **`no_std` en el núcleo:** NUDO y HILO no dependen de ningún sistema operativo. Hoy corren en Windows; mañana, sin cambiar una línea, en Linux o en BMO-X.
- **Control total:** cero dependencias en el código. Vulkan y Win32 se hablan a mano, como VERX Desktop habla con WebView2. Los SPIR-V los emite nuestro propio compilador. Las herramientas externas de *verificación* sí valen (capas de validación de Vulkan, `spirv-val`, Houdini como referencia), igual que QEMU en BMO-X: miden, pero no forman parte del programa.
- **La GPU manda:** los datos viven en la GPU de principio a fin. La CPU solo envía parámetros; la GPU decide el trabajo siguiente (dispatch indirecto, colas en la GPU).
- **Guardianes:** marca lo peligroso, hazlo pequeño, justifícalo y que un guardián lo vigile. Ver [docs/GUARDIANES.md](docs/GUARDIANES.md).

## Principios (desarrollo guiado por guardianes)

1. **Marca lo peligroso:** `unsafe`, memoria compartida en la GPU, barreras, atómicos. Así se acelera sin miedo en el resto.
2. **Nadie es root:** ni el autor, ni la IA, ni el compilador se saltan las reglas.
3. **La realidad manda:** el hardware real > la referencia (Houdini Apprentice) > la teoría. Si no coinciden, se investiga y se dice.
4. **Cada bloqueo enseña:** regla, motivo, culpable exacto y arreglo.
5. **Lo abierto se dice:** cada resultado lleva su etiqueta: **IGUAL**, **MÁS RÁPIDO**, **DISTINTO** u **ABIERTO**.

## Los planes

| Documento | Qué contiene |
|---|---|
| [docs/ARQUITECTURA.md](docs/ARQUITECTURA.md) | los crates aislados, qué hace la CPU y qué la GPU, el flujo de un fotograma |
| [docs/HOUDINI-MAPA.md](docs/HOUDINI-MAPA.md) | cada contexto de Houdini Apprentice y su equivalente en TELAR, con prioridad |
| [docs/MATEMATICAS.md](docs/MATEMATICAS.md) | la matemática necesaria y de qué artículo o libro sale |
| [docs/ESFERA-DE-FUEGO.md](docs/ESFERA-DE-FUEGO.md) | la simulación y el render de la esfera de fuego, paso a paso en la GPU |
| [docs/GUARDIANES.md](docs/GUARDIANES.md) | las reglas y por qué existen |
| [docs/PLAN.md](docs/PLAN.md) | las fases, cada una con su demo y su «hecho cuando» |

## La máquina de referencia

AMD Ryzen 5 5600X · NVIDIA GeForce RTX 3060 · Windows 11 · Vulkan 1.4 (SDK 1.4.350) · Rust 1.99.

Todas las cifras de rendimiento se miden aquí y se publican con la máquina al lado.
