# Datos visibles y herramientas VFX de TELAR

Actualización: 28 de octubre de 2026. La petición de ampliar TELAR hacia una
alternativa VFX completa extiende el alcance del mapa inicial; las funciones
históricamente marcadas «Fuera» pasan a ser investigación futura, sin fecha ni
equivalencia prometida. No se ha completado «todo Houdini».

## Qué aprender de la documentación pública

En Houdini, las redes y sus parámetros definen operaciones sobre datos. TELAR
debe hacer visible la entrada, salida, tipo, dominio y coste de cada operación;
un cable debe transportar datos con contrato, no ser únicamente un dibujo.
Fuente: [Networks and parameters](https://www.sidefx.com/docs/houdini/network/index.html).

Los atributos se asocian a puntos, vértices, primitivas o al detalle completo.
Propuesta TELAR: inspector con nombre, tipo, dominio, unidades, cantidad y
mínimo/máximo; tabla de una muestra limitada solicitada por el artista. Las
reducciones estadísticas irían en GPU y se descargaría solo el resumen bajo
petición, sin copiar toda la geometría cada fotograma.
Fuente: [Geometry attributes](https://www.sidefx.com/docs/houdini/model/attributes.html).

| Sistema de referencia | Contrato propuesto para TELAR | Situación real |
|---|---|---|
| SOP: geometría y atributos | Puertos tipados, selección por grupos, cocinar solo cambios | Grafo de rejilla existente; geometría general e inspector pendientes |
| VEX: expresiones sobre datos | HILO con variables tipadas y operaciones sobre atributos, compilado a SPIR-V | IR y compilación existentes; editor de texto y lenguaje de usuario pendientes |
| VOP: operaciones visuales | Misma IR que HILO, errores de tipos en cables, previsualizar salida | Pendiente; no confundir con los nodos limitados actuales |
| Pyro: volúmenes | Fuente, campos, solver, caché y shader como etapas independientes | Esfera procedural visible; solver físico pendiente |
| Vellum: restricciones | Posiciones/velocidades/masa inversa y restricciones XPBD | Plan; no hay solver de telas ni cuerpos blandos implementado |
| POP: partículas | Identidad estable, edad, vida, posición y velocidad | Pendiente |
| RBD / líquidos | Colisiones, restricciones y transferencia entre partículas/rejilla | Pendiente |
| Composición y producción | AOV, color, secuencias, cachés reproducibles, trabajos cancelables | HDR y render por lotes acotados; resto pendiente |

VEX es un lenguaje de expresiones utilizado en varios contextos; esto no implica
que todos los nodos o expresiones de Houdini se ejecuten en GPU. HILO será lenguaje
propio; no se anuncia compatibilidad VEX ni carga de proyectos `.hip`.
Fuente: [VEX](https://www.sidefx.com/docs/houdini/vex/index.html).
«HOP» queda sin identificar como requisito; no se lo convierte automáticamente en
otro contexto. SOP, DOP, VOP y CHOP son conceptos diferentes.

## Esfera física: siguientes entregas y criterios de salida

1. **Campos persistentes:** densidad, calor y velocidad con rejilla pequeña y
   dos copias para advección. Inspector de rangos y cortes. Probar interpolación,
   condiciones de borde y ausencia de lecturas/escrituras fuera de rango.
2. **Humo:** advección, flotación y proyección de presión. Mostrar divergencia
   antes/después; medir error, disipación numérica y tiempo GPU por etapa.
3. **Fuego:** modelo explícito de reacción, enfriamiento y expansión, con
   parámetros de fuente separados de los del solver y del shader.
4. **Calidad:** sombras del volumen, dispersión, cuerpo negro con unidades y
   gestión de color; integración con mallas, luces e historial progresivo.
5. **Producción:** caché por fotograma/semilla, secuencias HDR y pases separados,
   límite de VRAM y cancelación entre lotes. Comparación de datos e imagen con
   escena documentada; etiqueta ABIERTO hasta medir.

Pyro usa campos escalares/vectoriales, advección y proyección; `flame` tiene
semántica de vida restante de reactivos. El `flame` procedural actual de TELAR es
solo una máscara artística y no comparte esa semántica. Esta distinción debe
mantenerse al crear el solver y versionar archivos.
Fuente: [Pyro internals](https://www.sidefx.com/docs/houdini/pyro/background.html).

## Restricciones: comenzar con una tela pequeña

Vellum utiliza dinámica basada en posiciones para distintos materiales. El diseño
propuesto para TELAR empieza con XPBD de distancia, fijaciones y flexión en una
tela; después colisiones, autosolapamientos, presión, plasticidad y rotura.
Los atributos del artista deben distinguir masa, fricción, grosor y rigidez de
las variables internas del solver. No reutilizar un parámetro de render para
simular un material. Probar unidades, dependencia de paso temporal, estiramiento
residual y determinismo; elegir resolución paralela sin sumas atómicas desordenadas.
Fuentes: [Vellum](https://www.sidefx.com/docs/houdini/vellum/index.html),
[atributos de Vellum](https://www.sidefx.com/docs/houdini/vellum/vellumattributes.html).

La CPU conserva ventana, edición y envíos Vulkan; los campos y las restricciones
deberán permanecer en GPU. La autonomía se evalúa por tráfico, sincronizaciones,
trabajo despachado y tiempos, sin atribuir pensamiento autónomo al hardware.

## Actualización: primer solver de humo

Las primeras piezas de los puntos 1/2 ya existen en [HUMO-MAC-2D.md](HUMO-MAC-2D.md): doce planos persistentes, emisión, advección, flotación y presión, con vistas e inspección explícita. El editor funciona a 128² con paredes cerradas y presión de iteraciones fijas. Faltan cortes 3D, reducciones estadísticas GPU, solución por tolerancia y combustión; los otros sistemas de la tabla conservan su estado pendiente.

El criterio de presión por tolerancia ya está disponible en [PRESION-ADAPTATIVA.md](PRESION-ADAPTATIVA.md), con controles y diagnóstico residente. Sigue pendiente un solver más eficiente, humo 3D, subpasos, combustión y los restantes sistemas de la tabla.

## Organización visual

Ya existe un [centro nativo por categorías](ESTUDIO-CATEGORIAS.md), con acceso a editores disponibles y descripción de capacidades pendientes. La sesión compartida, los paneles acoplables y los grafos de contexto siguen pendientes.

## Explosión: recorrido y volumen inicial

[PLAN-EXPLOSION-PRODUCCION.md](PLAN-EXPLOSION-PRODUCCION.md) separa campos,
tiempo/colisiones, reacción, luz volumétrica, secundarios, cachés y plano final.
[HUMO-3D.md](HUMO-3D.md) entrega la primera base 32³, campos inspeccionables y
cámara orbitable; los restantes hitos siguen pendientes.
