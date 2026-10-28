//! Interacción del editor de objetos: controles paginados, números y acciones.
use crate::{EntradaNumero, EscenaObjetos, Rectangulo};
use alloc::{
    format,
    string::{String, ToString},
};

/// Acción explícita de archivo o render final; la UI no hace E/S.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccionEditor {
    /// Guardar escena activa.
    Guardar,
    /// Abrir escena activa.
    Abrir,
    /// Exportar render HDR con la configuración guardable.
    Renderizar,
}

/// Distribución de escena e inspector: seis campos por página.
#[derive(Clone, Copy)]
pub struct DisenoObjetos {
    /// Tamaño completo de ventana.
    pub tamano: (u32, u32),
    /// Región raytraced; acotada a 1024×768 píxeles físicos.
    pub visor: Rectangulo,
    pub(crate) panel: Rectangulo,
    pub(crate) completo: bool,
}
impl DisenoObjetos {
    /// Conserva el render acotado y deja sitio al inspector.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "dimensiones físicas de ventana")]
    pub fn nuevo(tamano: (u32, u32)) -> Self {
        let ancho = tamano.0.max(1) as f32;
        let alto = tamano.1.max(1) as f32;
        let completo = ancho >= 760.0 && alto >= 650.0;
        let mut diseno = Self {
            tamano,
            completo: ancho >= 760.0 && alto >= 650.0,
            visor: Rectangulo {
                x: 12.0,
                y: 76.0,
                ancho: (ancho - 360.0).clamp(1.0, 1024.0),
                alto: (alto - 104.0).clamp(1.0, 768.0),
            },
            panel: Rectangulo { x: ancho - 332.0, y: 76.0, ancho: 320.0, alto: (alto - 104.0).max(1.0) },
        };
        if !completo {
            diseno.visor = Rectangulo { x: 0.0, y: 0.0, ancho: ancho.min(1024.0), alto: alto.min(768.0) };
        }
        diseno
    }
    /// Región física para el backend, sin interpretar Vulkan en la UI.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "cajas acotadas positivas")]
    pub fn area(self) -> [u32; 4] {
        [self.visor.x as u32, self.visor.y as u32, self.visor.ancho as u32, self.visor.alto as u32]
    }
    /// Campo de valor del parámetro visible.
    #[must_use]
    pub fn valor(self, i: usize) -> Rectangulo {
        let rect = self.control(i);
        Rectangulo { x: rect.x + rect.ancho - 90.0, y: rect.y, ancho: 90.0, alto: 26.0 }
    }
    /// Fila del control, incluyendo etiqueta y deslizador.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "seis controles")]
    pub fn control(self, i: usize) -> Rectangulo {
        Rectangulo { x: self.panel.x + 16.0, y: 246.0 + i.min(5) as f32 * 62.0, ancho: 288.0, alto: 54.0 }
    }
    pub(crate) fn entidad(self, i: usize) -> Rectangulo {
        Rectangulo {
            x: self.panel.x + 16.0 + if i.is_multiple_of(2) { 0.0 } else { 148.0 },
            y: 112.0 + if i < 2 { 0.0 } else { 38.0 },
            ancho: 140.0,
            alto: 30.0,
        }
    }
    pub(crate) fn pagina(self, i: usize) -> Rectangulo {
        Rectangulo { x: self.panel.x + 16.0 + if i == 0 { 0.0 } else { 148.0 }, y: 198.0, ancho: 140.0, alto: 28.0 }
    }
    pub(crate) fn accion(i: usize) -> Rectangulo {
        Rectangulo {
            x: if i == 0 {
                180.0
            } else if i == 1 {
                300.0
            } else {
                420.0
            },
            y: 22.0,
            ancho: 108.0,
            alto: 30.0,
        }
    }
}

