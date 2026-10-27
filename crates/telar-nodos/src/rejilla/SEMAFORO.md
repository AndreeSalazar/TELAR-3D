# 🟡 Semáforo · crates/telar-nodos/src/rejilla

Estado: AMARILLO
Motivo: Matemática y geometría CPU; parámetros y recuentos afectan memoria y exactitud.
CPU: Cocinado de referencia, matrices/cámara y pruebas; la rejilla residente del visor se genera en GPU.
GPU: Sin despacho GPU directo en estos crates; un resultado CPU puede subirse por la app en modo de referencia.
VRAM: Sin reserva propia; el modo esfera-cpu delega su subida en telar-gpu.
RAM: Atributos, puntos, índices y grafo; crece con geometría y parámetros solicitados.
E/S: Sin E/S del núcleo no_std; los libretos y referencias se consultan en desarrollo.
Frecuencia: Cambios de grafo/parámetros, cámara y pruebas; cocinado CPU solo en las rutas que lo solicitan.
Limites: ABIERTO: tiempos y pico global; no atribuir una medición a una etiqueta.
Evidencia: Pruebas de nudo/telar-nodos, libretos de cada nodo y referencias etiquetadas.

Clasificación de riesgo, no certificación absoluta ni medición de rendimiento.
