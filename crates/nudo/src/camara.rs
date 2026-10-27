//! La cámara orbital: gira alrededor de un punto, como el visor de Houdini.
//!
//! - **Vista:** mirar desde la posición de la cámara hacia el objetivo, con +Y
//!   hacia arriba (mano derecha: la cámara mira hacia su −Z).
//! - **Proyección en perspectiva con la profundidad invertida y sin plano
//!   lejano:** lo cercano da 1 y el infinito da 0. Así los `f32` (que tienen más
//!   precisión cerca del 0) se reparten bien en la distancia: menos parpadeo
//!   entre caras lejanas. Se usa con la prueba de profundidad «mayor que» y el
//!   búfer de profundidad borrado a 0.
//! - **Y hacia abajo en Vulkan:** la proyección invierte Y para que lo de arriba
//!   salga arriba.
//!
//! Las matrices van por **columnas** (`m[columna][fila]`), como las lee el shader.

use crate::mate::{Vec3, cruz, normalizar, punto, radianes, restar, sin_cos};

/// Una matriz 4 × 4 por columnas.
pub type Mat4 = [[f32; 4]; 4];

/// Una cámara que orbita alrededor de `objetivo`.
#[derive(Debug, Clone, PartialEq)]
pub struct Camara {
    /// El punto que mira (y alrededor del que gira).
    pub objetivo: Vec3,
    /// A qué distancia del objetivo está.
    pub distancia: f32,
    /// Grados alrededor del eje Y (0 = mirando desde +Z).
    pub giro: f32,
    /// Grados sobre el horizonte (positivo = desde arriba), entre −89 y 89.
    pub inclinacion: f32,
    /// Campo de visión vertical, en grados.
    pub campo: f32,
    /// El plano cercano: lo que esté más cerca no se ve.
    pub cerca: f32,
}

impl Default for Camara {
    fn default() -> Self {
        Self { objetivo: [0.0; 3], distancia: 5.0, giro: 30.0, inclinacion: 20.0, campo: 45.0, cerca: 0.01 }
    }
}

impl Camara {
    /// Dónde está la cámara.
    #[must_use]
    pub fn posicion(&self) -> Vec3 {
        let (s_giro, c_giro) = sin_cos(radianes(self.giro));
        let (s_incl, c_incl) = sin_cos(radianes(self.inclinacion));
        let d = self.distancia;
        [self.objetivo[0] + d * c_incl * s_giro, self.objetivo[1] + d * s_incl, self.objetivo[2] + d * c_incl * c_giro]
    }

    /// Gira la cámara: `giro` grados alrededor de Y e `inclinacion` grados hacia
    /// arriba. La inclinación se queda entre −89° y 89° (en los polos, «arriba»
    /// dejaría de estar definido).
    pub fn orbitar(&mut self, giro: f32, inclinacion: f32) {
        self.giro = (self.giro + giro) % 360.0;
        self.inclinacion = (self.inclinacion + inclinacion).clamp(-89.0, 89.0);
    }

    /// Acerca (`factor` < 1) o aleja (`factor` > 1) la cámara del objetivo.
    pub fn acercar(&mut self, factor: f32) {
        self.distancia = (self.distancia * factor).clamp(self.cerca * 2.0, 1.0e6);
    }

    /// La matriz de la vista: del mundo a la cámara.
    #[must_use]
    pub fn vista(&self) -> Mat4 {
        let ojo = self.posicion();
        let f = normalizar(restar(self.objetivo, ojo)); // hacia donde mira
        let s = normalizar(cruz(f, [0.0, 1.0, 0.0])); // derecha
        let u = cruz(s, f); // arriba
        [
            [s[0], u[0], -f[0], 0.0],
            [s[1], u[1], -f[1], 0.0],
            [s[2], u[2], -f[2], 0.0],
            [-punto(s, ojo), -punto(u, ojo), punto(f, ojo), 1.0],
        ]
    }

    /// La proyección para una imagen de proporción `aspecto` (ancho / alto).
    #[must_use]
    pub fn proyeccion(&self, aspecto: f32) -> Mat4 {
        let (s, c) = sin_cos(radianes(self.campo) / 2.0);
        let g = c / s; // 1 / tan(campo / 2)
        [
            [g / aspecto, 0.0, 0.0, 0.0],
            [0.0, -g, 0.0, 0.0], // Y hacia abajo en Vulkan
            [0.0, 0.0, 0.0, -1.0],
            [0.0, 0.0, self.cerca, 0.0],
        ]
    }

    /// Proyección por vista: del mundo a la imagen.
    #[must_use]
    pub fn matriz(&self, aspecto: f32) -> Mat4 {
        multiplicar(&self.proyeccion(aspecto), &self.vista())
    }
}

/// `a × b` (por columnas).
#[must_use]
pub fn multiplicar(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    for (c, columna) in m.iter_mut().enumerate() {
        for (f, valor) in columna.iter_mut().enumerate() {
            *valor = (0..4).map(|k| a[k][f] * b[c][k]).sum();
        }
    }
    m
}

/// `m × (p, 1)`: un punto en coordenadas de recorte `(x, y, z, w)`.
#[must_use]
pub fn transformar(m: &Mat4, p: Vec3) -> [f32; 4] {
    core::array::from_fn(|f| m[0][f] * p[0] + m[1][f] * p[1] + m[2][f] * p[2] + m[3][f])
}
