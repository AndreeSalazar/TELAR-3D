//! Datos tipográficos seguros: Windows prepara el atlas una vez, la GPU lo dibuja.

/// Máscara de glifos rasterizada por el sistema, sin manejadores nativos.
pub struct FuenteSistema {
    /// Ancho de la máscara, en píxeles.
    pub ancho: u32,
    /// Alto de la máscara.
    pub alto: u32,
    /// Tamaño de cada celda cuadrada.
    pub celda: u32,
    /// Altura solicitada de los glifos.
    pub altura: f32,
    /// Primer carácter Unicode de la tabla.
    pub primero: u32,
    /// Avance horizontal por carácter, en píxeles de atlas.
    pub avances: Vec<f32>,
    /// Cobertura de 0 a 255 por píxel; el píxel 0 es blanco para las formas sólidas.
    pub cobertura: Vec<u8>,
}

/// Rasteriza el alfabeto latino con una fuente de interfaz del sistema.
/// No dibuja ventanas ni conoce Vulkan. Se ejecuta al preparar la interfaz.
///
/// # Errors
/// Si Windows no puede crear la fuente, el bitmap o las métricas.
pub fn fuente_interfaz() -> Result<FuenteSistema, String> {
    crate::ffi::fuente::crear()
}
