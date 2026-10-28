//! Catálogo de capacidades reales y futuras; no equivale a compatibilidad Houdini.
/// Editores que la app puede abrir; la UI no lanza procesos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Espacio {
    /// Objetos, cámara, luz y render acotados.
    Objetos,
    /// Grafo de superficie procedural.
    Geometria,
    /// Solver MAC 2D.
    Humo,
    /// Volumen analítico animado.
    Fuego,
}
/// Descripción de una capacidad; sin destino significa plan pendiente.
pub struct Herramienta {
    /// Nombre visible.
    pub nombre: &'static str,
    /// Resumen para la tarjeta.
    pub resumen: &'static str,
    /// Alcance concreto, mostrado en el inspector.
    pub detalle: &'static str,
    /// Editor existente; None no puede abrirse.
    pub espacio: Option<Espacio>,
    /// Etiqueta que distingue simulación, procedural y alcance futuro.
    pub estado: &'static str,
}
/// Área de trabajo especializada.
pub struct Categoria {
    /// Nombre visible.
    pub nombre: &'static str,
    /// Propósito del área.
    pub resumen: &'static str,
    /// Herramientas en orden de presentación.
    pub herramientas: &'static [Herramienta],
}
/// Organización de TELAR; cada categoría mantiene su propio catálogo.
pub const CATEGORIAS: &[Categoria] = &[
    Categoria {
        nombre: "Escena",
        resumen: "Objetos, cámara y luces",
        herramientas: &[
            Herramienta {
                nombre: "Objetos y luz",
                resumen: "Componer y guardar una escena",
                detalle: "Dos cajas, suelo, cámara y luz de área editables. Incluye vista progresiva y render final acotado. Todavía no hay importación de mallas ni escenas generales.",
                espacio: Some(Espacio::Objetos),
                estado: "Disponible",
            },
            Herramienta {
                nombre: "Escenas e instancias",
                resumen: "Jerarquías y objetos generales",
                detalle: "Plan: múltiples geometrías, transformaciones, instancias e importación. El editor actual conserva una escena acotada.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
    Categoria {
        nombre: "Geometría",
        resumen: "Construcción procedural con grafos",
        herramientas: &[
            Herramienta {
                nombre: "Superficie procedural",
                resumen: "Rejilla, onda y deformación",
                detalle: "Abre el grafo editable de la rejilla con operaciones concretas y parámetros propios. La generación y deformación permanecen en GPU. No es todavía un modelador general.",
                espacio: Some(Espacio::Geometria),
                estado: "Disponible",
            },
            Herramienta {
                nombre: "Atributos y grupos",
                resumen: "Datos por punto y primitiva",
                detalle: "Plan: inspector de atributos tipados, grupos y operaciones generales. Las bases de datos geométricos existen en el núcleo; falta su edición visual completa.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
    Categoria {
        nombre: "Simulación",
        resumen: "Campos, partículas y restricciones",
        herramientas: &[
            Herramienta {
                nombre: "Humo 2D",
                resumen: "Campos persistentes y presión",
                detalle: "Solver MAC 128² en GPU: advección, fuerzas y presión con tolerancia. Puedes inspeccionar campos, pausar, avanzar y reiniciar. Sigue siendo una simulación 2D con paredes cerradas.",
                espacio: Some(Espacio::Humo),
                estado: "Simulación GPU",
            },
            Herramienta {
                nombre: "Esfera de fuego",
                resumen: "Volumen analítico animado",
                detalle: "Marcha de rayos y campos procedurales con controles de aspecto. La forma depende de funciones matemáticas; todavía no transporta gas ni calcula combustión física.",
                espacio: Some(Espacio::Fuego),
                estado: "Procedural GPU",
            },
            Herramienta {
                nombre: "Fuego y humo 3D",
                resumen: "Volúmenes con reacción",
                detalle: "Plan: campos 3D persistentes, combustión, colisiones, sombras y render del volumen. El fuego procedural y el solver 2D actuales siguen separados.",
                espacio: None,
                estado: "Pendiente",
            },
            Herramienta {
                nombre: "Telas y cuerpos blandos",
                resumen: "Restricciones tipo XPBD",
                detalle: "Plan: telas, fijaciones, flexión, colisiones y cuerpos blandos. No hay todavía un solver equivalente a Vellum.",
                espacio: None,
                estado: "Pendiente",
            },
            Herramienta {
                nombre: "Partículas y líquidos",
                resumen: "Emisión, vida y movimiento",
                detalle: "Plan: partículas con identidad y atributos, transferencia a rejilla y líquidos. No hay todavía un editor ni solver general de estas familias.",
                espacio: None,
                estado: "Pendiente",
            },
            Herramienta {
                nombre: "Rígidos y fractura",
                resumen: "Colisiones y destrucción",
                detalle: "Plan: cuerpos rígidos, restricciones y fractura. No implementado en el estudio actual.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
    Categoria {
        nombre: "Materiales",
        resumen: "Superficies, volúmenes y aspecto",
        herramientas: &[
            Herramienta {
                nombre: "Aspecto de superficies",
                resumen: "Color y rugosidad del visor",
                detalle: "Abre el estudio procedural: selecciona Salida para editar exposición, rugosidad y colores. Es material de vista previa; todavía no hay un editor de materiales por nodos.",
                espacio: Some(Espacio::Geometria),
                estado: "Vista previa",
            },
            Herramienta {
                nombre: "Materiales por nodos",
                resumen: "Texturas, vidrio y capas",
                detalle: "Plan: grafos de material, texturas, transmisión y materiales volumétricos. El render actual no ofrece todavía estos sistemas completos.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
    Categoria {
        nombre: "Animación",
        resumen: "Tiempo, curvas y movimiento",
        herramientas: &[
            Herramienta {
                nombre: "Curvas y claves",
                resumen: "Animación editable de parámetros",
                detalle: "Plan: línea de tiempo, claves, curvas y reproducción reproducible. Hoy los laboratorios tienen fase procedural o paso de simulación, sin editor general de animación.",
                espacio: None,
                estado: "Pendiente",
            },
            Herramienta {
                nombre: "Personajes",
                resumen: "Rigging y deformación",
                detalle: "Investigación futura: esqueletos, controles y deformación de personajes. No implementado.",
                espacio: None,
                estado: "Plan futuro",
            },
        ],
    },
    Categoria {
        nombre: "Render",
        resumen: "Iluminación, calidad y salida",
        herramientas: &[
            Herramienta {
                nombre: "Render progresivo",
                resumen: "Acumulación HDR de la escena",
                detalle: "Abre el editor de objetos. El visor acumula muestras; F6 solicita un render final por lotes, con cancelación y salida PFM/PPM. Faltan GI, materiales complejos y pases de producción.",
                espacio: Some(Espacio::Objetos),
                estado: "Disponible",
            },
            Herramienta {
                nombre: "Producción y pases",
                resumen: "Secuencias, AOV y gestión de color",
                detalle: "Plan: salida profesional, pases, secuencias reproducibles y transporte de luz más completo. La exportación HDR actual es una base acotada.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
    Categoria {
        nombre: "Composición",
        resumen: "Imágenes, capas y corrección",
        herramientas: &[Herramienta {
            nombre: "Capas y composición",
            resumen: "Combinar pases e imágenes",
            detalle: "Plan: grafo de composición, máscaras, filtros y manejo de capas. No hay todavía un compositor de imágenes en TELAR.",
            espacio: None,
            estado: "Pendiente",
        }],
    },
    Categoria {
        nombre: "Datos y HILO",
        resumen: "Expresiones, atributos y diagnóstico",
        herramientas: &[
            Herramienta {
                nombre: "Inspección de humo",
                resumen: "Campos y resumen de convergencia",
                detalle: "Abre Humo 2D. Las vistas muestran los campos; D solicita un resumen de 64 B con iteraciones, residual y estado GPU. No es todavía una tabla general de atributos.",
                espacio: Some(Espacio::Humo),
                estado: "Disponible",
            },
            Herramienta {
                nombre: "Editor HILO",
                resumen: "Expresiones sobre datos",
                detalle: "HILO ya tiene IR y compilación SPIR-V. Falta el lenguaje editable para artistas, sus errores de tipos y su integración visual. No se anuncia compatibilidad VEX.",
                espacio: None,
                estado: "Pendiente",
            },
            Herramienta {
                nombre: "Caché y tareas",
                resumen: "Variantes y trabajos reproducibles",
                detalle: "Plan: cachés de simulación y dependencias entre trabajos. Los archivos actuales guardan configuraciones; no son cachés completas de los campos.",
                espacio: None,
                estado: "Pendiente",
            },
        ],
    },
];
