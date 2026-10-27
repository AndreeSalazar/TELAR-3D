//! Render original para TELAR: microfacetas GGX, dos luces y ambiente aproximado.
//! Genera SPIR-V; no conoce Vulkan, ventanas, archivos ni objetos de escena.
#![no_std]
extern crate alloc;
mod programa;
mod superficie;
mod tipos;
pub use programa::vertices;
pub use superficie::fragmentos;

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
