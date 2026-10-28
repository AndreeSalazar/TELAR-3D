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
        9 => ("Tolerancia absoluta", 0.0, 0.1),
        10 => ("Subpasos por cuadro", 1.0, 4.0),
        11 => ("Radio obstáculo · 0 desactiva", 0.0, 0.3),
        12 => ("Altura obstáculo", 0.31, 0.69),
        _ => ("CFL predictor · celdas", 0.1, 1.0),
    }
}
/// Inspector de los solvers MAC; los campos físicos viven en GPU.
pub struct EditorHumo {
    /// Selecciona presentación 3D; la app mantiene los campos y su formato por separado.
    pub volumen: bool,
    /// Parámetros específicos 3D, guardados aparte de los diez comunes.
    pub volumen_opciones: crate::VolumenHumo,
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
            volumen_opciones: crate::VolumenHumo::default(),
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
        let campo = if self.volumen && self.pagina == 2 { 10 + i } else { self.pagina * 6 + i };
        let cantidad = if self.pagina == 0 {
            6
        } else if self.pagina == 1 || (self.volumen && self.pagina == 2) {
            4
        } else {
            0
        };
        (i < cantidad).then_some(campo)
    }
    /// Parámetro de la página actual.
    #[must_use]
    pub fn control(&self, i: usize) -> Option<Control> {
        let campo = self.campo(i)?;
        let (nombre, minimo, maximo) = dominio(campo);
        let nombre = if self.volumen && campo == 5 { "Tiempo solicitado / cuadro" } else { nombre };
        Some(Control { nombre, minimo, maximo, valor: self.valor(campo) })
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
                let resultado = texto.parse::<f32>().map_err(|_| "número inválido").and_then(|v| {
                    if *i < 10 { self.humo.poner(*i, v) } else { self.volumen_opciones.poner(*i - 10, v) }
                });
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
        for i in 0..if self.volumen { 3 } else { 2 } {
            if self.pagina_rect(d, i).contiene(x, y) {
                self.pagina = i;
            }
        }
        for i in 0..6 {
            let Some(campo) = self.campo(i) else { continue };
            if d.valor(i).contiene(x, y) {
                self.edicion = Some((campo, self.valor(campo).to_string()));
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
        let valor = if campo == 7 {
            ((valor * 0.5 + 0.5) as u32 * 2) as f32
        } else if campo == 10 {
            (valor + 0.5) as u32 as f32
        } else {
            valor
        };
        if let Err(e) =
            if campo < 10 { self.humo.poner(campo, valor) } else { self.volumen_opciones.poner(campo - 10, valor) }
        {
            self.mensaje = format!("Error: {e}");
        }
    }
}

impl EditorHumo {
    fn valor(&self, campo: usize) -> f32 {
        if campo < 10 { self.humo.0[campo] } else { self.volumen_opciones.0[campo - 10] }
    }
    #[allow(clippy::cast_precision_loss, reason = "tres pestañas")]
    pub(crate) fn pagina_rect(&self, d: DisenoObjetos, i: usize) -> crate::Rectangulo {
        if !self.volumen {
            return d.pagina(i);
        }
        let mut rect = d.pagina(0);
        rect.x += i as f32 * 98.0;
        rect.ancho = 92.0;
        rect
    }
}
