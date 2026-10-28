# Render final sin esperar desde la ventana

Entrega del **28 de octubre de 2026**. F6 inicia un trabajador de render y vuelve
al bucle de eventos del editor. El mismo botón cambia a **Cancelar · F6** mientras
hay trabajo. La cabecera muestra preparación, porcentaje de píxeles calculados,
exportación y resultado. No se modifica el formato de escena v5.

## Comportamiento

- Cada trabajo captura los parámetros existentes al pulsar F6. Editar después
  cambia la vista previa; el render iniciado conserva su propia instantánea.
- Solo hay un render final a la vez. F6 durante el trabajo solicita cancelación.
  Puede iniciarse otro después de terminar o cancelar el anterior.
- La cancelación se comprueba antes de preparar, entre lotes y antes de publicar
  archivos. Un despacho GPU ya enviado termina normalmente. Si la escritura de
  archivos ya empezó, puede completarse y se informa como render guardado.
- Cerrar la ventana solicita cancelación y sigue atendiendo eventos/dibujando
  hasta que el trabajador libere recursos; no se destruyen buffers en vuelo.
  La ruta de error conserva un join de respaldo para liberar correctamente.
- Guardar/abrir escenas siguen siendo operaciones explícitas y síncronas pequeñas.
  Esta entrega separa el render final, no toda posible espera de archivos o driver.

## CPU, GPU y comunicación

La app crea el contexto de cálculo dentro del trabajador; no comparte el objeto
Vulkan de la ventana entre hilos. El trabajador valida/emite shaders y prepara
recursos. El cálculo de rayos, sombras, media y color sigue en GPU. La CPU envía
rangos de grupos ya previstos y espera sus vallas fuera del hilo de la ventana.
No es un planificador autónomo general en GPU.

Un canal acotado de **ocho mensajes** lleva solo estados y contadores pequeños.
Se descarta progreso intermedio si el canal se llena; el resultado final se
recupera del trabajador terminado. Un indicador atómico comunica cancelación.
El sondeo de la ventana no espera una valla GPU ni un trabajador activo.

La ejecución conserva **un pipeline, un conjunto, dos buffers y un comando
reutilizable** por trabajo. El buffer de salida completo permanece en GPU entre
lotes. No se recompila el shader ni se descarga la imagen en cada lote.
Solo al completar todos los grupos se hace una copia y lectura final para exportar.
Cancelar antes de esa copia devuelve ausencia de resultado, sin descargar imagen.

Se usa [vkCmdDispatchBase](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdDispatchBase.html)
para preservar el índice global de cada píxel al dividir el trabajo. El pipeline
habilita DISPATCH_BASE y el pool permite reiniciar comandos; las vallas preceden
a cada reinicio y las barreras publican las escrituras para cálculo/copia/host.
El emisor SPIR-V conserva su tamaño de grupo de 64 y sus comprobaciones de límites.

La app elige `clamp(32768 / (64*spp), 1, 64)` grupos por lote. La granularidad
mínima es un grupo: hasta **65536 muestras de cámara por lote** con 1024 spp,
más los rayos de sombra asociados. Es un presupuesto de trabajo, no una garantía
temporal. Un dispositivo/driver saturado todavía puede retrasar presentación;
la vista previa y el render final compiten por la misma GPU.

El porcentaje cuenta píxeles de lotes cuya valla terminó, no tiempo estimado ni
contadores descargados desde VRAM. Llegar al 100% del cálculo no implica que los
archivos ya estén escritos: hay una fase separada de exportación.

## Coste y evidencia

Se mantienen los límites de calidad del editor: hasta 512 por eje, 1024 spp y
1048576 muestras totales. Salida lógica de 20 B/píxel más 20 de staging, además
de alineación, código, pipeline y recursos del contexto. Durante el render coexiste
con el visor; pico físico completo sin medir. RAM adicional: instantánea de escena,
hilo, hasta ocho mensajes y descarga/archivos al terminar. Se conserva el esquema
de nombres nuevos y escritura protegida frente a destinos existentes.

Pruebas en **RTX 3060 / Windows**:

- **IGUAL:** hash para 1003 invocaciones en lotes de tres grupos, incluido el
  último grupo parcial. Progreso esperado 0/192/384/576/768/960/1003.
- **IGUAL:** HDR, contador y revisión RGB idénticos entre un despacho y cuatro
  lotes sobre la misma escena de render, en este dispositivo.
- Cancelación antes, entre y al acabar los lotes; salida ausente y sin archivos
  nuevos. Validación de límites y cabecera antes de crear módulos Vulkan.
- Trabajador retenido por una puerta de prueba: cien sondeos retornan mientras
  sigue ocupado; después atiende cancelación y termina. Sin esperas temporales
  artificiales en el código del producto.
- Prueba nativa de 120 fotogramas con exportación en paralelo y redimensionado,
  cero avisos de validación. En una ejecución de diagnóstico se dibujaron
  **13 fotogramas con render final activo**. No es un benchmark de latencia ni
  una garantía de FPS o respuesta en otros equipos.

Referencias de código: tests/lotes.rs y tests/objetos_editor.rs de telar-app,
y la prueba del controlador en render_asincrono.rs. `cargo verify` incluye
regresiones del visor procedural y de las escenas anteriores.

El visor ahora tiene su propio historial progresivo, independiente de este trabajo
final: [VISOR-PROGRESIVO.md](VISOR-PROGRESIVO.md). Faltan presentación de avances
del render final, rebotes, BVH y denoiser. La exportación por lotes mantiene su
instantánea y resolución/aspecto propios.
