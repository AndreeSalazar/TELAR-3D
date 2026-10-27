# Semáforos obligatorios y consumo de recursos

Cada carpeta del proyecto con archivos propios lleva un **SEMAFORO.md**. El guardián
`semaforos` lo exige en `cargo verify` y al commitear. Incluye raíz, configuración,
documentos, fuentes, pruebas y ejemplos. Se excluyen `.git`, `target` y `node_modules`:
son control de versiones, salida generada o dependencias externas, no carpetas fuente.

- 🔴 **ROJO — crítico:** fronteras FFI, memoria GPU, ejecución indirecta o código
  que puede comprometer esos contratos. Todas las carpetas FFI deben ser rojas.
- 🟡 **AMARILLO — riesgo probable:** comportamiento, rendimiento, entradas externas
  o resultados numéricos que requieren comprobación y mediciones adicionales.
- 🟢 **VERDE — seguro en su contexto declarado:** contenido de bajo riesgo y sin
  operaciones críticas directas. No es una garantía absoluta ni significa coste cero.

Los padres reflejan al menos el máximo riesgo declarado por sus hijos. El color
no aumenta o disminuye automáticamente al pasar un test: riesgo y resultado de
verificación son cosas distintas. Una nueva carpeta sin informe bloquea el guardián.

Formato obligatorio, una línea por campo:

```text
Estado: AMARILLO
Motivo: Explicar el riesgo concreto de esta carpeta.
CPU: Qué trabajo hace y cuándo; indicar ausencia de trabajo directo si procede.
GPU: Qué programa se ejecuta o si solo genera código para ejecutar después.
VRAM: Qué reserva directamente, quién la reserva o si no hay reserva propia.
RAM: Estructuras y crecimiento esperado.
E/S: Lectura/escritura, presentación, herramientas o ausencia de E/S directa.
Frecuencia: Arranque, cambio, fotograma, fragmento, prueba o solo contenido.
Limites: Límites conocidos y medidas pendientes; no inventar tiempos ni memoria libre.
Evidencia: Prueba, contrato o documento que permite revisar la afirmación.
```

Se distingue consumo **directo** de consumo por descendientes o programas
generados. Un emisor SPIR-V usa CPU/RAM al compilar y genera trabajo GPU posterior;
la carpeta no reserva VRAM por existir. Una prueba GPU puede consumir mucha VRAM
al ejecutarse aunque el producto no la utilice por fotograma.

El guardián verifica presencia, campos, valores de color, fronteras FFI y jerarquía.
La exactitud de una estimación requiere revisión y medición humana; no se infiere
un tiempo de GPU mirando una etiqueta. Las cifras no medidas se declaran ABIERTAS.
