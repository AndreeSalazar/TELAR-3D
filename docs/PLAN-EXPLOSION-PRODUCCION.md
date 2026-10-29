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
| P3 · Explosión | Fuente animada de corta duración, expansión objetivo, reacción/enfriamiento y humo; parámetros separados de simulación y aspecto | Parcial: pulso, fuente posicionable, combustible/reacción/enfriamiento en GPU; faltan química calibrada y expansión ligada a reacción |
| P4 · Render volumétrico | Absorción, emisión, sombras y dispersión; temperatura con unidades/mapeo y gestión de color | Parcial: Beer–Lambert, emisión artística, luz puntual/sombra y dispersión simple HG; faltan dispersión múltiple, superficies y color de producción |
| P5 · Secundarios | Trayectorias, estelas, chispas y fragmentos con edad/vida; fuentes secundarias controlables | Parcial: hasta 128 secundarios por umbral de calor, respuesta inercial al MAC, vida/semilla e historial de cuatro posiciones; faltan escala, fragmentos y colisión continua |
| P6 · Caché y salida | Estado versionado por fotograma/semilla, reanudación comprobada, secuencia HDR y pases | Base: caché exacta ligada a GPU/driver y secuencia PFM/PPM con cuatro pases; faltan portabilidad, EXR/OpenVDB y flujo de producción |
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
modernas. [La primera aplicación propia](INTERCAMBIO-GEO.md) exporta/inspecciona muestras GEO; la compatibilidad externa sigue ABIERTA, sin dependencias externas ni cierre de esos hitos.

Primera base de P5: [PARTICULAS-GPU.md](PARTICULAS-GPU.md). No cierra P5/P6; los estados de partículas y humo siguen separados.

Primer acoplamiento de P5 con P1/P3: [TRAZADORES-MAC.md](TRAZADORES-MAC.md). Transporte pasivo y unidireccional; el emisor balístico sigue separado. No cierra P3/P4/P5/P6.

Entrega integrada del 29-10-2026: [PLANO-VFX.md](PLANO-VFX.md). Avanza P3–P6
con pruebas de reacción, transmisión y reanudación exacta. Mantiene los hitos
parciales: no completa P7 ni sustituye grafo VFX, curvas, deshacer y proyecto.
