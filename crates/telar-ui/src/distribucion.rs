//! Cajas calculadas con la misma hoja que pinta los controles.
use crate::Rectangulo;
use telar_estilo::{Estado, Hoja, TEMA_TELAR};

/// Distribución adaptable; bajo 720×600 conserva únicamente el visor.
#[derive(Clone, Copy)]
pub struct Distribucion {
    /// Tamaño de ventana.
    pub tamano: (u32, u32),
    /// Área interactiva y de renderizado 3D.
    pub visor: Rectangulo,
    /// Contenido arrastrable del grafo.
    pub grafo: Rectangulo,
    pub(crate) escena: Rectangulo,
    pub(crate) ajustes: Rectangulo,
    pub(crate) red: Rectangulo,
    pub(crate) completa: bool,
    pub(crate) cabecera: f32,
    pub(crate) pie: f32,
    pub(crate) margen: f32,
    pub(crate) espacio: f32,
    relleno: f32,
    nodo: [f32; 2],
    control: [f32; 3],
    boton: [f32; 2],
    muestra: [f32; 2],
}

impl Distribucion {
    /// Distribución de la hoja incorporada. En vivo se reutiliza la hoja del estudio.
    #[must_use]
    pub fn nueva(tamano: (u32, u32)) -> Self {
        Self::con_hoja(tamano, &Hoja::analizar(TEMA_TELAR).unwrap_or_default())
    }

    #[allow(clippy::cast_precision_loss, reason = "dimensiones limitadas de ventana")]
    pub(crate) fn con_hoja(tamano: (u32, u32), hoja: &Hoja) -> Self {
        let estilo = |clase| hoja.resolver(clase, Estado::default(), None);
        let base = estilo("estudio");
        let panel = estilo("panel");
        let inspector = estilo("inspector");
        let barra = estilo("barra");
        let nodo = estilo("nodo");
        let control = estilo("control");
        let boton = estilo("boton");
        let muestra = estilo("muestra");
        let visor_estilo = estilo("visor");
        let (ancho, alto) = (tamano.0.max(1) as f32, tamano.1.max(1) as f32);
        let completa = ancho >= 720.0 && alto >= 600.0;
        // Los límites geométricos mantienen operable la vista aunque la hoja sea extrema.
        let margen = base.relleno.clamp(4.0, 20.0);
        let espacio = base.espacio.clamp(4.0, 24.0);
        let cabecera = estilo("cabecera").alto.unwrap_or(52.0).clamp(40.0, 64.0);
        let pie = estilo("pie").alto.unwrap_or(32.0).clamp(24.0, 40.0);
        let alto_panel = (alto - cabecera - pie - margen * 2.0).max(1.0);
        let lateral = inspector
            .ancho
            .unwrap_or(360.0)
            .max(inspector.minimo)
            .min(inspector.maximo.max(inspector.minimo))
            .clamp(280.0, (ancho * 0.4).max(280.0));
        let escena = Rectangulo {
            x: margen,
            y: cabecera + margen,
            ancho: (ancho - lateral - margen * 2.0 - espacio).max(1.0),
            alto: alto_panel,
        };
        let barra_alto = barra.alto.unwrap_or(36.0).clamp(24.0, 40.0);
        let borde_visor = visor_estilo.grosor.clamp(0.0, 4.0);
        let visor = if completa {
            Rectangulo {
                x: escena.x + borde_visor,
                y: escena.y + barra_alto,
                ancho: escena.ancho - borde_visor * 2.0,
                alto: escena.alto - barra_alto - borde_visor,
            }
        } else {
            Rectangulo { x: 0.0, y: 0.0, ancho, alto }
        };
        let ajustes = Rectangulo {
            x: escena.x + escena.ancho + espacio,
            y: escena.y,
            ancho: lateral,
            alto: inspector.alto.unwrap_or(286.0).clamp(240.0, (alto_panel * 0.53).max(240.0)),
        };
        let red = Rectangulo {
            x: ajustes.x,
            y: ajustes.y + ajustes.alto + espacio,
            ancho: lateral,
            alto: (alto_panel - ajustes.alto - espacio).max(1.0),
        };
        let relleno = (panel.relleno + panel.grosor).clamp(8.0, 24.0);
        let grafo = Rectangulo {
            x: red.x + relleno,
            y: red.y + barra_alto + relleno,
            ancho: red.ancho - relleno * 2.0,
            alto: (red.alto - barra_alto - relleno * 2.0).max(1.0),
        };
        Self {
            tamano,
            visor,
            grafo,
            escena,
            ajustes,
            red,
            completa,
            cabecera,
            pie,
            margen,
            espacio,
            relleno,
            nodo: [
                nodo.ancho
                    .unwrap_or(152.0)
                    .max(nodo.minimo)
                    .min(nodo.maximo.max(nodo.minimo))
                    .min(grafo.ancho * 0.46)
                    .clamp(100.0, 180.0),
                nodo.alto.unwrap_or(56.0).clamp(40.0, 60.0),
            ],
            control: [
                control.alto.unwrap_or(24.0).clamp(16.0, 28.0),
                control.margen.clamp(8.0, 24.0),
                control.espacio.clamp(24.0, 32.0),
            ],
            boton: [boton.ancho.unwrap_or(98.0).clamp(72.0, 104.0), boton.alto.unwrap_or(30.0).clamp(24.0, 32.0)],
            muestra: [muestra.espacio.clamp(4.0, 12.0), muestra.alto.unwrap_or(30.0).clamp(24.0, 32.0)],
        }
    }

    /// Zona del deslizador del nodo seleccionado, separada de los botones inferiores.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "índice limitado a dos controles")]
    pub fn control(self, indice: usize) -> Rectangulo {
        let y = self.ajustes.y + 104.0 + self.control[1] + indice.min(1) as f32 * (self.control[0] + self.control[2]);
        Rectangulo {
            x: self.ajustes.x + self.relleno,
            y,
            ancho: self.ajustes.ancho - self.relleno * 2.0,
            alto: self.control[0].min((self.pausa().y - y - 2.0).max(1.0)),
        }
    }

    /// Botón de pausa.
    #[must_use]
    pub fn pausa(self) -> Rectangulo {
        Rectangulo {
            x: self.ajustes.x + self.relleno,
            y: self.ajustes.y + self.ajustes.alto - self.relleno - self.boton[1],
            ancho: self.boton[0],
            alto: self.boton[1],
        }
    }

    /// Muestra de color de vista previa.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "tres muestras")]
    pub fn muestra(self, indice: usize) -> Rectangulo {
        let boton = self.pausa();
        let inicio = boton.x + boton.ancho + self.relleno;
        let paso = (self.ajustes.x + self.ajustes.ancho - self.relleno - inicio + self.muestra[0]) / 3.0;
        Rectangulo {
            x: inicio + indice.min(2) as f32 * paso,
            y: boton.y,
            ancho: paso - self.muestra[0],
            alto: self.muestra[1],
        }
    }

    pub(crate) fn dimensiones_nodo(self) -> [f32; 2] {
        self.nodo
    }

    /// Rectángulo entero para el viewport Vulkan.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "cajas limitadas a dimensiones de ventana positivas"
    )]
    pub fn area(self) -> [u32; 4] {
        [self.visor.x as u32, self.visor.y as u32, self.visor.ancho as u32, self.visor.alto as u32]
    }
}
