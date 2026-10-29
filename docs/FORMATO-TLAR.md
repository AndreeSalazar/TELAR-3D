# .tlar · arranque v1, escena v2, grafo v3 y MSAA ampliado v4

## Arranque v1 (conservado)

El formato propio guarda la cabecera TLAR y el coordinador SPIR-V. Es un primer
tipo de archivo de ejecución/diagnóstico, todavía no una escena completa ni un
paquete de todos los nodos, materiales o texturas.
Es una instantánea de arranque, no un ejecutable autónomo: no incluye los demás
trabajadores ni crea sus recursos. La app actual solo exporta e inspecciona este tipo.

Prefijo de cinco u32 little-endian: firma TLAR, versión de archivo 1, número de
palabras de arranque (32), número de palabras SPIR-V y CRC32 del contenido. Sigue
la cabecera de arranque y el módulo SPIR-V. El máximo es 1.048.576 palabras SPIR-V;
el lector rechaza longitudes incoherentes, exceso, bytes sobrantes y truncamiento.
CRC32 detecta corrupción accidental; no es una firma ni autenticación criptográfica.

```powershell
cargo run -p telar-app --release -- --exportar-tlar arranque.tlar --puntos 65536
cargo run -p telar-app --release -- --inspeccionar-tlar arranque.tlar
```

Exportar consulta el perfil real. Inspeccionar solo lee y verifica el archivo;
no crea dispositivo ni ejecuta su SPIR-V. La validez de cinco palabras iniciales
no demuestra que el programa completo sea válido o seguro. Una ejecución futura
de archivos deberá validar todo el módulo y reconstruir el contrato para la GPU
destino. La identidad y capacidades guardadas son una instantánea del exportador.

Coste: CPU y RAM O(bytes) al leer/escribir y calcular CRC, disco para el archivo,
ningún coste por fotograma. No se carga código automáticamente desde un .tlar.
El contenedor reside en `telar-kernel/src/paquete.rs`; la E/S queda en la app.

## Escena v2 (datos del estudio actual)

Tipo separado, sin shaders ni cabecera GPU. Tamaño exacto: 128 bytes. Todos los
enteros y bits IEEE754 de f32 son u32 little-endian. Prefijo de cuatro palabras:
firma TLAR `0x52414c54`, versión 2, longitud de contenido 28 palabras, CRC32 del
contenido. CRC usa el mismo polinomio reflejado `0xedb88320`, inicial u32::MAX e
inversión final. El prefijo se valida semánticamente, no se incluye en el CRC.

| Palabras del contenido (desde cero) | Significado |
|---|---|
| 0..1 | Lado y capacidad de Rejilla, 2..1024, lado ≤ capacidad |
| 2..3 | Amplitud y velocidad de Onda, f32 en 0..0,8 y 0..2 |
| 4 | Factor de Relieve, f32 en 0..3 |
| 5..6 | Exposición y rugosidad, f32 en 0,25..4 y 0,08..1 |
| 7..8 | Material 0..2 y MSAA solicitado 1/2/4 |
| 9..10 | Selección 0..3 y pausa 0/1 |
| 11..18 | Cuatro posiciones x/y normalizadas, f32 en 0..1 |
| 19..21 | Objetivo x/y/z de cámara, f32 en -10⁶..10⁶ |
| 22 | Distancia, f32 en 0,001..10⁶ y al menos dos veces el plano cercano |
| 23..25 | Giro -360..360, inclinación -89..89, campo vertical 1..179 grados |
| 26 | Plano cercano, f32 en 10⁻⁶..10³ |
| 27 | Fase, f32 en [0,1000) |

Se rechazan NaN, infinitos, sobrantes y campos desconocidos. La validación no
modifica el estudio hasta comprobar todos los campos. Nunca se persisten manejadores
Vulkan, muestras efectivas, fuentes, CSS ni estado de arrastre. El codec de UI
es puro (`telar-ui/src/estado.rs`); cámara, contenedor y E/S están en la app
(`proyecto.rs` y `sesion.rs`).

Uso y límites: [ESCENAS.md](ESCENAS.md). V2 representa la cadena fija, no un grafo
general ni un archivo de producción con texturas/volúmenes. Una futura extensión
requiere contrato y versión propios; no se interpretan bytes adicionales como código.

## Grafo v3

