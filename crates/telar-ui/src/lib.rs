//! Interfaz nativa de TELAR: estado y triángulos 2D, sin Vulkan ni Windows.
//! Grafo de rejilla editable, inspectores numéricos y materiales de vista previa.
#![no_std]
extern crate alloc;

mod calidad;
mod dibujo_fuego;
mod fuego;
pub use fuego::{EditorFuego, Fuego};
mod escena_objetos;
pub use escena_objetos::{EscenaObjetos, PALABRAS_OBJETOS};
mod dibujo_objetos;
mod editor_objetos;
pub use editor_objetos::{AccionEditor, DisenoObjetos, EditorObjetos};
mod dibujo;
mod estado;
pub use estado::PALABRAS_ESTUDIO;
mod proyecto;
pub use proyecto::AccionProyecto;
mod estado_red;
mod fuente;
mod lienzo;
mod numero;
mod recorte;
mod red;
mod red_interaccion;
pub use red::{OperacionRed, TipoNodo};
mod parametros;
pub use fuente::Fuente;
pub use numero::EntradaNumero;
pub use parametros::{Control, Onda, Rejilla, Relieve, Salida};

pub use telar_estilo::Caja as Rectangulo;
mod distribucion;
pub use distribucion::Distribucion;

/// Estado del primer estudio visual. Los parámetros de superficie se envían a GPU.
pub struct Estudio {
    /// Etapa seleccionada: rejilla, onda, relieve o salida.
    pub seleccionado: usize,
    /// Configuración del generador de geometría.
    pub rejilla: Rejilla,
    /// Configuración de la onda.
    pub onda: Onda,
    /// Configuración del relieve.
    pub relieve: Relieve,
    /// Configuración del render.
    pub salida: Salida,
    /// Pausa la fase; la cámara continúa siendo interactiva.
    pub pausado: bool,
    arrastre: Option<Arrastre>,
    nodos: alloc::vec::Vec<[f32; 2]>,
    red: red::Red,
    encima: Option<Objetivo>,
    hoja: telar_estilo::Hoja,
    accion_proyecto: Option<AccionProyecto>,
    mensaje_proyecto: alloc::string::String,
    edicion: Option<numero::Edicion>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Objetivo {
    Nodo(usize),
    Control(usize),
    Pausa,
    Material(usize),
    Calidad(usize),
    Proyecto(usize),
    AccionRed(usize),
}

#[derive(Clone, Copy)]
enum Arrastre {
    Control { nodo: usize, indice: usize },
    Nodo { indice: usize, desplazamiento: [f32; 2] },
}

impl Default for Estudio {
    fn default() -> Self {
        Self {
            seleccionado: 1,
            rejilla: Rejilla { lado: 256, capacidad: 256 },
            onda: Onda { amplitud: 0.35, velocidad: 1.0 },
            relieve: Relieve { factor: 1.3 },
            salida: Salida { exposicion: 1.4, rugosidad: 0.3, material: 0, muestras: 2, muestras_reales: 2 },
            pausado: false,
            arrastre: None,
            nodos: alloc::vec![[0.03, 0.02], [0.97, 0.31], [0.03, 0.62], [0.97, 0.99]],
            red: red::Red::default(),
            encima: None,
            hoja: telar_estilo::Hoja::analizar(telar_estilo::TEMA_TELAR).unwrap_or_default(),
            accion_proyecto: None,
            mensaje_proyecto: "Escena sin guardar".into(),
            edicion: None,
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
            let preferido = self.seleccionado.min(self.cantidad_nodos() - 1);
            self.encima = core::iter::once(preferido)
                .chain((0..self.cantidad_nodos()).rev().filter(|i| *i != preferido))
                .find(|i| diseno.grafo.contiene(x, y) && self.nodo(diseno, *i).contiene(x, y))
                .map(Objetivo::Nodo)
                .or_else(|| (0..3).find(|i| diseno.accion_red(*i).contiene(x, y)).map(Objetivo::AccionRed))
                .or_else(|| (0..2).find(|i| diseno.proyecto(*i).contiene(x, y)).map(Objetivo::Proyecto))
                .or_else(|| {
                    (0..2)
                        .find(|i| self.control(*i).is_some() && diseno.control(*i).contiene(x, y))
                        .map(Objetivo::Control)
                })
                .or_else(|| diseno.pausa().contiene(x, y).then_some(Objetivo::Pausa))
                .or_else(|| {
                    (0..3).find(|i| self.seleccionado == 3 && diseno.calidad(*i).contiene(x, y)).map(Objetivo::Calidad)
                })
                .or_else(|| {
                    (0..3).find(|i| self.seleccionado == 3 && diseno.muestra(*i).contiene(x, y)).map(Objetivo::Material)
                });
        }
        anterior != self.encima
    }

