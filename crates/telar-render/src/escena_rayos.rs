//! Escena pequeña de iluminación directa: datos de preparación, sin Vulkan.
use alloc::vec::Vec;

/// Triángulo opaco, difuso y de dos caras, en coordenadas de mundo.
#[derive(Clone, Copy)]
pub struct Triangulo {
    /// Tres posiciones, sin transformación pendiente.
    pub vertices: [[f32; 3]; 3],
    /// Reflectancia lineal RGB de cero a uno.
    pub color: [f32; 3],
}

/// Cámara de perspectiva: dirección frontal y dos ejes del plano de imagen.
#[derive(Clone, Copy)]
pub struct Camara {
    /// Posición de mundo.
    pub origen: [f32; 3],
    /// Dirección hacia el centro de la imagen.
    pub frente: [f32; 3],
    /// Semiancho del plano situado a una unidad del ojo.
    pub derecha: [f32; 3],
    /// Semialto del plano situado a una unidad del ojo.
    pub arriba: [f32; 3],
}

/// Trabajo independiente: se recompila al editar; no tiene historial progresivo.
pub struct Trabajo {
    /// Dimensiones de salida, máximo 512 por eje.
    pub tamano: [u32; 2],
    /// Muestras por píxel, 1..1024; máximo total de muestras 1 048 576.
    pub muestras: u32,
    /// Semilla entera, reproducible en el mismo dispositivo.
    pub semilla: u32,
    /// Cámara explícita.
    pub camara: Camara,
    /// De uno a 32 triángulos, incrustados al preparar el programa.
    pub triangulos: Vec<Triangulo>,
    /// Centro del cuadrado luminoso horizontal, orientado hacia abajo.
    pub luz: [f32; 3],
    /// Semilado positivo de la luz de área.
    pub radio_luz: f32,
    /// Radiancia de la luz, 0..100.
    pub energia: f32,
    /// Color lineal de la luz, componentes de cero a uno.
    pub color_luz: [f32; 3],
    /// Exposición de la vista sRGB; el HDR permanece lineal sin exposición.
    pub exposicion: f32,
    /// Habilita rayos de sombra entre todos los triángulos.
    pub sombras: bool,
}

impl Trabajo {
    /// Emite un kernel main con grupo 64, SSBO u32 en set=0/binding=0.
    /// Cada píxel escribe RGB f32, recuento u32 y RGB8 sRGB empaquetado: 20 B.
    /// # Errors
    /// Rechaza tamaños, presupuesto, cámara o geometría fuera del contrato.
    pub fn compilar(&self) -> Result<Vec<u32>, &'static str> {
        self.validar()?;
        Ok(crate::imagen_rayos::emitir(self))
    }

    /// Valida antes de crear recursos, incluyendo el presupuesto aritmético.
    /// # Errors
    /// No acepta datos no finitos, triángulos degenerados o bases colapsadas.
    pub fn validar(&self) -> Result<(), &'static str> {
        if !self.color_luz.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
            return Err("color de luz inválido");
        }
        if !self.exposicion.is_finite() || !(0.01..=16.0).contains(&self.exposicion) {
            return Err("exposición fuera de 0.01..16");
        }
        let [ancho, alto] = self.tamano;
        if !(1..=512).contains(&ancho)
            || !(1..=512).contains(&alto)
            || !(1..=1024).contains(&self.muestras)
            || u64::from(ancho) * u64::from(alto) * u64::from(self.muestras) > 1_048_576
            || !(1..=32).contains(&self.triangulos.len())
        {
            return Err("render fuera del presupuesto: 512 por eje, 1024 spp, 1048576 muestras totales, 32 triángulos");
        }
        let acotado = |v: &f32| v.is_finite() && v.abs() <= 100.0;
        if !self.camara.origen.iter().chain(&self.luz).all(acotado)
            || !self.radio_luz.is_finite()
            || !(0.01..=10.0).contains(&self.radio_luz)
            || !self.energia.is_finite()
            || !(0.0..=100.0).contains(&self.energia)
        {
            return Err("cámara o luz fuera del contrato");
        }
        let [frente, derecha, arriba] = [self.camara.frente, self.camara.derecha, self.camara.arriba];
        if !frente.iter().chain(&derecha).chain(&arriba).all(acotado)
            || producto(frente, cruz(derecha, arriba)).abs() < 1e-5
        {
            return Err("base de cámara colapsada o no finita");
        }
        for triangulo in &self.triangulos {
            let [a, b, c] = triangulo.vertices;
            let normal = cruz(restar(b, a), restar(c, a));
            if !triangulo.vertices.iter().flatten().all(acotado)
                || !triangulo.color.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                || producto(normal, normal) < 1e-10
            {
                return Err("triángulo degenerado o fuera del contrato");
            }
        }
        Ok(())
    }
}

pub(crate) fn restar(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    core::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn producto(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}
pub(crate) fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