Contenedor separado con el mismo prefijo de cuatro u32: firma TLAR, versión 3,
cantidad total de palabras de contenido y CRC32 del contenido completo. Las primeras
28 palabras conservan los parámetros base, cámara y fase de v2. Sigue una extensión
de `2 + 7N` palabras, donde `N` es el número de identidades en 4..10. Tamaño total
248..416 bytes; el lector de escena está limitado a 417 bytes para detectar exceso.
V2 conserva su tamaño exacto de 128 bytes y se escribe si la red no ha sido editada.

La extensión empieza por `N` y la selección actual (0..N-1). Cada registro, en
orden de identidad estable, contiene siete palabras:

| Palabra del registro | Dato |
|---|---|
| 0 | Productor de entrada; `u32::MAX` solo para Rejilla (ID 0) |
| 1 | Habilitado 0/1; Rejilla y Salida siempre 1 |
| 2..3 | Posición f32: x en 0..1; y en 0..16 unidades de lienzo de 80 px |
| 4 | Tipo: 0 Rejilla, 1 Onda, 2 Relieve, 3 Salida |
| 5..6 | Onda: amplitud/velocidad; Relieve: factor/0; fuente/salida: 0/0 |

Los IDs 0/1/2/3 mantienen tipos Rejilla/Onda/Relieve/Salida. Los demás solo pueden
ser Onda o Relieve. Los parámetros de 1/2 deben coincidir con el prefijo; los rangos
son los de sus inspectores. Las posiciones y selección completas de la extensión
son autoritativas; el prefijo conserva una selección máxima de 3 y posiciones
limitadas a 0..1 para mantener el dominio del registro base. No se guardan pan,
arrastres ni texto de edición pendiente. El pan arranca en cero al reabrir.

Cada nodo salvo Rejilla debe tener un productor existente distinto de Salida;
todas las cadenas deben terminar en Rejilla sin ciclos. La comprobación incluye
ramas no alcanzables desde Salida. Los campos reservados deben ser cero y todos
los flotantes deben ser finitos. El visor compila solo el camino de Salida;
omitir un nodo conserva sus datos y deja pasar su entrada.

V3 contiene únicamente datos. El SPIR-V se construye con el compilador incorporado
tras validar el grafo; no se deserializa código del archivo. El CRC no autentica
el archivo. La fase v3 es el reloj común; cada Onda aplica su multiplicador de
velocidad. V2 mantiene su semántica de fase histórica. Ver [GRAFO-EDITABLE.md](GRAFO-EDITABLE.md).

## Escena v4 · solicitudes MSAA superiores a 4

V4 amplía el dominio del máximo MSAA solicitado a **8/16/32/64**. Mantiene el
prefijo, CRC, registro base de 28 palabras y extensión opcional de red ya descritos.
Sin red ocupa 128 B; con red, 248..416 B. La fase conserva la semántica de la cadena
histórica o de la red según la presencia de esa extensión.

El escritor usa v4 solo para solicitudes superiores a 4; para 1/2/4 conserva v2
(cadena histórica) o v3 (red editable). Los lectores nuevos rechazan muestras altas
etiquetadas como v2/v3 y v4 con muestras bajas: la versión debe coincidir con el
dominio de datos. Los lectores antiguos rechazan v4 antes de preparar recursos.

Solo se guarda la **petición**, nunca el soporte ni el resultado de otra GPU.
Al abrir se consulta el soporte de color y profundidad en el dispositivo destino y
se elige el mayor nivel admitido que no exceda la petición. Por ejemplo, una petición
32 puede dar 8 efectivos y se conserva como 32 al volver a guardar. El visor muestra
ambos números. V4 sigue siendo datos acotados, sin shaders proporcionados por archivo.

## Escena v5 · objetos y luz editables

Formato independiente del payload de rejilla: 184 B fijos, cabecera LE u32
`[0x52414c54,5,42,CRC32(payload)]` y 42 palabras. No incluye código ejecutable.
Palabras relativas al payload:

| Índices | Contenido |
|---|---|
| 0..9, 10..19 | Cada caja: posición XYZ, tamaño completo XYZ, color lineal RGB y visible 0/1 |
| 20..22 | Centro de luz XYZ |
| 23, 24 | Semilado y radiancia |
| 25..27, 28 | Color lineal de luz RGB, sombras 0/1 |
| 29..33 | Exposición, spp final, ancho, alto, semilla |
| 34..36, 37..39 | Ojo XYZ y objetivo XYZ |
| 40, 41 | Selección 0..3, página 0..1 |