    /// Tarjeta del nodo; la posición se conserva al redimensionar el lienzo.
    #[must_use]
    pub fn nodo(&self, diseno: Distribucion, indice: usize) -> Rectangulo {
        let posicion = self.nodos[indice.min(self.cantidad_nodos() - 1)];
        let [ancho, alto] = diseno.dimensiones_nodo();
        Rectangulo {
            x: diseno.grafo.x + posicion[0] * (diseno.grafo.ancho - ancho).max(0.0),
            y: diseno.grafo.y + posicion[1] * if self.red.activa { 80.0 } else { (diseno.grafo.alto - alto).max(1.0) }
                - self.red.desplazamiento,
            ancho,
            alto,
        }
    }
    /// Color base sRGB; la app añade rugosidad y metalicidad del perfil de render.
    #[must_use]
    pub fn color(&self) -> [f32; 4] {
        color_material(self.salida.material)
    }

    /// Procesa una pulsación. Devuelve true si el panel consume la interacción.
    pub fn pulsar(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        if !diseno.completa {
            return false;
        }
        if !self.confirmar_numero() {
            return true;
        }
        if self.pulsar_numero(diseno, x, y) {
            return true;
        }
        if self.pulsar_red(diseno, x, y) {
            return true;
        }
        for (indice, accion) in [AccionProyecto::Guardar, AccionProyecto::Abrir].into_iter().enumerate() {
            if diseno.proyecto(indice).contiene(x, y) {
                self.soltar();
                self.solicitar_proyecto(accion);
                return true;
            }
        }
        let preferido = self.seleccionado.min(self.cantidad_nodos() - 1);
        for indice in core::iter::once(preferido).chain((0..self.cantidad_nodos()).rev().filter(|i| *i != preferido)) {
            let rect = self.nodo(diseno, indice);
            if diseno.grafo.contiene(x, y) && rect.contiene(x, y) {
                self.seleccionado = indice;
                self.arrastre = Some(Arrastre::Nodo { indice, desplazamiento: [x - rect.x, y - rect.y] });
                return true;
            }
        }
        for indice in 0..2 {
            if self.control(indice).is_some() && diseno.control(indice).contiene(x, y) {
                self.arrastre = Some(Arrastre::Control { nodo: self.seleccionado, indice });
                self.mover(diseno, x);
                return true;
            }
        }
        if diseno.pausa().contiene(x, y) {
            self.pausado = !self.pausado;
            return true;
        }
        if self.seleccionado == 3 {
            for (indice, muestras) in [1, 2, 4].into_iter().enumerate() {
                if diseno.calidad(indice).contiene(x, y) {
                    self.salida.muestras = muestras;
                    return true;
                }
            }
        }
        for material in 0..3 {
            let rect = diseno.muestra(material);
            if self.seleccionado == 3 && rect.contiene(x, y) {
                self.salida.material = material;
                self.salida.rugosidad = [0.3, 0.42, 0.15][material];
                return true;
            }
        }
        !diseno.visor.contiene(x, y)
    }

    /// Actualiza un deslizador activo y devuelve si cambió un parámetro.
    pub fn mover(&mut self, diseno: Distribucion, x: f32) -> bool {
        let Some(Arrastre::Control { nodo, indice }) = self.arrastre else {
            return false;
        };
        if nodo != self.seleccionado {
            self.soltar();
            return false;
        }
        let control = diseno.control(indice);
        let valor = ((x - control.x) / control.ancho).clamp(0.0, 1.0);
        self.ajustar(indice, valor)
    }

    /// Mueve un nodo dentro del grafo o actualiza el deslizador activo.
    /// La disposición visual se edita independientemente de los puertos de conexión.
    pub fn mover_en(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        if !diseno.completa {
            self.soltar();
            return false;
        }
        if let Some(Arrastre::Nodo { indice, desplazamiento }) = self.arrastre {
            let rect = self.nodo(diseno, indice);
            self.nodos[indice] = [
                ((x - desplazamiento[0] - diseno.grafo.x) / (diseno.grafo.ancho - rect.ancho).max(1.0)).clamp(0.0, 1.0),
                ((y + self.red.desplazamiento - desplazamiento[1] - diseno.grafo.y)
                    / if self.red.activa { 80.0 } else { (diseno.grafo.alto - rect.alto).max(1.0) })
                .clamp(0.0, if self.red.activa { 16.0 } else { 1.0 }),
            ];
            true
        } else {
            self.mover(diseno, x)
        }
    }

    /// Termina cualquier arrastre, también al soltar fuera del control.
    pub fn soltar(&mut self) {
        if matches!(self.arrastre, Some(Arrastre::Control { .. })) {
            self.marcar_red();
        }
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