/// Editor nativo reutilizable, independiente del backend y los archivos.
pub struct EditorObjetos {
    /// Escena editable, única fuente de parámetros.
    pub escena: EscenaObjetos,
    /// Estado o error para el usuario.
    pub mensaje: String,
    /// Trabajo final en curso; no forma parte del archivo de escena.
    pub render_activo: bool,
    /// Progreso enviado por el trabajador, independiente de los mensajes de edición.
    pub estado_render: String,
    pub(crate) hoja: telar_estilo::Hoja,
    pub(crate) edicion: Option<(usize, String)>,
    reemplazar: bool,
    desliz: Option<usize>,
    accion: Option<AccionEditor>,
}
impl Default for EditorObjetos {
    fn default() -> Self {
        Self {
            escena: EscenaObjetos::default(),
            mensaje: "Escena de objetos sin guardar".into(),
            render_activo: false,
            estado_render: "Vista GPU · 1 spp · órbita / rueda".into(),
            hoja: telar_estilo::Hoja::analizar(telar_estilo::TEMA_TELAR).unwrap_or_default(),
            edicion: None,
            reemplazar: false,
            desliz: None,
            accion: None,
        }
    }
}
impl EditorObjetos {
    /// Aplica CSS al editor, con las mismas reglas que el estudio procedural.
    /// # Errors
    /// Rechaza hojas inválidas sin reemplazar la anterior.
    pub fn cargar_estilos(&mut self, css: &str) -> Result<(), String> {
        self.hoja = telar_estilo::Hoja::analizar(&format!("{}\n{css}", telar_estilo::TEMA_TELAR))?;
        Ok(())
    }
    /// Campo activo: Enter y Escape pertenecen al editor numérico.
    #[must_use]
    pub fn editando(&self) -> bool {
        self.edicion.is_some()
    }
    /// Evita recompilar durante un arrastre de parámetro.
    #[must_use]
    pub fn arrastrando(&self) -> bool {
        self.desliz.is_some()
    }
    /// Recibe entradas de texto confirmadas por la plataforma.
    pub fn numero(&mut self, entrada: EntradaNumero) -> bool {
        if self.edicion.is_none() {
            return false;
        }
        match entrada {
            EntradaNumero::Confirmar => {
                self.confirmar();
            }
            EntradaNumero::Cancelar => self.edicion = None,
            otra => {
                if let Some((_, texto)) = &mut self.edicion {
                    match otra {
                        EntradaNumero::Caracter(c)
                            if c.is_ascii_digit() || matches!(c, '.' | ',' | '+' | '-' | 'e' | 'E') =>
                        {
                            if self.reemplazar {
                                texto.clear();
                                self.reemplazar = false;
                            }
                            if texto.len() < 24 {
                                texto.push(if c == ',' { '.' } else { c });
                            }
                        }
                        EntradaNumero::Retroceso => {
                            if self.reemplazar {
                                texto.clear();
                            } else {
                                texto.pop();
                            }
                            self.reemplazar = false;
                        }
                        EntradaNumero::Borrar => {
                            texto.clear();
                            self.reemplazar = false;
                        }
                        _ => {}
                    }
                }
            }
        }
        true
    }
    fn confirmar(&mut self) -> bool {
        let Some((indice, texto)) = &self.edicion else { return true };
        let resultado = texto.parse::<f32>().map_err(|_| "número inválido").and_then(|v| self.escena.poner(*indice, v));
        match resultado {
            Ok(()) => {
                self.edicion = None;
                self.mensaje = "Parámetro aplicado".into();
                true
            }
            Err(e) => {
                self.mensaje = e.into();
                false
            }
        }
    }
    /// Pulsa controles; devuelve si la interfaz consumió el evento.
    pub fn pulsar(&mut self, diseno: DisenoObjetos, x: f32, y: f32) -> bool {
        if !diseno.completo {
            return true;
        }
        if !self.confirmar() {
            return true;
        }
        for i in 0..6 {
            let Some(control) = self.escena.control(i) else { continue };
            if diseno.valor(i).contiene(x, y) {
                self.edicion = Some((i, control.valor.to_string()));
                self.reemplazar = true;
                return true;
            }
            if diseno.control(i).contiene(x, y) {
                self.desliz = Some(i);
                self.mover(diseno, x);
                return true;
            }
        }
        for i in 0..4 {
            if diseno.entidad(i).contiene(x, y) {
                self.escena.seleccionado = i;
                return true;
            }
        }
        for i in 0..2 {
            if diseno.pagina(i).contiene(x, y) {
                self.escena.pagina = i;
                return true;
            }
        }
        for (i, accion) in
            [AccionEditor::Guardar, AccionEditor::Abrir, AccionEditor::Renderizar].into_iter().enumerate()
        {
            if DisenoObjetos::accion(i).contiene(x, y) {
                self.accion = Some(accion);
                return true;
            }
        }
        !diseno.visor.contiene(x, y)
    }
    /// Deslizador activo; la escena se valida antes de cambiar el parámetro.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "enteros positivos limitados a 16777215"
    )]
    pub fn mover(&mut self, diseno: DisenoObjetos, x: f32) -> bool {
        let Some(i) = self.desliz else { return false };
        let Some(control) = self.escena.control(i) else { return false };
        let rect = diseno.control(i);
        let fraccion = ((x - rect.x) / rect.ancho).clamp(0.0, 1.0);
        let valor = control.minimo + (control.maximo - control.minimo) * fraccion;
        let valor = if self.escena.entero(i) { (valor + 0.5) as u32 as f32 } else { valor };
        self.mensaje =
            self.escena.poner(i, valor).map_or_else(Into::into, |()| "Suelta para actualizar la escena".into());
        true
    }
    /// Termina el arrastre sin descartar el valor confirmado.
    pub fn soltar(&mut self) {
        self.desliz = None;
    }
    /// Solicita una acción, confirmando primero cualquier edición.
    pub fn solicitar(&mut self, accion: AccionEditor) {
        if self.confirmar() {
            self.accion = Some(accion);
        }
    }
    /// Consume una acción explícita para que la app haga E/S fuera del dibujo.
    pub fn tomar_accion(&mut self) -> Option<AccionEditor> {
        self.accion.take()
    }
}
