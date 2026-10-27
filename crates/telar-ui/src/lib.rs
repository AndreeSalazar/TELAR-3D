//! Interfaz nativa de TELAR: estado y triángulos 2D, sin Vulkan ni Windows.
//! Red fija de cuatro etapas, inspector, controles y materiales de vista previa.
#![no_std]
extern crate alloc;

mod dibujo;
mod fuente;
mod lienzo;
pub use fuente::Fuente;

/// Coordenadas en píxeles lógicos de la ventana.
#[derive(Clone, Copy, Debug)]
pub struct Rectangulo {
    /// Esquina izquierda.
    pub x: f32,
    /// Esquina superior.
    pub y: f32,
    /// Ancho.
    pub ancho: f32,
    /// Alto.
    pub alto: f32,
}

impl Rectangulo {
    /// Comprueba si una posición pertenece a este rectángulo.
    #[must_use]
    pub fn contiene(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.ancho && y < self.y + self.alto
    }
}

/// Distribución adaptable; en ventanas pequeñas se conserva solo el visor.
#[derive(Clone, Copy)]
pub struct Distribucion {
    /// Tamaño de la ventana.
    pub tamano: (u32, u32),
    /// Área que puede manejar la cámara.
    pub visor: Rectangulo,
    /// Lienzo de nodos, debajo del inspector.
    pub grafo: Rectangulo,
    pub(crate) inspector: f32,
    pub(crate) completa: bool,
}

impl Distribucion {
    /// Calcula paneles y márgenes sin invadir un visor de tamaño negativo.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana limitadas por el sistema")]
    pub fn nueva(tamano: (u32, u32)) -> Self {
        let (ancho, alto) = (tamano.0.max(1) as f32, tamano.1.max(1) as f32);
        let completa = ancho >= 720.0 && alto >= 600.0;
        let inspector = ancho - (ancho * 0.34).clamp(300.0, 460.0);
        let visor = if completa {
            Rectangulo { x: 16.0, y: 84.0, ancho: (inspector - 32.0).max(1.0), alto: alto - 144.0 }
        } else {
            Rectangulo { x: 0.0, y: 0.0, ancho, alto }
        };
        let inicio_grafo = (alto * 0.5).max(330.0);
        let grafo = Rectangulo {
            x: inspector + 16.0,
            y: inicio_grafo + 38.0,
            ancho: ancho - inspector - 32.0,
            alto: (alto - inicio_grafo - 76.0).max(1.0),
        };
        Self { tamano, visor, grafo, inspector, completa }
    }

    /// Zona interactiva de amplitud (0) o velocidad (1).
    #[must_use]
    pub fn control(self, indice: usize) -> Rectangulo {
        Rectangulo {
            x: self.inspector + 24.0,
            y: [170.0, 226.0][indice.min(1)],
            ancho: self.grafo.ancho - 16.0,
            alto: 24.0,
        }
    }

    /// Botón de pausa de la animación.
    #[must_use]
    pub fn pausa(self) -> Rectangulo {
        Rectangulo { x: self.inspector + 24.0, y: 282.0, ancho: 90.0, alto: 28.0 }
    }

    /// Muestra de color de vista previa.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "tres muestras de interfaz")]
    pub fn muestra(self, indice: usize) -> Rectangulo {
        let inicio = self.inspector + 24.0;
        let paso = (self.control(0).ancho - 116.0) / 3.0;
        Rectangulo { x: inicio + 116.0 + indice.min(2) as f32 * paso, y: 282.0, ancho: paso - 8.0, alto: 28.0 }
    }

    /// Rectángulo entero para el viewport Vulkan.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "la distribución limita valores a dimensiones positivas y representables de ventana"
    )]
    pub fn area(self) -> [u32; 4] {
        [self.visor.x as u32, self.visor.y as u32, self.visor.ancho as u32, self.visor.alto as u32]
    }
}

