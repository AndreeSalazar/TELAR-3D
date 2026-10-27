//! Valores resueltos y geometría de cajas `border-box`.

/// Color RGB sRGB normalizado.
pub type Color = [f32; 3];

/// Estados combinables; equivalentes a hover, checked, focus y active.
#[derive(Clone, Copy, Default)]
pub struct Estado(pub(crate) u8);

impl Estado {
    /// Construye estados del puntero y selección.
    #[must_use]
    pub const fn nuevo(hover: bool, seleccionado: bool) -> Self {
        Self(hover as u8 | (seleccionado as u8) << 1)
    }

    /// Añade o retira foco de teclado.
    #[must_use]
    pub const fn con_foco(self, foco: bool) -> Self {
        Self((self.0 & !4) | (foco as u8) << 2)
    }

    /// Añade o retira pulsación activa.
    #[must_use]
    pub const fn con_pulsacion(self, pulsado: bool) -> Self {
        Self((self.0 & !8) | (pulsado as u8) << 3)
    }
}

/// Sombra sólida desplazada; esta versión no implementa desenfoque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sombra {
    /// Desplazamiento horizontal.
    pub x: f32,
    /// Desplazamiento vertical.
    pub y: f32,
    /// Color opaco.
    pub color: Color,
}

/// Propiedades calculadas de una caja nativa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estilo {
    /// Fondo.
    pub fondo: Color,
    /// Color del texto; se hereda.
    pub color: Color,
    /// Color de borde.
    pub borde: Color,
    /// Grosor interior del borde.
    pub grosor: f32,
    /// Radio de esquinas.
    pub radio: f32,
    /// Relleno uniforme interior.
    pub relleno: f32,
    /// Margen uniforme exterior, sin colapso de márgenes.
    pub margen: f32,
    /// Separación entre cajas hermanas.
    pub espacio: f32,
    /// Ancho preferido `border-box` (None = automático).
    pub ancho: Option<f32>,
    /// Alto preferido `border-box`.
    pub alto: Option<f32>,
    /// Ancho mínimo.
    pub minimo: f32,
    /// Ancho máximo.
    pub maximo: f32,
    /// Tamaño nominal de fuente en píxeles; se hereda.
    pub fuente: f32,
    /// Altura de línea en píxeles; se hereda.
    pub linea: f32,
    /// Sombra opcional sólida.
    pub sombra: Option<Sombra>,
}

impl Default for Estilo {
    fn default() -> Self {
        Self {
            fondo: [0.08, 0.09, 0.11],
            color: [0.87, 0.89, 0.92],
            borde: [0.2, 0.22, 0.26],
            grosor: 0.0,
            radio: 0.0,
            relleno: 0.0,
            margen: 0.0,
            espacio: 0.0,
            ancho: None,
            alto: None,
            minimo: 0.0,
            maximo: f32::MAX,
            fuente: 13.0,
            linea: 20.0,
            sombra: None,
        }
    }
}

/// Rectángulo de caja en píxeles lógicos, independiente del backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caja {
    /// Posición horizontal.
    pub x: f32,
    /// Posición vertical.
    pub y: f32,
    /// Ancho.
    pub ancho: f32,
    /// Alto.
    pub alto: f32,
}

impl Caja {
    /// Reduce una caja por sus cuatro lados sin producir dimensiones negativas.
    #[must_use]
    pub fn interior(self, espacio: f32) -> Self {
        let x = espacio.max(0.0).min(self.ancho * 0.5);
        let y = espacio.max(0.0).min(self.alto * 0.5);
        Self { x: self.x + x, y: self.y + y, ancho: self.ancho - 2.0 * x, alto: self.alto - 2.0 * y }
    }

    /// Contenido de una caja exterior `border-box`, descontando borde y padding.
    #[must_use]
    pub fn contenido(self, estilo: &Estilo) -> Self {
        self.interior(estilo.grosor + estilo.relleno)
    }

    /// Comprueba el mismo rectángulo que se pinta, para interacción coherente.
    #[must_use]
    pub fn contiene(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.ancho && y < self.y + self.alto
    }
}
