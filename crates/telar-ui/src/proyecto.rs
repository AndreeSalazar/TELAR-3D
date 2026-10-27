//! Acciones de archivo: la UI solicita, la app realiza la E/S.
use crate::{Distribucion, Estudio, Objetivo, Rectangulo, lienzo::Lienzo};

/// Petición explícita del usuario, consumida una sola vez por la aplicación.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccionProyecto {
    /// Guardar en la ruta activa de la sesión.
    Guardar,
    /// Abrir la ruta activa de la sesión.
    Abrir,
}

impl Estudio {
    /// Solicita E/S sin ejecutar operaciones de disco desde el crate puro.
    pub fn solicitar_proyecto(&mut self, accion: AccionProyecto) {
        self.accion_proyecto = Some(accion);
    }

    /// Extrae la solicitud pendiente, sin repetirla en fotogramas posteriores.
    pub fn tomar_accion_proyecto(&mut self) -> Option<AccionProyecto> {
        self.accion_proyecto.take()
    }

    /// Comunica el resultado de la operación; se muestra en la cabecera.
    pub fn informar_proyecto(&mut self, mensaje: &str) {
        self.mensaje_proyecto = mensaje.into();
    }

    pub(crate) fn dibujar_proyecto(&self, lienzo: &mut Lienzo, diseno: Distribucion) {
        for (indice, titulo) in ["Guardar · F5", "Abrir · F9"].into_iter().enumerate() {
            let estilo = self.estilo("calidad", self.encima == Some(Objetivo::Proyecto(indice)), false);
            let rect = diseno.proyecto(indice);
            lienzo.caja(rect, &estilo);
            lienzo.etiqueta(rect.contenido(&estilo), titulo, &estilo);
        }
        #[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana")]
        let ancho = (diseno.tamano.0 as f32 - 650.0).max(0.0);
        let rect = Rectangulo { x: 488.0, y: 17.0, ancho, alto: 22.0 };
        lienzo.etiqueta(rect, &self.mensaje_proyecto, &self.estilo("etiqueta", false, false));
    }
}