/// Estado del primer estudio visual. Los parámetros de superficie se envían a GPU.
pub struct Estudio {
    /// Etapa seleccionada: rejilla, onda, relieve o salida.
    pub seleccionado: usize,
    /// Altura de la onda, entre 0 y 0,8.
    pub amplitud: f32,
    /// Velocidad de fase, entre 0 y 2.
    pub velocidad: f32,
    /// Pausa la fase; la cámara continúa siendo interactiva.
    pub pausado: bool,
    /// Material de vista previa: cobre, porcelana o hielo.
    pub material: usize,
    arrastre: Option<Arrastre>,
    nodos: [[f32; 2]; 4],
    encima: Option<Objetivo>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Objetivo {
    Nodo(usize),
    Control(usize),
    Pausa,
    Material(usize),
}

#[derive(Clone, Copy)]
enum Arrastre {
    Control(usize),
    Nodo { indice: usize, desplazamiento: [f32; 2] },
}

impl Default for Estudio {
    fn default() -> Self {
        Self {
            seleccionado: 1,
            amplitud: 0.35,
            velocidad: 1.0,
            pausado: false,
            material: 0,
            arrastre: None,
            nodos: [[0.03, 0.02], [0.97, 0.26], [0.03, 0.56], [0.97, 0.85]],
            encima: None,
        }
    }
}

impl Estudio {
    /// Actualiza el realce del puntero; solo pide recomponer cuando cambia su destino.
    pub fn apuntar(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        let anterior = self.encima;
        self.encima = None;
        if diseno.completa {
            let preferido = self.seleccionado.min(3);
            self.encima = core::iter::once(preferido)
                .chain((0..4).rev().filter(|i| *i != preferido))
                .find(|i| self.nodo(diseno, *i).contiene(x, y))
                .map(Objetivo::Nodo)
                .or_else(|| (0..2).find(|i| diseno.control(*i).contiene(x, y)).map(Objetivo::Control))
                .or_else(|| diseno.pausa().contiene(x, y).then_some(Objetivo::Pausa))
                .or_else(|| (0..3).find(|i| diseno.muestra(*i).contiene(x, y)).map(Objetivo::Material));
        }
        anterior != self.encima
    }

    /// Tarjeta del nodo; la posición se conserva al redimensionar el lienzo.
    #[must_use]
    pub fn nodo(&self, diseno: Distribucion, indice: usize) -> Rectangulo {
        let posicion = self.nodos[indice.min(3)];
        let ancho = (diseno.grafo.ancho * 0.42).clamp(104.0, 144.0);
        Rectangulo {
            x: diseno.grafo.x + posicion[0] * (diseno.grafo.ancho - ancho).max(0.0),
            y: diseno.grafo.y + posicion[1] * (diseno.grafo.alto - 46.0).max(0.0),
            ancho,
            alto: 46.0,
        }
    }
    /// Color de vista previa en sRGB; todavía no es un material PBR.
    #[must_use]
    pub fn color(&self) -> [f32; 4] {
        match self.material {
            1 => [0.82, 0.86, 0.89, 1.0],
            2 => [0.28, 0.72, 0.82, 1.0],
            _ => [0.94, 0.49, 0.26, 1.0],
        }
    }

    /// Procesa una pulsación. Devuelve true si el panel consume la interacción.
    pub fn pulsar(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        if !diseno.completa {
            return false;
        }
        let preferido = self.seleccionado.min(3);
        for indice in core::iter::once(preferido).chain((0..4).rev().filter(|i| *i != preferido)) {
            let rect = self.nodo(diseno, indice);
            if rect.contiene(x, y) {
                self.seleccionado = indice;
                self.arrastre = Some(Arrastre::Nodo { indice, desplazamiento: [x - rect.x, y - rect.y] });
                return true;
            }
        }
        for indice in 0..2 {
            if diseno.control(indice).contiene(x, y) {
                self.arrastre = Some(Arrastre::Control(indice));
                self.mover(diseno, x);
                return true;
            }
        }
        if diseno.pausa().contiene(x, y) {
            self.pausado = !self.pausado;
            return true;
        }
        for material in 0..3 {
            let rect = diseno.muestra(material);
            if rect.contiene(x, y) {
                self.material = material;
                return true;
            }
        }
        !diseno.visor.contiene(x, y)
    }

    /// Actualiza un deslizador activo y devuelve si cambió un parámetro.
    pub fn mover(&mut self, diseno: Distribucion, x: f32) -> bool {
        let Some(Arrastre::Control(indice)) = self.arrastre else {
            return false;
        };
        let control = diseno.control(indice);
        let valor = ((x - control.x) / control.ancho).clamp(0.0, 1.0);
        if indice == 0 {
            self.amplitud = valor * 0.8;
        } else {
            self.velocidad = valor * 2.0;
        }
        true
    }

    /// Mueve un nodo dentro del grafo o actualiza el deslizador activo.
    /// Las conexiones y el orden de ejecución permanecen fijos en esta entrega.
    pub fn mover_en(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        if !diseno.completa {
            self.soltar();
            return false;
        }
        if let Some(Arrastre::Nodo { indice, desplazamiento }) = self.arrastre {
            let rect = self.nodo(diseno, indice);
            self.nodos[indice] = [
                ((x - desplazamiento[0] - diseno.grafo.x) / (diseno.grafo.ancho - rect.ancho).max(1.0)).clamp(0.0, 1.0),
                ((y - desplazamiento[1] - diseno.grafo.y) / (diseno.grafo.alto - rect.alto).max(1.0)).clamp(0.0, 1.0),
            ];
            true
        } else {
            self.mover(diseno, x)
        }
    }

    /// Termina cualquier arrastre, también al soltar fuera del control.
    pub fn soltar(&mut self) {
        self.arrastre = None;
    }
}
