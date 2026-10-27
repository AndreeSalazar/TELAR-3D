//! Interfaz nativa de TELAR: estado y triángulos 2D, sin Vulkan ni Windows.
//! Red fija de cuatro etapas, inspector, controles y materiales de vista previa.
#![no_std]
extern crate alloc;

mod dibujo;
mod fuente;
mod lienzo;
pub use fuente::Fuente;

pub use telar_estilo::Caja as Rectangulo;
mod distribucion;
pub use distribucion::Distribucion;

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
    /// Perfil de vista previa: cobre, porcelana o esmalte azul.
    pub material: usize,
    arrastre: Option<Arrastre>,
    nodos: [[f32; 2]; 4],
    encima: Option<Objetivo>,
    hoja: telar_estilo::Hoja,
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
            nodos: [[0.03, 0.02], [0.97, 0.31], [0.03, 0.62], [0.97, 0.99]],
            encima: None,
            hoja: telar_estilo::Hoja::analizar(telar_estilo::TEMA_TELAR).unwrap_or_default(),
        }
    }
}

impl Estudio {
    /// Aplica personalizaciones sobre el tema incorporado; conserva la escena.
    /// Solo reemplaza la hoja anterior cuando todas las reglas son válidas.
    ///
    /// # Errors
    /// Si el CSS contiene sintaxis o propiedades no implementadas.
    pub fn cargar_estilos(&mut self, css: &str) -> Result<(), alloc::string::String> {
        self.hoja = telar_estilo::Hoja::analizar(&alloc::format!("{}\n{css}", telar_estilo::TEMA_TELAR))?;
        Ok(())
    }

    /// Calcula cajas con la hoja ya analizada, sin volver a analizar CSS por cuadro.
    #[must_use]
    pub fn distribucion(&self, tamano: (u32, u32)) -> Distribucion {
        Distribucion::con_hoja(tamano, &self.hoja)
    }

    /// Fondo sRGB del visor, para limpiar la imagen antes de dibujar la escena.
    #[must_use]
    pub fn fondo_visor(&self) -> [f32; 4] {
        let color = self.estilo("visor", false, false).fondo;
        [color[0], color[1], color[2], 1.0]
    }

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
        let [ancho, alto] = diseno.dimensiones_nodo();
        Rectangulo {
            x: diseno.grafo.x + posicion[0] * (diseno.grafo.ancho - ancho).max(0.0),
            y: diseno.grafo.y + posicion[1] * (diseno.grafo.alto - alto).max(0.0),
            ancho,
            alto,
        }
    }
    /// Color base sRGB; la app añade rugosidad y metalicidad del perfil de render.
    #[must_use]
    pub fn color(&self) -> [f32; 4] {
        color_material(self.material)
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

fn color_material(material: usize) -> [f32; 4] {
    match material {
        1 => [0.82, 0.86, 0.89, 1.0],
        2 => [0.28, 0.72, 0.82, 1.0],
        _ => [0.94, 0.49, 0.26, 1.0],
    }
}
