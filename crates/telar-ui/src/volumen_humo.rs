//! Parámetros propios del tiempo y del obstáculo de humo 3D, sin recursos GPU.
/// Subpasos por cuadro, radio y altura de esfera, límite de desplazamiento CFL.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumenHumo(pub [f32; 4]);
impl Default for VolumenHumo {
    fn default() -> Self {
        Self([2.0, 0.16, 0.52, 0.5])
    }
}
impl VolumenHumo {
    /// Valida enteros, finitud y esfera contenida dentro del dominio unitario.
    /// # Errors
    /// Rechaza índices y valores sin modificar el estado anterior.
    pub fn poner(&mut self, i: usize, valor: f32) -> Result<(), &'static str> {
        let previo = *self;
        let Some(campo) = self.0.get_mut(i) else {
            return Err("campo volumétrico desconocido");
        };
        *campo = valor;
        if let Err(e) = self.validar() {
            *self = previo;
            return Err(e);
        }
        Ok(())
    }
    /// # Errors
    /// Rechaza NaN/inf, subpasos fraccionarios, esfera fuera del dominio o CFL inválido.
    pub fn validar(&self) -> Result<(), &'static str> {
        let [subpasos, radio, altura, cfl] = self.0;
        if self.0.iter().any(|v| !v.is_finite())
            || !(1.0..=4.0).contains(&subpasos)
            || subpasos % 1.0 != 0.0
            || !(0.0..=0.3).contains(&radio)
            || !(0.31..=0.69).contains(&altura)
            || !(0.1..=1.0).contains(&cfl)
        {
            return Err("tiempo/colisión fuera de dominio");
        }
        Ok(())
    }
}
