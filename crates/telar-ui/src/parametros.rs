//! Configuración propia de cada etapa del estudio; no conoce el backend GPU.
use crate::Estudio;

/// Resolución lógica dentro de la capacidad reservada al abrir el estudio.
pub struct Rejilla {
    /// Puntos por lado, de dos hasta capacidad.
    pub lado: u32,
    /// Capacidad física por lado; cambiarla requiere preparar recursos nuevos.
    pub capacidad: u32,
}
/// Deformación animada.
pub struct Onda {
    /// Altura de la onda de cero a 0,8.
    pub amplitud: f32,
    /// Velocidad de fase de cero a dos.
    pub velocidad: f32,
}
/// Operación HILO sobre la altura y la normal.
pub struct Relieve {
    /// Escala vertical, de cero a tres; uno conserva la onda.
    pub factor: f32,
}
/// Presentación del resultado; no modifica geometría.
pub struct Salida {
    /// Máximo MSAA elegido: potencia de dos entre 1 y 64; puede superar el soporte real.
    pub muestras: u32,
    /// Cantidad efectiva informada por el backend, que puede ser menor.
    pub muestras_reales: u32,
    /// Exposición, de 0,25 a cuatro.
    pub exposicion: f32,
    /// Rugosidad GGX, de 0,08 a uno.
    pub rugosidad: f32,
    /// Perfil de material cobre, porcelana o esmalte azul.
    pub material: usize,
}

/// Descripción y valor de un control del nodo seleccionado.
pub struct Control {
    /// Etiqueta de interfaz.
    pub nombre: &'static str,
    /// Valor actual.
    pub valor: f32,
    /// Límite inferior.
    pub minimo: f32,
    /// Límite superior.
    pub maximo: f32,
}

impl Estudio {
    /// Ajusta resolución y límites al preparar la escena, sin reservar memoria.
    pub fn configurar_capacidad(&mut self, capacidad: u32) {
        let capacidad = capacidad.clamp(2, 1024);
        self.rejilla = Rejilla { lado: capacidad, capacidad };
    }

    /// Controles propios; los huecos no dibujados tampoco reciben eventos.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "resolución limitada a 1024")]
    pub fn control(&self, indice: usize) -> Option<Control> {
        let (nombre, valor, minimo, maximo) = match (self.seleccionado, indice) {
            (0, 0) => ("Puntos por lado", self.rejilla.lado as f32, 2.0, self.rejilla.capacidad as f32),
            (1, 0) => ("Amplitud", self.onda.amplitud, 0.0, 0.8),
            (1, 1) => ("Velocidad", self.onda.velocidad, 0.0, 2.0),
            (2, 0) => ("Factor de altura", self.relieve.factor, 0.0, 3.0),
            (3, 0) => ("Exposición", self.salida.exposicion, 0.25, 4.0),
            (3, 1) => ("Rugosidad", self.salida.rugosidad, 0.08, 1.0),
            _ => match (self.operacion_nodo(self.seleccionado)?, indice) {
                (crate::OperacionRed::Onda { amplitud, .. }, 0) => ("Amplitud", amplitud, 0.0, 0.8),
                (crate::OperacionRed::Onda { velocidad, .. }, 1) => ("Velocidad", velocidad, 0.0, 2.0),
                (crate::OperacionRed::Relieve { factor }, 0) => ("Factor de altura", factor, 0.0, 3.0),
                _ => return None,
            },
        };
        Some(Control { nombre, valor, minimo, maximo })
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "valor limitado a [2,1024]"
    )]
    pub(crate) fn ajustar(&mut self, indice: usize, fraccion: f32) -> bool {
        let Some(control) = self.control(indice) else { return false };
        let valor = control.minimo + fraccion * (control.maximo - control.minimo);
        let valor = if self.seleccionado == 0 { (valor + 0.5) as u32 as f32 } else { valor };
        self.poner_control(indice, valor)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "entero validado en [2,1024]")]
    pub(crate) fn poner_control(&mut self, indice: usize, valor: f32) -> bool {
        let Some(control) = self.control(indice) else { return false };
        if !valor.is_finite()
            || !(control.minimo..=control.maximo).contains(&valor)
            || (self.seleccionado == 0 && valor % 1.0 != 0.0)
        {
            return false;
        }
        match (self.seleccionado, indice) {
            (0, 0) => self.rejilla.lado = valor as u32,
            (1, 0) => self.onda.amplitud = valor,
            (1, 1) => self.onda.velocidad = valor,
            (2, 0) => self.relieve.factor = valor,
            (3, 0) => self.salida.exposicion = valor,
            (3, 1) => self.salida.rugosidad = valor,
            (id, indice) if id >= 4 => match (self.red.extras.get_mut(id - 4), indice) {
                (Some(crate::OperacionRed::Onda { amplitud, .. }), 0) => *amplitud = valor,
                (Some(crate::OperacionRed::Onda { velocidad, .. }), 1) => *velocidad = valor,
                (Some(crate::OperacionRed::Relieve { factor }), 0) => *factor = valor,
                _ => return false,
            },
            _ => return false,
        }
        true
    }
}
