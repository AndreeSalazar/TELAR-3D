//! Lo que la app le da al visor: los shaders y las constantes de cada dibujo.
//! Código puro, sin `unsafe`: cómo se empaquetan las constantes se prueba sin GPU.

use crate::eleccion::color_para;

/// Los dos shaders del visor, en SPIR-V (entrada `main`). Los de `hilo-spirv`
/// (`visor::vertices` y `visor::fragmentos`) cumplen el contrato: vértice =
/// posición (ubicación 0) + normal (ubicación 1), y 128 bytes de constantes.
#[derive(Debug, Clone, Copy)]
pub struct Sombreadores<'a> {
    /// El de vértices.
    pub vertices: &'a [u32],
    /// El de fragmentos.
    pub fragmentos: &'a [u32],
}

/// Las constantes de un dibujo: 8 `vec4`, 128 bytes, en el orden en que las
/// leen los shaders. Los colores, en sRGB (como en CSS).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constantes {
    /// Proyección × vista, por columnas (`nudo::Camara::matriz`).
    pub matriz: [[f32; 4]; 4],
    /// Hacia dónde está la luz (unitario).
    pub luz: [f32; 3],
    /// El color del objeto.
    pub color: [f32; 4],
    /// La luz ambiente que llega de arriba…
    pub cielo: [f32; 4],
    /// … y la que llega de abajo.
    pub suelo: [f32; 4],
}

impl Constantes {
    /// Las 32 `f32` tal y como van a la GPU, con los colores preparados para una
    /// imagen de `formato` (en lineal si es sRGB).
    #[must_use]
    pub fn plana(&self, formato: u32) -> [f32; 32] {
        let [x, y, z] = self.luz;
        let columnas = [self.matriz[0], self.matriz[1], self.matriz[2], self.matriz[3], [x, y, z, 0.0]];
        let colores = [self.color, self.cielo, self.suelo].map(|c| color_para(formato, c));
        let mut plana = [0.0; 32];
        for (i, v) in columnas.iter().chain(&colores).enumerate() {
            plana[i * 4..i * 4 + 4].copy_from_slice(v);
        }
        plana
    }
}
