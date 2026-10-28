# Partio, BGEO y referencias públicas de Houdini

Revisión: **28 de octubre de 2026**. Registro de fuentes primarias solicitado
para TELAR. Lecturas verificadas; **ABIERTO** para compatibilidad, corrección y
rendimiento contra Houdini: no se ha ejecutado una comparación externa.

Estas fuentes documentan intercambio de datos, APIs y herramientas públicas.
No constituyen evidencia de una reconstrucción completa por ingeniería inversa
de Pyro, Vellum, Karma o del núcleo de Houdini. Tampoco proporcionan por sí solas
un solver GPU ni un renderizador de producción para TELAR.

## Fuentes conservadas

| Fuente primaria | Qué se comprobó | Para qué estudiarla en TELAR |
|---|---|---|
| [Repositorio wdas/partio](https://github.com/wdas/partio) | Biblioteca C++ con enlaces Python; modelo de partículas/atributos y herramientas de inspección/conversión | Diseñar datos e intercambio de partículas |
| [Partio en Disney Animation](https://www.disneyanimation.com/open-source/partio/) | Motivación de unificar datos entre formatos GEO/BGEO, PTC, PDB/PDA | Entender un flujo de producción entre aplicaciones |
| [Lector BGEO de Partio](https://github.com/wdas/partio/blob/main/src/lib/io/BGEO.cpp) | El archivo consultado acepta BGEO antiguo versión 5 y rechaza la cabecera de BGEO nuevo, sugiriendo `.bhclassic` | Delimitar el primer formato comprobable; evitar prometer soporte universal |
| [Licencia de Partio](https://github.com/wdas/partio/blob/main/LICENSE) | Aviso de Disney y condiciones de redistribución y uso de nombres | Conservar la procedencia y consultar condiciones antes de cualquier reutilización |
| [LAIKA bgeo_reader](https://github.com/LaikaStudios/bgeo_reader) | Origen en 2014/Houdini 13, uso en Katana y procedimientos PRMan; soporte parcial y dependencia de la biblioteca JSON de Houdini | Estudiar lectura de geometría y separación de formatos/render |
| [SideFX: formato y esquema GA](https://www.sidefx.com/docs/hdk/_h_d_k__g_a__using.html#HDK_GA_FileFormat) | Cambio de formato en Houdini 12, JSON ASCII/binario y referencias de interpretación | Especificar versiones, atributos y representaciones numéricas |
| [SideFX: geo/bgeo clásico](https://www.sidefx.com/docs/houdini/io/formats/geo.html) | Cabeceras, puntos, primitivas, grupos y atributos de la descripción con PGEOMETRY | Separar la descripción clásica del esquema JSON |
| [Respuesta de personal SideFX, 11 de agosto de 2016](https://www.sidefx.com/forum/topic/45584/) | Explicación pública de JSON binario y referencias oficiales de lectura/escritura | Antecedente histórico de interoperabilidad documentada |
| [Introducción oficial al HDK](https://www.sidefx.com/docs/hdk/_h_d_k__intro.html) | API C++ para ampliar Houdini con operadores y otras funciones | Comprender contratos de extensiones; no se integra en TELAR |
| [Houdini Engine](https://www.sidefx.com/docs/houdini/ref/hengine.html) | API C para utilizar activos de Houdini desde aplicaciones anfitrionas | Distinguir integrar Houdini de construir un motor independiente |

Las ramas `main`/`master` y las páginas web pueden cambiar. Este registro no fija
un commit ni garantiza el estado de una futura descarga. Antes de implementar
un conversor se anotarán revisión exacta, versión exportadora y archivos de ensayo.
No se han copiado fuentes C++/Python ni descargado cachés de terceros al repositorio.

## Partio: alcance útil y límite importante

Su README separa información de partículas, acceso de lectura y acceso mutable,
y contempla leer únicamente cabeceras. También describe atributos no intercalados
y herramientas `partinfo`, `partconvert` y `partview`. Son referencias de diseño
para inspección y cachés. [README del proyecto](https://github.com/wdas/partio).

El soporte anunciado de BGEO debe acotarse a una variante concreta. El lector
consultado comprueba cabecera y versión 5; reconocer la extensión `.bgeo` no
demuestra que acepte un archivo moderno `.bgeo.sc`. El alcance de otros lectores
debe estudiarse aparte. [Archivo BGEO.cpp](https://github.com/wdas/partio/blob/main/src/lib/io/BGEO.cpp).

Partio se estudiará primero como referencia de **partículas y atributos**. Los
campos MAC densos de TELAR necesitan además dimensiones, transformación del
dominio, ubicación en caras/centros y metadatos de simulación. Guardarlos como
puntos sin ese contrato no preservaría automáticamente un estado reanudable.

## Antecedentes históricos: interpretación correcta

LAIKA declara que su lector comenzó en 2014 como generador de escena para Katana
con datos de Houdini 13, y evolucionó para sus necesidades de producción. Su
README enumera versiones hasta 16.5 y un subconjunto de primitivas; esto no
certifica versiones posteriores. Depender de la biblioteca JSON de Houdini no
lo convierte en un lector autónomo libre de esa instalación.
[Historia y requisitos originales](https://github.com/LaikaStudios/bgeo_reader).

El mensaje de SideFX de 2016 y la guía GA explican implementaciones oficiales de
referencia en `public/binary_json` y `public/hgeo`. Este antecedente apoya estudiar
formatos documentados; no prueba que sus autores descompilaran el solver.
[Mensaje original](https://www.sidefx.com/forum/topic/45584/),
[guía GA](https://www.sidefx.com/docs/hdk/_h_d_k__g_a__using.html#HDK_GA_FileFormat).

## Procedencia y decisión de integración

La licencia consultada de Partio exige conservar avisos y condiciones al
redistribuir fuentes/binarios y restringe usar nombres de Disney para promocionar
derivados sin permiso específico. El proyecto LAIKA declara elección MIT o
Apache-2.0. Se conservan enlaces a sus textos, sin cambiar la licencia de TELAR.
[Partio LICENSE](https://github.com/wdas/partio/blob/main/LICENSE),
[LAIKA README y licencias](https://github.com/LaikaStudios/bgeo_reader#license).

Decisión actual: **solo investigación**, sin instalar, enlazar, portar ni incorporar
código externo. Se mantienen Rust propio, núcleo no_std, backend Vulkan aislado
y cero dependencias de terceros según CLAUDE.md. Un posible uso como herramienta
externa de verificación requerirá fijar versión y comprobar el entorno cuando se
realice ese trabajo; esta anotación no lo presenta como instalado o funcional.

## Propuesta propia para TELAR

La siguiente tabla es un contrato por diseñar y probar, no compatibilidad implementada:

| Dato propuesto | Metadatos necesarios |
|---|---|
| Posición | Tres componentes, unidades y espacio de coordenadas |
| Velocidad | Tres componentes y unidades por segundo |
| Identidad | Tipo/rango entero y estabilidad entre fotogramas |
| Edad y vida | Unidad temporal y criterio de nacimiento/muerte |
| Atributos adicionales | Nombre, tipo, aridad, valor por defecto y dominio |
| Estado de volumen MAC | Dimensiones, transformación, canales, caras/centros, dt y reloj |

Plan de estudio, todavía **ABIERTO**:

1. Diseñar partículas/atributos propios y distinguir datos de simulación, caché e
   intercambio. Se relaciona con P5/P6 del plan de explosión.
2. Preparar un conjunto mínimo creado por nosotros: puntos con posición/velocidad,
   identificadores y atributos escalares; exportador y variante registrados.
3. Probar primero un formato textual o clásico documentado. Rechazar de forma
   explícita variantes no soportadas, archivos truncados y tamaños excesivos.
4. Comparar recuentos, tipos, aridad, unidades, rangos, valores y orden. Una imagen
   parecida no demuestra igualdad de datos. Publicar tolerancias y resultados.
5. Evaluar por separado BGEO JSON, compresión y otros formatos. No anunciar
   `.bgeo.sc` ni PTC hasta superar ensayos específicos.
6. Diseñar caché reanudable de campos aparte del intercambio de partículas.
   Los `.tlar` actuales guardan configuración; no son todavía esa caché.

Ubicación propuesta del trabajo: lectura/escritura y validación en la capa host,
con límites de RAM/E/S y staging explícitos; simulación, campos y dibujo siguen
en GPU. Una secuencia de caché podrá requerir E/S por fotograma: se medirá antes
de decidir precarga o streaming. No se promete coste cero ni una biblioteca GPU.

El estudio de estas fuentes no sustituye los hitos físicos pendientes de
[combustión, iluminación y salida](../docs/PLAN-EXPLOSION-PRODUCCION.md).
