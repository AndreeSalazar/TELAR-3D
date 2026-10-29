//! Contrato de parámetros reactivos y de aspecto; no posee recursos GPU.
use crate::Control;
/// Combustión, secundarios, luz y aspecto; veintiocho escalares validados.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vfx(pub [f32; 28]);
impl Default for Vfx {
    fn default() -> Self {
        Self([
            8.0, 4.0, 0.25, 4.0, 0.8, 0.4, 2.0, 128.0, 7.0, 1.0, 5.0, 0.3, -1.0, 2.0, 1.0, 12.0, 8.0, 0.7, 3.0, 0.2,
            0.5, 0.2, 0.5, 0.1, 1.5, 8.0, 0.02, 1.0,
        ])
    }
}
impl Vfx {
    /// Control con dominio y unidades del contrato VFX.
    #[must_use]
    pub fn control(&self, i: usize) -> Option<Control> {
        let (nombre, minimo, maximo) = match i {
            0 => ("Combustible / segundo", 0.0, 20.0),
            1 => ("Tasa de reacción", 0.0, 20.0),
            2 => ("Umbral de ignición", 0.0, 10.0),
            3 => ("Calor por combustible", 0.0, 10.0),
            4 => ("Humo por combustible", 0.0, 10.0),
            5 => ("Enfriamiento / segundo", 0.0, 5.0),
            6 => ("Vida de partículas / s", 0.1, 10.0),
            7 => ("Cantidad / entero", 1.0, 128.0),
            8 => ("Semilla / entero", 0.0, 65535.0),
            9 => ("Escala de advección", 0.0, 2.0),
            10 => ("Respuesta al gas / s", 0.0, 20.0),
            11 => ("Gravedad de partículas", 0.0, 8.0),
            12 => ("Luz X", -3.0, 3.0),
            13 => ("Luz Y", 0.1, 5.0),
            14 => ("Luz Z", -3.0, 3.0),
            15 => ("Intensidad de luz", 0.0, 100.0),
            16 => ("Extinción del humo", 0.1, 30.0),
            17 => ("Albedo del humo", 0.0, 1.0),
            18 => ("Emisión del calor", 0.0, 20.0),
            19 => ("Anisotropía de fase", -0.8, 0.8),
            20 => ("Fuente X", 0.15, 0.85),
            21 => ("Fuente Y", 0.15, 0.85),
            22 => ("Fuente Z", 0.15, 0.85),
            23 => ("Calor mínimo / partículas", 0.0, 10.0),
            24 => ("Fuerza de detalle", 0.0, 5.0),
            25 => ("Frecuencia del detalle", 1.0, 20.0),
            26 => ("Disipación de combustible", 0.0, 2.0),
            27 => ("Intensidad de estelas", 0.0, 10.0),
            _ => return None,
        };
        Some(Control { nombre, minimo, maximo, valor: self.0[i] })
    }
    /// # Errors
    /// Rechaza valores no finitos, fuera de dominio y enteros fraccionarios.
    pub fn validar(&self) -> Result<(), &'static str> {
        for (i, v) in self.0.iter().enumerate() {
            let c = self.control(i).ok_or("parámetro VFX inexistente")?;
            if !v.is_finite() || !(c.minimo..=c.maximo).contains(v) || (matches!(i, 7 | 8) && v % 1.0 != 0.0) {
                return Err("parámetro VFX fuera de dominio");
            }
        }
        Ok(())
    }
    /// # Errors
    /// Rechaza índices/valores sin cambiar la configuración anterior.
    pub fn poner(&mut self, i: usize, v: f32) -> Result<(), &'static str> {
        let mut nuevo = *self;
        *nuevo.0.get_mut(i).ok_or("campo VFX inexistente")? = v;
        nuevo.validar()?;
        *self = nuevo;
        Ok(())
    }
}
