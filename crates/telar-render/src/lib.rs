//! Render original: GGX, luz de área y autosombras aproximadas de rejilla residente.
//! Genera SPIR-V; no conoce Vulkan, ventanas, archivos ni objetos de escena.
#![no_std]
extern crate alloc;
mod area;
mod programa;
mod sombras;
mod superficie;
mod tipos;
pub use programa::vertices;
pub use superficie::fragmentos;

/// Luz rectangular horizontal, orientada hacia abajo; muestreo fijo de vista previa.
#[derive(Clone, Copy)]
pub struct LuzArea {
    /// Centro de la fuente en coordenadas de mundo.
    pub centro: [f32; 3],
    /// Semilado del cuadrado luminoso, en unidades de mundo.
    pub semilado: f32,
    /// Muestras estratificadas: 1 o 4. No hay acumulación temporal.
    pub muestras: u32,
    /// Pasos de sombra por segmento, de 8 a 64.
    pub pasos: u32,
    /// Autosombra aproximada de la rejilla residente, no de mallas arbitrarias.
    pub sombras: bool,
}

impl Default for LuzArea {
    fn default() -> Self {
        Self { centro: [2.5, 1.4, 1.8], semilado: 0.7, muestras: 4, pasos: 32, sombras: true }
    }
}

/// Shader de rejilla x/z en [-2,2], con luz de área y lectura de geometría residente.
/// Enlaces de solo lectura: vértices f32[] en 1, control u32[] en 3.
///
/// # Errors
/// Rechaza luces no finitas, tamaños o presupuestos fuera del contrato.
pub fn fragmentos_rejilla(luz: LuzArea) -> Result<alloc::vec::Vec<u32>, &'static str> {
    if !luz.centro.iter().all(|v| v.is_finite() && v.abs() <= 100.0)
        || !luz.semilado.is_finite()
        || !(0.01..=10.0).contains(&luz.semilado)
        || !matches!(luz.muestras, 1 | 4)
        || !(8..=64).contains(&luz.pasos)
    {
        return Err("luz de área fuera del contrato");
    }
    Ok(superficie::emitir(Some(luz)))
}

/// Material de vista previa con color lineal y respuesta microfacetada.
#[derive(Clone, Copy)]
pub struct Material {
    /// Reflectancia base lineal RGB, de cero a uno.
    pub color: [f32; 3],
    /// Rugosidad artística de cero a uno; el shader limita el mínimo a 0,08.
    pub rugosidad: f32,
    /// Fracción metálica de cero a uno.
    pub metal: f32,
}

/// Parámetros del render, independientes del backend gráfico.
pub struct Parametros {
    /// Matriz de recorte por columnas.
    pub matriz: [[f32; 4]; 4],
    /// Cámara en coordenadas de mundo.
    pub ojo: [f32; 3],
    /// Dirección de luz principal.
    pub luz: [f32; 3],
    /// Material.
    pub material: Material,
    /// Exposición positiva, hasta 16.
    pub exposicion: f32,
    /// Intensidad principal de cero a 16.
    pub intensidad: f32,
    /// Codificar sRGB en shader para un destino UNORM; false para un destino sRGB.
    pub codificar_srgb: bool,
}

impl Parametros {
    /// Empaqueta 128 bytes validados; no ejecuta trabajo por píxel en CPU.
    ///
    /// # Errors
    /// Rechaza datos no finitos, color/material fuera de rango y energía inválida.
    pub fn bloque(&self) -> Result<[f32; 32], &'static str> {
        let m = self.material;
        if !self.matriz.iter().flatten().chain(&self.ojo).chain(&self.luz).all(|v| v.is_finite())
            || !m.color.iter().chain([m.rugosidad, m.metal].iter()).all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            || !self.exposicion.is_finite()
            || !(0.01..=16.0).contains(&self.exposicion)
            || !self.intensidad.is_finite()
            || !(0.0..=16.0).contains(&self.intensidad)
        {
            return Err("parámetros de render fuera del contrato");
        }
        let mut bloque = [0.0; 32];
        for (destino, columna) in bloque[..16].as_chunks_mut::<4>().0.iter_mut().zip(self.matriz) {
            *destino = columna;
        }
        bloque[16..19].copy_from_slice(&self.ojo);
        bloque[19] = self.exposicion;
        bloque[20..23].copy_from_slice(&self.luz);
        bloque[23] = self.intensidad;
        bloque[24..27].copy_from_slice(&m.color);
        bloque[27] = m.rugosidad;
        bloque[28..32].copy_from_slice(&[m.metal, 0.035, 0.14, if self.codificar_srgb { 1.0 } else { 0.0 }]);
        Ok(bloque)
    }
}
