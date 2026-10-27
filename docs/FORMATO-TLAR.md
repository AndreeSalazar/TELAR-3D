# .tlar v1 · archivo de arranque de TELAR 3D

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
