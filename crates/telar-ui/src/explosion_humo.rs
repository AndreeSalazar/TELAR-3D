//! Fuente pulsada propia de TELAR; parámetros, sin combustible químico ni recursos GPU.
/// Duración en segundos (0 = continuo), expansión, multiplicadores de calor/humo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionHumo(pub [f32; 4]);
impl Default for ExplosionHumo {
    fn default() -> Self {
        Self([0.0, 8.0, 5.0, 1.0])
    }
}
impl ExplosionHumo {
    /// # Errors
    /// Rechaza datos no finitos o fuera de los dominios del emisor.
    pub fn validar(&self) -> Result<(), &'static str> {
        let limites = [2.0, 20.0, 10.0, 4.0];
        if self.0.iter().zip(limites).any(|(v, max)| !v.is_finite() || !(0.0..=max).contains(v)) {
            return Err("pulso fuera de dominio");
        }
        Ok(())
    }
    /// # Errors
    /// Rechaza índices o valores sin alterar la configuración previa.
    pub fn poner(&mut self, i: usize, valor: f32) -> Result<(), &'static str> {
        let previo = *self;
        let Some(campo) = self.0.get_mut(i) else {
            return Err("campo de pulso desconocido");
        };
        *campo = valor;
        if let Err(e) = self.validar() {
            *self = previo;
            return Err(e);
        }
        Ok(())
    }
}
