# Referencias contra Houdini

Bibliografía de interoperabilidad y antecedentes:
[Partio, BGEO y documentación pública de Houdini](partio-houdini-intercambio.md).
Separa fuentes verificadas de compatibilidad todavía ABIERTO; no es un resultado
de comparación de nodos ni una dependencia de TELAR.

Una por nodo (lo exige la regla `nodo-completo`). Cada una dice:
- qué mide TELAR (comprobado por sus tests);
- qué debería dar Houdini (según su documentación);
- cómo comprobarlo en Houdini Apprentice;
- su **etiqueta**.

| Etiqueta | Significa |
|---|---|
| **IGUAL** | comprobado en Apprentice: la misma salida, dentro de la tolerancia |
| **MÁS RÁPIDO** | igual, y además en menos tiempo, con la cifra y la máquina |
| **DISTINTO** | comprobado, sale otra cosa, y se explica por qué |
| **ABIERTO** | todavía no comparado en Apprentice. Se dice, no se finge |

Cómo leer las cifras en Houdini:
- **El nodo:** clic central (o el botón «i») enseña la información: puntos, primitivas, vértices y la caja que la contiene (*bounds*).
- **El Geometry Spreadsheet** enseña cada punto y su `P`.

Primera aplicación del registro: [intercambio GEO](../docs/INTERCAMBIO-GEO.md), con datos y pruebas propios. La comparación externa permanece ABIERTA.
