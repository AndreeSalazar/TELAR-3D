# .tlar · arranque v1 y escena v2 de TELAR 3D

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
