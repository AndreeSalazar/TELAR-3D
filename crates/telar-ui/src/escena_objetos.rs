//! Escena acotada: dos cajas, luz, cámara y salida; datos sin recursos GPU.
use crate::Control;

/// Contrato de datos para escena .tlar v5, incluyendo selección y página.
pub const PALABRAS_OBJETOS: usize = 42;

/// Valores independientes del backend; no contienen shaders ni capacidades GPU.
#[derive(Clone, PartialEq)]
pub struct EscenaObjetos {
    pub(crate) datos: [f32; 40],
    /// Caja naranja, caja azul, luz o salida.
    pub seleccionado: usize,
    /// Página de transformación/ajustes o color/cámara.
    pub pagina: usize,
}

impl Default for EscenaObjetos {
    fn default() -> Self {
        Self {
            datos: [
                -0.85, 0.65, 0.0, 1.2, 1.3, 1.2, 0.8, 0.18, 0.07, 1.0, 0.9, 0.4, -0.55, 0.9, 0.8, 0.9, 0.08, 0.42,
                0.65, 1.0, -1.0, 4.0, 1.0, 0.65, 25.0, 1.0, 1.0, 1.0, 1.0, 1.0, 16.0, 128.0, 96.0, 1.0, 4.0, 3.0, 6.0,
                0.0, 0.5, 0.0,
            ],
            seleccionado: 0,
            pagina: 0,
        }
    }
}

impl EscenaObjetos {
    /// Estado fijo sin interacción transitoria; entero little endian a cargo de la app.
    #[must_use]
    pub fn guardar(&self) -> [u32; PALABRAS_OBJETOS] {
        core::array::from_fn(|i| match i {
            40 => u32::try_from(self.seleccionado).unwrap_or(u32::MAX),
            41 => u32::try_from(self.pagina).unwrap_or(u32::MAX),
            _ => self.datos[i].to_bits(),
        })
    }
    /// Valida antes de aplicar; no modifica el estado si un campo falla.
    /// # Errors
    /// Rechaza dominio, presupuesto, cámara degenerada y opciones desconocidas.
    pub fn restaurar(&mut self, palabras: &[u32; PALABRAS_OBJETOS]) -> Result<(), &'static str> {
        let nueva = Self {
            datos: core::array::from_fn(|i| f32::from_bits(palabras[i])),
            seleccionado: palabras[40] as usize,
            pagina: palabras[41] as usize,
        };
        nueva.validar()?;
        *self = nueva;
        Ok(())
    }
    /// Comprueba los límites de una escena completa, antes de preparar shaders.
    /// # Errors
    /// Devuelve el motivo del rechazo sin cambiar datos.
    pub fn validar(&self) -> Result<(), &'static str> {
        if self.seleccionado > 3 || self.pagina > 1 {
            return Err("selección de escena desconocida");
        }
        for (i, v) in self.datos.iter().enumerate() {
            let (_, min, max, entero) = dominio(i);
            if !v.is_finite() || !(min..=max).contains(v) || (entero && v % 1.0 != 0.0) {
                return Err("parámetro fuera de dominio");
            }
        }
        if self.datos[30] * self.datos[31] * self.datos[32] > 1_048_576.0 {
            return Err("resolución × spp supera 1048576 muestras");
        }
        let delta: [f32; 3] = core::array::from_fn(|i| self.datos[37 + i] - self.datos[34 + i]);
        if delta[0] * delta[0] + delta[2] * delta[2] < 1e-6 {
            return Err("cámara vertical o sin dirección");
        }
        Ok(())
    }
    /// Valores geométricos/de render; la app los interpreta sin acoplar la UI a Vulkan.
    #[must_use]
    pub fn valores(&self) -> &[f32; 40] {
        &self.datos
    }
    pub(crate) fn campo(&self, indice: usize) -> Option<usize> {
        if indice >= 6 {
            return None;
        }
        let campos: &[usize] = match (self.seleccionado, self.pagina) {
            (0, 0) => &[0, 1, 2, 3, 4, 5],
            (0, 1) => &[6, 7, 8, 9],
            (1, 0) => &[10, 11, 12, 13, 14, 15],
            (1, 1) => &[16, 17, 18, 19],
            (2, 0) => &[20, 21, 22, 23, 24, 28],
            (2, 1) => &[25, 26, 27],
            (3, 0) => &[29, 30, 31, 32, 33],
            (3, 1) => &[34, 35, 36, 37, 38, 39],
            _ => return None,
        };
        campos.get(indice).copied()
    }
    /// Parámetro visible en la página actual, con rango real.
    #[must_use]
    pub fn control(&self, indice: usize) -> Option<Control> {
        let campo = self.campo(indice)?;
        let (nombre, minimo, maximo, _) = dominio(campo);
        Some(Control { nombre, valor: self.datos[campo], minimo, maximo })
    }
    /// Modifica un control visible de forma transaccional.
    /// # Errors
    /// Conserva la escena anterior ante un valor inválido.
    pub fn poner(&mut self, indice: usize, valor: f32) -> Result<(), &'static str> {
        let campo = self.campo(indice).ok_or("control inexistente")?;
        let previo = self.datos[campo];
        self.datos[campo] = valor;
        if let Err(error) = self.validar() {
            self.datos[campo] = previo;
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn entero(&self, indice: usize) -> bool {
        self.campo(indice).is_some_and(|i| dominio(i).3)
    }
}

fn dominio(i: usize) -> (&'static str, f32, f32, bool) {
    let geometria = i % 10;
    if i < 20 {
        return match geometria {
            0 | 2 => (if geometria == 0 { "Posición X" } else { "Posición Z" }, -3.0, 3.0, false),
            1 => ("Posición Y", 0.0, 3.0, false),
            3..=5 => (["Tamaño X", "Tamaño Y", "Tamaño Z"][geometria - 3], 0.1, 3.0, false),
            6..=8 => (["Rojo lineal", "Verde lineal", "Azul lineal"][geometria - 6], 0.0, 1.0, false),
            _ => ("Visible (0/1)", 0.0, 1.0, true),
        };
    }
    match i {
        20 | 22 => (if i == 20 { "Luz X" } else { "Luz Z" }, -10.0, 10.0, false),
        21 => ("Luz Y", 0.1, 10.0, false),
        23 => ("Semilado luz", 0.01, 3.0, false),
        24 => ("Radiancia", 0.0, 100.0, false),
        25..=27 => (["Rojo luz", "Verde luz", "Azul luz"][i - 25], 0.0, 1.0, false),
        28 => ("Sombras (0/1)", 0.0, 1.0, true),
        29 => ("Exposición", 0.01, 16.0, false),
        30 => ("Muestras por píxel", 1.0, 1024.0, true),
        31 | 32 => (if i == 31 { "Ancho final" } else { "Alto final" }, 1.0, 512.0, true),
        33 => ("Semilla", 0.0, 16_777_215.0, true),
        34..=39 => (["Ojo X", "Ojo Y", "Ojo Z", "Objetivo X", "Objetivo Y", "Objetivo Z"][i - 34], -10.0, 10.0, false),
        _ => ("Inválido", 0.0, 0.0, false),
    }
}