Las primeras 40 palabras son bits f32, incluyendo opciones enteras codificadas
como valores f32 exactos; selección/página son u32. La semilla está limitada a
16777215. Todos los valores deben ser finitos; tamaños 0.1..3, colores 0..1,
coordenadas de cajas X/Z -3..3 y Y 0..3, luz X/Z -10..10 y Y 0.1..10,
radio 0.01..3, radiancia 0..100, exposición 0.01..16. Cámara -10..10,
dirección no vertical ni colapsada. spp 1..1024, dimensiones 1..512 y producto
ancho×alto×spp <=1048576. Visible y sombras deben valer exactamente 0 o 1.

Decodificación de tamaño fijo y transaccional; el lector toma como máximo 185 B
para detectar exceso. El escritor reutiliza el temporal exclusivo y reemplazo
atómico del proyecto; no implica recuperación garantizada frente a apagones.
Los lectores de rejilla rechazan v5 y solicitan `--estudio-rayos`; los lectores
v5 rechazan v1..v4. El inspector reconoce todos los formatos sin ejecutar shaders.
Ver [EDITOR-OBJETOS.md](EDITOR-OBJETOS.md) y las pruebas objetos_editor.rs.

## Configuración de humo 3D v9

Registro de 56 B con los mismos diez parámetros f32 y CRC de carga útil que humo
2D v8; versión 9 identifica un dominio MAC 3D de 32³ en el editor actual. Los
lectores rechazan el intercambio de versiones 8/9. No contiene campos, cámara,
semilla ni caché; abrir inicia estado vacío. Ver [HUMO-3D.md](HUMO-3D.md).

## Tiempo y esfera de humo v10

Registro 72 B: cabecera TLAR/version=10/cantidad=14/CRC, diez f32 de humo y cuatro de subpasos, radio, altura y CFL. CRC sobre carga útil; dominios validados antes de aplicar. V9 migra sin esfera, un subpaso y CFL 0.5. No contiene campos/reloj/cámara. Ver [TIEMPO-COLISION-3D.md](TIEMPO-COLISION-3D.md).

## Pulso expansivo v11

Registro de 88 B: cabecera TLAR/version=11/cantidad=18/CRC y 18 f32 (humo 10, tiempo/esfera 4, pulso 4). Duración, expansión y multiplicadores calor/humo se validan antes de aplicar. V9/v10 migran sin pulso; no contiene reloj/campos/cámara. Ver [PULSO-EXPANSIVO-3D.md](PULSO-EXPANSIVO-3D.md).

## Configuración de partículas v12

56 B: cabecera u32 LE `[0x52414c54,12,10,CRC32(payload)]` seguida de diez f32:
plazas, vida, impulso, gravedad, arrastre, dt, radio visual, semilla, duración y
dispersión. Dominios en [PARTICULAS-GPU.md](PARTICULAS-GPU.md); plazas/semilla enteros.
Lector acotado a 57 B, CRC y validación antes de aplicar. Rechaza otros modos.
F5/F9 conservan configuración; abrir reinicia plazas y reloj. Sin cámara,
generaciones, posiciones ni caché reanudable.

El modo `--trazadores` utiliza configuración física MAC v11. El modificador de presentación/transporte no se serializa: reabrir con el mismo modificador. No guarda plazas ni reloj; [TRAZADORES-MAC.md](TRAZADORES-MAC.md).

## Plano VFX v13 y caché independiente

200 B: cabecera TLAR/version=13/cantidad=46/CRC32(payload), seguida de 18 f32
del contrato v11 y 28 f32 de `telar-ui/src/vfx.rs`, en ese orden. Se comprueban
CRC, tamaño y dominios antes de aplicar; lector acotado a 201 B. F5/F9 conservan
parámetros, no reloj ni campos; abrir requiere el modo `--plano-vfx`.

.tlcache revisión 2 es otro formato: doce u32 LE de cabecera (magic 0x48434c54,
revisión, lado=32, modo 0/1/2, palabras, avances, vendor, dispositivo, driver,
versión Vulkan, bytes de escena, CRC), escena v11 de 88 B o v13 de 200 B,
cámara de ocho f32 y todas las palabras GPU. CRC protege cabecera salvo el propio
CRC y cuerpo. Modo 2 exige v13; 0/1 exigen v11. Límite 16 MiB, recuentos exactos,
valores finitos y perfil de GPU comprobados antes de subir. Una revisión futura
debe cambiar si cambia el ABI o la interpretación del solver. Publicación
exclusiva sin reemplazo; [uso, restricciones y pruebas](PLANO-VFX.md).
