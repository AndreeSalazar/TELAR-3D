//! Edición numérica transitoria: confirma valores válidos sin alterar la escena antes.
use crate::{Distribucion, Estudio};
use alloc::{
    format,
    string::{String, ToString},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Campo {
    Control(usize),
    Muestras,
}

pub(crate) struct Edicion {
    pub(crate) campo: Campo,
    nodo: usize,
    pub(crate) texto: String,
    reemplazar: bool,
    pub(crate) error: bool,
}

/// Entrada abstracta del editor; no depende del teclado de una plataforma.
#[derive(Clone, Copy)]
pub enum EntradaNumero {
    /// Añadir un carácter decimal; coma se interpreta como punto.
    Caracter(char),
    /// Aplicar si el valor es válido; en otro caso mantener el editor abierto.
    Confirmar,
    /// Descartar el texto sin cambiar el último valor válido.
    Cancelar,
    /// Quitar el último carácter o la selección inicial.
    Retroceso,
    /// Vaciar el campo.
    Borrar,
}

impl Estudio {
    /// Indica que Escape/Enter deben dirigirse al campo en lugar de a la ventana.
    #[must_use]
    pub fn editando_numero(&self) -> bool {
        self.edicion.is_some()
    }

    /// Procesa una entrada, devolviendo si el editor la consumió.
    pub fn editar_numero(&mut self, entrada: EntradaNumero) -> bool {
        if self.edicion.is_none() {
            return false;
        }
        match entrada {
            EntradaNumero::Cancelar => self.edicion = None,
            EntradaNumero::Confirmar => {
                self.confirmar_numero();
            }
            otra => {
                let Some(edicion) = &mut self.edicion else { return true };
                match otra {
                    EntradaNumero::Caracter(c)
                        if c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | '+' | 'e' | 'E') =>
                    {
                        if edicion.reemplazar {
                            edicion.texto.clear();
                            edicion.reemplazar = false;
                        }
                        if edicion.texto.len() < 24 {
                            edicion.texto.push(if c == ',' { '.' } else { c });
                        }
                    }
                    EntradaNumero::Retroceso => {
                        if edicion.reemplazar {
                            edicion.texto.clear();
                        } else {
                            edicion.texto.pop();
                        }
                        edicion.reemplazar = false;
                    }
                    EntradaNumero::Borrar => {
                        edicion.texto.clear();
                        edicion.reemplazar = false;
                    }
                    _ => {}
                }
                edicion.error = false;
            }
        }
        true
    }

    pub(crate) fn confirmar_numero(&mut self) -> bool {
        let Some(edicion) = &self.edicion else { return true };
        let valido = edicion.nodo == self.seleccionado;
        let campo = edicion.campo;
        let texto = edicion.texto.clone();
        let aplicar = valido
            && match campo {
                Campo::Muestras => texto.parse::<u32>().ok().filter(|n| matches!(n, 1 | 2 | 4)).is_some_and(|n| {
                    self.salida.muestras = n;
                    true
                }),
                Campo::Control(indice) => texto.parse::<f32>().is_ok_and(|n| self.poner_control(indice, n)),
            };
        if aplicar {
            if matches!(campo, Campo::Control(_)) {
                self.marcar_red();
            }
            self.edicion = None;
        } else if let Some(edicion) = &mut self.edicion {
            edicion.error = true;
        }
        aplicar
    }

    pub(crate) fn pulsar_numero(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        let campo = (0..2)
            .find(|i| self.control(*i).is_some() && diseno.valor(*i).contiene(x, y))
            .map(Campo::Control)
            .or_else(|| (self.seleccionado == 3 && diseno.numero_muestras().contiene(x, y)).then_some(Campo::Muestras));
        let Some(campo) = campo else { return false };
        let texto = match campo {
            Campo::Control(indice) => self.control(indice).map_or_else(String::new, |c| c.valor.to_string()),
            Campo::Muestras => self.salida.muestras.to_string(),
        };
        self.soltar();
        self.edicion = Some(Edicion { campo, nodo: self.seleccionado, texto, reemplazar: true, error: false });
        true
    }

    pub(crate) fn texto_numero(&self, campo: Campo, normal: &str) -> String {
        self.edicion
            .as_ref()
            .filter(|e| e.campo == campo && e.nodo == self.seleccionado)
            .map_or_else(|| normal.into(), |e| format!("{}|", e.texto))
    }

    pub(crate) fn ayuda_numero(&self) -> Option<String> {
        self.edicion.as_ref().map(|edicion| {
            if !edicion.error {
                return "Enter: aplicar · Esc: cancelar".into();
            }
            match edicion.campo {
                Campo::Muestras => "MSAA válido: 1, 2 o 4 · Esc: cancelar".into(),
                Campo::Control(indice) => self.control(indice).map_or_else(
                    || "Parámetro no disponible · Esc: cancelar".into(),
                    |c| format!("Rango: {}..{} · Enter: aplicar", c.minimo, c.maximo),
                ),
            }
        })
    }

    pub(crate) fn estilo_numero(&self, campo: Campo) -> telar_estilo::Estilo {
        let edicion = self.edicion.as_ref().filter(|e| e.campo == campo && e.nodo == self.seleccionado);
        let mut estilo = self.estilo("valor", false, edicion.is_some());
        if edicion.is_some_and(|e| e.error) {
            estilo.borde = self.estilo("error", false, false).color;
        }
        estilo
    }
}
