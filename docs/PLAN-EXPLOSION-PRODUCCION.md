# Objetivo de producción: explosión 3D dirigible y reproducible

Objetivo acordado: construir en TELAR una cadena capaz de producir una explosión
con humo, fuego, estelas y acabado cinematográfico, tomando como referencia visual
la imagen compartida. No se conoce su autoría ni el procedimiento que la produjo.
Se estudian guías públicas; no se dispone del código propietario de Houdini.

## Recorrido de implementación

| Hito | Entrega verificable | Estado actual |
|---|---|---|
| P1 · Campos 3D | Velocidad MAC, densidad y calor persistentes; advección trilineal, fuerzas, presión, visor orbitable y referencia escalar | Base implementada y probada a 32³; ver HUMO-3D.md |
| P2 · Tiempo y colisiones | Subpasos temporales, control de estabilidad, obstáculos y velocidades de colisión; error de presión medido | Parcial: subpasos, predictor CFL GPU y esfera estática probados; faltan mallas y colisiones móviles |
| P3 · Explosión | Fuente animada de corta duración, expansión objetivo, reacción/enfriamiento y humo; parámetros separados de simulación y aspecto | Parcial: pulso finito de calor/humo y expansión compensada en GPU; combustible/reacción y fuentes dirigibles pendientes |
| P4 · Render volumétrico | Absorción, emisión, sombras y dispersión; temperatura con unidades/mapeo y gestión de color | Pendiente; existe fuego analítico separado |
| P5 · Secundarios | Trayectorias, estelas, chispas y fragmentos con edad/vida; fuentes secundarias controlables | Pendiente |
| P6 · Caché y salida | Estado versionado por fotograma/semilla, reanudación comprobada, secuencia HDR y pases | Pendiente; guardar parámetros no es cachear campos |
| P7 · Plano final | Cámara, composición, rango dinámico y comparación reproducible de secuencia; coste GPU/VRAM publicado | Pendiente |

Los hitos se cierran con datos y pruebas. Añadir una carpeta, tarjeta, shader o
captura no completa por sí solo una simulación ni una herramienta de producción.
Los cambios se mantienen en Rust, con núcleo no_std, Vulkan aislado y semáforos.

## Contratos para artistas

- Fuente: posición, tamaño, dirección, duración e intensidad con unidades explícitas.
- Simulación: escala de celda, dt/subpasos, fuerzas, disipación y presupuesto/error.
- Aspecto: densidad visible, emisión, color y dispersión separados del estado físico.
- Inspección: rangos, cortes, campos y causas de parada; resúmenes calculados en GPU.
- Reproducibilidad: configuración, semilla, paso, versión de esquema y salida asociada.
- Flujo: integrar los editores en proyecto y paneles compartidos sin perder cambios.

## Criterios de aceptación

1. Comparar rejillas pequeñas contra una referencia escalar independiente.
2. Validar SPIR-V, accesos, sincronización y resultados finitos en Vulkan.
3. Medir el residual y distinguir tolerancia alcanzada de presupuesto agotado.
4. Verificar pausa, reinicio, edición, movimiento de cámara y repetibilidad en GPU.
5. Publicar reserva lógica y límites; ninguna promesa fija de FPS o consumo cero.
6. Comprobar una secuencia completa, no solo una imagen favorable.

La CPU mantiene eventos, cámara, archivos y envíos; los campos se calculan y
conservan en GPU. La referencia CPU se utiliza para comprobar, no para simular
los fotogramas del producto. El primer prototipo 3D será deliberadamente pequeño
y denso antes de estudiar regiones activas y mayores resoluciones. La entrega
[HUMO-3D.md](HUMO-3D.md) documenta esa base y su captura; P2 tiene su primera entrega en [TIEMPO-COLISION-3D.md](TIEMPO-COLISION-3D.md); P2 avanzado y P3–P7 continúan abiertos.

## Guías públicas de referencia

- [Crear explosiones en Pyro](https://www.sidefx.com/docs/houdini/pyro/explode): núcleo y estelas como fuentes controlables.
- [Funcionamiento de Pyro](https://www.sidefx.com/docs/houdini/pyro/background.html): campos, fuerzas y proyección.
- [Pyro Solver Sparse](https://www.sidefx.com/docs/houdini/nodes/dop/pyrosolver_sparse.html): operadores de forma por escalas.
- [Pyro Bake Volume](https://www.sidefx.com/docs/houdini/nodes/sop/pyrobakevolume.html): desarrollo del aspecto del volumen.

No se anuncia equivalencia Pyro ni compatibilidad VEX/.hip. La comparación de
calidad y rendimiento contra Houdini permanece **ABIERTO** hasta medirla.

La primera pieza de P3 queda documentada y probada en [PULSO-EXPANSIVO-3D.md](PULSO-EXPANSIVO-3D.md). Las capturas del pulso no equivalen al plano cinematográfico objetivo.

## Referencias para secundarios e intercambio

Para preparar P5/P6 se conservan [Partio, LAIKA BGEO y documentación oficial](../referencias/partio-houdini-intercambio.md).
El registro distingue partículas de cachés de campos y BGEO clásico de variantes
modernas. Su integración sigue ABIERTO; no añade dependencias ni completa esos hitos.
