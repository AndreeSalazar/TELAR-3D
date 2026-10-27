//! Interfaz nativa de TELAR: estado y triángulos 2D, sin Vulkan ni Windows.
//! Red fija de cuatro etapas, inspector, controles y materiales de vista previa.
#![no_std]
extern crate alloc;

mod dibujo;
mod lienzo;

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
    pub(crate) inspector: f32,
    pub(crate) completa: bool,
}

impl Distribucion {
    /// Calcula paneles y márgenes sin invadir un visor de tamaño negativo.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana limitadas por el sistema")]
    pub fn nueva(tamano: (u32, u32)) -> Self {
        let (ancho, alto) = (tamano.0.max(1) as f32, tamano.1.max(1) as f32);
        let completa = ancho >= 720.0 && alto >= 580.0;
        let inspector = ancho - 258.0;
        let visor = if completa {
            Rectangulo { x: 218.0, y: 76.0, ancho: (inspector - 234.0).max(1.0), alto: alto - 136.0 }
        } else {
            Rectangulo { x: 0.0, y: 0.0, ancho, alto }
        };
        Self { tamano, visor, inspector, completa }
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
    arrastre: Option<usize>,
}

impl Default for Estudio {
    fn default() -> Self {
        Self { seleccionado: 1, amplitud: 0.35, velocidad: 1.0, pausado: false, material: 0, arrastre: None }
    }
}

impl Estudio {
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
        for indice in 0..4 {
            #[allow(clippy::cast_precision_loss, reason = "cuatro nodos de interfaz")]
            let rect = Rectangulo { x: 20.0, y: 140.0 + indice as f32 * 92.0, ancho: 178.0, alto: 64.0 };
            if rect.contiene(x, y) {
                self.seleccionado = indice;
                return true;
            }
        }
        let inicio = diseno.inspector + 24.0;
        for (indice, altura) in [248.0, 324.0].into_iter().enumerate() {
            if (Rectangulo { x: inicio - 8.0, y: altura - 14.0, ancho: 226.0, alto: 32.0 }).contiene(x, y) {
                self.arrastre = Some(indice);
                self.mover(diseno, x);
                return true;
            }
        }
        if (Rectangulo { x: inicio, y: 373.0, ancho: 210.0, alto: 36.0 }).contiene(x, y) {
            self.pausado = !self.pausado;
            return true;
        }
        for material in 0..3 {
            #[allow(clippy::cast_precision_loss, reason = "tres materiales de vista previa")]
            let rect = Rectangulo { x: inicio + material as f32 * 72.0, y: 458.0, ancho: 62.0, alto: 42.0 };
            if rect.contiene(x, y) {
                self.material = material;
                return true;
            }
        }
        !diseno.visor.contiene(x, y)
    }

    /// Actualiza un deslizador activo y devuelve si cambió un parámetro.
    pub fn mover(&mut self, diseno: Distribucion, x: f32) -> bool {
        let Some(indice) = self.arrastre else {
            return false;
        };
        let valor = ((x - diseno.inspector - 24.0) / 210.0).clamp(0.0, 1.0);
        if indice == 0 {
            self.amplitud = valor * 0.8;
        } else {
            self.velocidad = valor * 2.0;
        }
        true
    }

    /// Termina cualquier arrastre, también al soltar fuera del control.
    pub fn soltar(&mut self) {
        self.arrastre = None;
    }
}
