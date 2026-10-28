//! Centro de herramientas: categorías, selección y acciones independientes del backend.
use crate::{CATEGORIAS, Categoria, Espacio, Herramienta, Rectangulo};
use alloc::{format, string::String};
/// Distribución de navegación, tarjetas e inspector del catálogo.
#[derive(Clone, Copy)]
pub struct DisenoCatalogo {
    /// Dimensiones físicas de ventana.
    pub tamano: (u32, u32),
    /// Debajo de 1080×700 se muestra una indicación de tamaño.
    pub completo: bool,
    pub(crate) lateral: Rectangulo,
    pub(crate) centro: Rectangulo,
    pub(crate) detalle: Rectangulo,
}
impl DisenoCatalogo {
    /// Distribución acotada; las categorías no comparten coordenadas con los editores.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana")]
    pub fn nuevo(tamano: (u32, u32)) -> Self {
        let (w, h) = (tamano.0.max(1) as f32, tamano.1.max(1) as f32);
        Self {
            tamano,
            completo: w >= 1080.0 && h >= 700.0,
            lateral: Rectangulo { x: 16.0, y: 88.0, ancho: 208.0, alto: (h - 128.0).max(1.0) },
            centro: Rectangulo { x: 240.0, y: 156.0, ancho: (w - 576.0).max(1.0), alto: (h - 196.0).max(1.0) },
            detalle: Rectangulo { x: w - 312.0, y: 88.0, ancho: 296.0, alto: (h - 128.0).max(1.0) },
        }
    }
    /// Caja de una categoría de la barra lateral.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "ocho categorías")]
    pub fn categoria(self, i: usize) -> Rectangulo {
        Rectangulo { x: 28.0, y: 148.0 + i.min(7) as f32 * 46.0, ancho: 184.0, alto: 38.0 }
    }
    /// Tarjeta de herramienta; hasta seis por categoría.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "seis tarjetas")]
    pub fn herramienta(self, i: usize) -> Rectangulo {
        let ancho = (self.centro.ancho - 14.0) * 0.5;
        Rectangulo {
            x: self.centro.x + if i.is_multiple_of(2) { 0.0 } else { ancho + 14.0 },
            y: self.centro.y + (i.min(5) / 2) as f32 * 142.0,
            ancho,
            alto: 128.0,
        }
    }
    /// Acción de apertura separada de la selección de tarjeta.
    #[must_use]
    pub fn abrir(self) -> Rectangulo {
        Rectangulo { x: self.detalle.x + 16.0, y: self.detalle.y + self.detalle.alto - 124.0, ancho: 264.0, alto: 42.0 }
    }
}
/// Selección transitoria; cambiarla no modifica ni guarda ninguna escena.
pub struct Catalogo {
    categoria: usize,
    herramienta: usize,
    /// Confirmación o error de apertura comunicado por la app.
    pub mensaje: String,
    pub(crate) hoja: telar_estilo::Hoja,
    accion: Option<Espacio>,
}
impl Default for Catalogo {
    fn default() -> Self {
        Self {
            categoria: 2,
            herramienta: 0,
            mensaje: "Selecciona una herramienta. Abrir crea otra ventana; cada editor conserva su propia escena."
                .into(),
            hoja: telar_estilo::Hoja::analizar(&format!(
                "{}\n{}",
                telar_estilo::TEMA_TELAR,
                telar_estilo::TEMA_ESTUDIO
            ))
            .unwrap_or_default(),
            accion: None,
        }
    }
}
impl Catalogo {
    /// Personaliza el centro con el mismo subconjunto CSS nativo.
    /// # Errors
    /// Rechaza CSS no válido sin cambiar la hoja anterior.
    pub fn cargar_estilos(&mut self, css: &str) -> Result<(), String> {
        self.hoja = telar_estilo::Hoja::analizar(&format!(
            "{}\n{}\n{css}",
            telar_estilo::TEMA_TELAR,
            telar_estilo::TEMA_ESTUDIO
        ))?;
        Ok(())
    }
    /// Categoría seleccionada, siempre válida.
    #[must_use]
    pub fn categoria(&self) -> &'static Categoria {
        &CATEGORIAS[self.categoria]
    }
    /// Índice seleccionado, para navegación y realce.
    #[must_use]
    pub fn indice_categoria(&self) -> usize {
        self.categoria
    }
    /// Índice de herramienta seleccionado.
    #[must_use]
    pub fn indice_herramienta(&self) -> usize {
        self.herramienta
    }
    /// Descripción del editor o de su alcance pendiente.
    #[must_use]
    pub fn herramienta(&self) -> &'static Herramienta {
        &self.categoria().herramientas[self.herramienta]
    }
    /// Cambia de contexto, seleccionando su primera herramienta.
    pub fn seleccionar_categoria(&mut self, i: usize) -> bool {
        if i >= CATEGORIAS.len() {
            return false;
        }
        self.categoria = i;
        self.herramienta = 0;
        self.accion = None;
        true
    }
    /// Solo cambia el detalle; no abre procesos automáticamente.
    pub fn seleccionar_herramienta(&mut self, i: usize) -> bool {
        if i >= self.categoria().herramientas.len() {
            return false;
        }
        self.herramienta = i;
        self.accion = None;
        true
    }
    /// Solicita apertura solo cuando existe un editor implementado.
    pub fn abrir(&mut self) -> bool {
        self.accion = self.herramienta().espacio;
        if self.accion.is_none() {
            self.mensaje = "Esta capacidad está pendiente; todavía no tiene un editor que abrir.".into();
        }
        self.accion.is_some()
    }
    /// Consume una solicitud; la app decide cómo ejecutar el editor.
    pub fn tomar_accion(&mut self) -> Option<Espacio> {
        self.accion.take()
    }
    /// Hit testing del catálogo; las herramientas pendientes pueden inspeccionarse.
    pub fn pulsar(&mut self, d: DisenoCatalogo, x: f32, y: f32) {
        if !d.completo {
            return;
        }
        if let Some(i) = (0..CATEGORIAS.len()).find(|i| d.categoria(*i).contiene(x, y)) {
            self.seleccionar_categoria(i);
            return;
        }
        if let Some(i) = (0..self.categoria().herramientas.len()).find(|i| d.herramienta(*i).contiene(x, y)) {
            self.seleccionar_herramienta(i);
            return;
        }
        if d.abrir().contiene(x, y) {
            self.abrir();
        }
    }
    /// Flechas arriba/abajo recorren las herramientas del contexto actual.
    pub fn siguiente(&mut self, adelante: bool) {
        let cantidad = self.categoria().herramientas.len();
        let indice =
            if adelante { (self.herramienta + 1) % cantidad } else { (self.herramienta + cantidad - 1) % cantidad };
        self.seleccionar_herramienta(indice);
    }
}
