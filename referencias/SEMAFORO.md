# 🟢 Semáforo · referencias

Estado: VERDE
Motivo: Documentación, referencias o datos; no se ejecutan automáticamente.
CPU: Sin trabajo de producto directo; leer, comparar o visualizar consume recursos de la herramienta.
GPU: Ningún despacho propio; una captura es un resultado, no un programa activo.
VRAM: No reserva VRAM del producto; un visor externo puede utilizar su propia memoria.
RAM: Tamaño de los documentos y datos al abrirlos; no reside por fotograma en TELAR.
E/S: Lectura o escritura de documentación, capturas y ejemplos cuando se solicitan.
Frecuencia: Solo consulta, actualización o prueba explícita del contenido.
Limites: Los .tlar son datos de arranque; su CRC no autentica código. Referencias pendientes se etiquetan ABIERTAS.
Evidencia: docs/FORMATO-TLAR.md, docs/ARRANQUE-GPU.md y docs/SEMAFOROS.md.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.

partio-houdini-intercambio.md conserva enlaces y conclusiones verificadas el
28/10/2026; las propuestas de importación/caché están ABIERTAS. No incorpora
código externo, no ejecuta Partio/HDK ni reserva recursos de producto. Incluye
límites de variantes BGEO y procedencia/licencias como referencias documentales.

El registro Partio enlaza la implementación propia de GEO y distingue pruebas internas de compatibilidad externa ABIERTA. No incorpora código de Partio/HDK; el documento no ejecuta importación.

Partículas GPU (28-10-2026): base independiente de 128 plazas; dos despachos,
6148 B lógicos compartidos y visor gaussiano hasta 384². CPU: constantes/eventos,
configuración y GEO explícito; referencia escalar solo tests. GPU: física y dibujo;
VRAM/RAM: costes lógicos y temporales descritos en docs/PARTICULAS-GPU.md, sin
medición del pico físico. E/S: v12 de 56 B y GEO de vivas; descarga explícita
5664 B al exportar, ninguna por cuadro estable. Sin acoplamiento al humo ni
compatibilidad externa certificada; perfil de ocho atributos separado del MAC.
Pruebas y captura reales: docs/PARTICULAS-GPU.md y telar-app/tests/particulas_gpu.rs.
El estado de riesgo de esta carpeta se conserva; no significa consumo cero.
