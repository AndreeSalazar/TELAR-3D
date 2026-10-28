//! Parámetros del humo MAC 2D/3D y edición numérica; sin recursos GPU.
use crate::{Control, DisenoObjetos, EntradaNumero};
use alloc::{
    format,
    string::{String, ToString},
};

/// Radio, emisión, flotación, viento, disipación, dt, exposición, máximo de iteraciones y tolerancias.
#[derive(Clone, PartialEq)]
pub struct Humo(pub [f32; 10]);
impl Default for Humo {
    fn default() -> Self {
        Self([0.065, 3.0, 2.0, 0.08, 0.15, 0.016, 1.0, 64.0, 0.02, 0.005])
    }
}
impl Humo {
    /// Valida antes de aplicar, guardar o enviar a GPU.
    /// # Errors
    /// Rechaza valores no finitos, dominios y enteros fraccionarios.
    pub fn validar(&self) -> Result<(), &'static str> {
        for (i, valor) in self.0.iter().enumerate() {
            let (_, min, max) = dominio(i);
            if !valor.is_finite() || !(min..=max).contains(valor) || (i == 7 && valor % 2.0 != 0.0) {
                return Err("parámetro de humo fuera de dominio");
            }
        }
        Ok(())
    }
    /// Cambia un campo de forma transaccional.
    /// # Errors
    /// Rechaza índices o valores inválidos sin alterar el estado.
    pub fn poner(&mut self, i: usize, valor: f32) -> Result<(), &'static str> {
        if i >= 10 {
            return Err("campo de humo inexistente");
        }
        let previo = self.0[i];
        self.0[i] = valor;
        if let Err(e) = self.validar() {
            self.0[i] = previo;
            return Err(e);
        }
        Ok(())
    }
}
fn dominio(i: usize) -> (&'static str, f32, f32) {
    match i {
        0 => ("Radio emisor / dominio", 0.02, 0.18),
        1 => ("Emisión densidad / calor", 0.0, 10.0),
        2 => ("Flotación", 0.0, 8.0),
        3 => ("Viento / calor", -1.0, 1.0),
        4 => ("Disipación de densidad", 0.0, 2.0),
        5 => ("Paso temporal · segundos", 0.001, 0.04),
        6 => ("Exposición", 0.1, 8.0),
        7 => ("Máximo presión · par", 2.0, 128.0),
        8 => ("Tolerancia relativa", 0.0, 0.5),
        _ => ("Tolerancia absoluta", 0.0, 0.1),
    }
}
/// Inspector de los solvers MAC; los campos físicos viven en GPU.
pub struct EditorHumo {
    /// Selecciona presentación 3D; la app mantiene los campos y su formato por separado.
    pub volumen: bool,
    /// Avanza el estado físico un paso fijo por fotograma dibujado.
    pub animando: bool,
    /// Valores guardables, independientes de la vista.
    pub humo: Humo,
    /// Densidad, calor, rapidez o divergencia residual.
    pub vista: usize,
    /// Fuente o integración.
    pub pagina: usize,
    /// Estado de archivo/edición.
    pub mensaje: String,
    pub(crate) hoja: telar_estilo::Hoja,
    pub(crate) edicion: Option<(usize, String)>,
    reemplazar: bool,
    desliz: Option<usize>,
}
impl Default for EditorHumo {
    fn default() -> Self {
        Self {
            animando: true,
            volumen: false,
            humo: Humo::default(),
            vista: 0,
            pagina: 0,
            mensaje:
                "Borde GPU: verde = tolerancia · amarillo = límite · rojo = inválido · D: datos · Espacio/N/R · F5/F9"
                    .into(),
            hoja: telar_estilo::Hoja::analizar(telar_estilo::TEMA_TELAR).unwrap_or_default(),
            edicion: None,
            reemplazar: false,
            desliz: None,
        }
    }
}
impl EditorHumo {
    pub(crate) fn campo(&self, i: usize) -> Option<usize> {
        let campo = self.pagina * 6 + i;
        (i < 6 && campo < 10).then_some(campo)
    }
    /// Parámetro de la página actual.
    #[must_use]
    pub fn control(&self, i: usize) -> Option<Control> {
        let campo = self.campo(i)?;
        let (nombre, minimo, maximo) = dominio(campo);
        Some(Control { nombre, minimo, maximo, valor: self.humo.0[campo] })
    }
    /// Campo con foco numérico.
    #[must_use]
    pub fn editando(&self) -> bool {
        self.edicion.is_some()
    }
    /// Termina un arrastre.
    pub fn soltar(&mut self) {
        self.desliz = None;
    }
    /// Introduce texto; Enter confirma después de `WM_CHAR`, conservando valores inválidos fuera de la escena.
    pub fn numero(&mut self, entrada: EntradaNumero) {
        let Some((i, texto)) = &mut self.edicion else { return };
        match entrada {
            EntradaNumero::Cancelar => self.edicion = None,
            EntradaNumero::Confirmar => {
                let resultado =
                    texto.parse::<f32>().map_err(|_| "número inválido").and_then(|v| self.humo.poner(*i, v));
                match resultado {
                    Ok(()) => {
                        self.edicion = None;
                        self.mensaje = "Parámetro aplicado".into();
                    }
                    Err(e) => self.mensaje = e.into(),
                }
            }
            EntradaNumero::Caracter(c) if c.is_ascii_digit() || matches!(c, '.' | ',' | '+' | '-' | 'e' | 'E') => {
                if self.reemplazar {
                    texto.clear();
                    self.reemplazar = false;
                }
                if texto.len() < 24 {
                    texto.push(if c == ',' { '.' } else { c });
                }
            }
            EntradaNumero::Retroceso | EntradaNumero::Borrar => {
                if self.reemplazar || matches!(entrada, EntradaNumero::Borrar) {
                    texto.clear();
                } else {
                    texto.pop();
                }
                self.reemplazar = false;
            }
            EntradaNumero::Caracter(_) => {}
        }
    }
    /// Valores y deslizadores. La ventana procesa F5/F9 para E/S.
    pub fn pulsar(&mut self, d: DisenoObjetos, x: f32, y: f32) {
        self.numero(EntradaNumero::Confirmar);
        if self.editando() || !d.completo {
            return;
        }
        for i in 0..4 {
            if d.entidad(i).contiene(x, y) {
                self.vista = i;
            }
        }
        for i in 0..2 {
            if d.pagina(i).contiene(x, y) {
                self.pagina = i;
            }
        }
        for i in 0..6 {
            let Some(campo) = self.campo(i) else { continue };
            if d.valor(i).contiene(x, y) {
                self.edicion = Some((campo, self.humo.0[campo].to_string()));
                self.reemplazar = true;
                return;
            }
            if d.control(i).contiene(x, y) {
                self.desliz = Some(i);
                self.mover(d, x);
                return;
            }
        }
    }
    /// Actualiza el deslizador activo con límites del contrato.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "iteraciones pares <=128"
    )]
    pub fn mover(&mut self, d: DisenoObjetos, x: f32) {
        let Some(i) = self.desliz else { return };
        let Some(campo) = self.campo(i) else { return };
        let (_, min, max) = dominio(campo);
        let rect = d.control(i);
        let valor = min + (max - min) * ((x - rect.x) / rect.ancho).clamp(0.0, 1.0);
        let valor = if campo == 7 { ((valor * 0.5 + 0.5) as u32 * 2) as f32 } else { valor };
        if let Err(e) = self.humo.poner(campo, valor) {
            self.mensaje = format!("Error: {e}");
        }
    }
}
