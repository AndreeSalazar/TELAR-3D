//! Parámetros del emisor de plazas residente; no ejecuta simulación ni E/S.
use crate::Control;
/// Cantidad, vida base, impulso, gravedad, arrastre, dt, radio visual, semilla, duración y dispersión.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particulas(pub [f32; 10]);
impl Default for Particulas {
    fn default() -> Self {
        Self([128.0, 1.5, 0.8, 1.5, 0.2, 0.016, 0.012, 7.0, 0.0, 0.6])
    }
}
impl Particulas {
    /// Parámetro del inspector, con unidad y dominio explícitos.
    #[must_use]
    pub fn control(&self, i: usize) -> Option<Control> {
        let (nombre, minimo, maximo) = match i {
            0 => ("Plazas habilitadas · entero", 1.0, 128.0),
            1 => ("Vida base · segundos", 0.1, 5.0),
            2 => ("Impulso vertical", 0.05, 2.0),
            3 => ("Gravedad", 0.0, 8.0),
            4 => ("Arrastre", 0.0, 5.0),
            5 => ("Paso temporal · segundos", 0.001, 0.04),
            6 => ("Radio visual · unidades", 0.002, 0.03),
            7 => ("Semilla · entero", 0.0, 65535.0),
            8 => ("Duración · 0 continuo", 0.0, 2.0),
            9 => ("Dispersión lateral", 0.0, 1.0),
            _ => return None,
        };
        Some(Control { nombre, minimo, maximo, valor: self.0[i] })
    }
    /// # Errors
    /// Rechaza datos no finitos, dominios y enteros fraccionarios antes de enviar constantes.
    pub fn validar(&self) -> Result<(), &'static str> {
        for (i, v) in self.0.iter().enumerate() {
            let c = self.control(i).ok_or("campo de partículas desconocido")?;
            if !v.is_finite() || !(c.minimo..=c.maximo).contains(v) || (matches!(i, 0 | 7) && v % 1.0 != 0.0) {
                return Err("parámetro de partículas fuera de dominio");
            }
        }
        Ok(())
    }
    /// # Errors
    /// Rechaza índices/valores sin modificar los parámetros anteriores.
    pub fn poner(&mut self, i: usize, v: f32) -> Result<(), &'static str> {
        let previo = *self;
        let Some(campo) = self.0.get_mut(i) else {
            return Err("campo de partículas desconocido");
        };
        *campo = v;
        if let Err(e) = self.validar() {
            *self = previo;
            return Err(e);
        }
        Ok(())
    }
}
