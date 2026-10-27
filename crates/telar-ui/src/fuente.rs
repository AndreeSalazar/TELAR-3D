//! Métricas tipográficas y máscara, independientes de su origen y del backend GPU.
use alloc::vec::Vec;

/// Atlas de cobertura proporcional para el alfabeto latino.
pub struct Fuente {
    /// Ancho del atlas.
    pub ancho: u32,
    /// Alto del atlas.
    pub alto: u32,
    /// Celda cuadrada de cada glifo, con dos píxeles de margen superior/izquierdo.
    pub celda: u32,
    /// Altura base a la que se rasterizó.
    pub altura: f32,
    /// Primer código Unicode de la tabla.
    pub primero: u32,
    /// Avances horizontales por glifo.
    pub avances: Vec<f32>,
    /// Máscara de cobertura, con el píxel cero blanco.
    pub cobertura: Vec<u8>,
}

impl Fuente {
    /// Comprueba dimensiones y métricas antes de componer texto.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "métricas del atlas de glifos")]
    pub fn valida(&self) -> bool {
        self.ancho > 0
            && self.alto > 0
            && self.celda > 4
            && self.altura.is_finite()
            && self.altura > 0.0
            && self.ancho.is_multiple_of(self.celda)
            && self.alto.is_multiple_of(self.celda)
            && self.ancho.checked_mul(self.alto).is_some_and(|n| n as usize == self.cobertura.len())
            && self.cobertura.first() == Some(&255)
            && !self.avances.is_empty()
            && self.primero <= u32::from('?')
            && ((u32::from('?') - self.primero) as usize) < self.avances.len()
            && self.avances.len() as u64 <= u64::from(self.ancho / self.celda) * u64::from(self.alto / self.celda)
            && self.avances.iter().all(|v| v.is_finite() && *v >= 0.0 && *v <= self.celda as f32)
    }
}
