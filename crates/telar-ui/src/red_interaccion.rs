//! Puertos, creación y desplazamiento del lienzo; no evalúa geometría.
use crate::{Distribucion, Estudio, Objetivo, Rectangulo, TipoNodo, lienzo::Lienzo};

impl Estudio {
    /// Conexión pendiente; Escape la cancela sin cerrar el estudio.
    #[must_use]
    pub fn conectando(&self) -> bool {
        self.red.origen.is_some()
    }

    /// Cancela el origen de conexión seleccionado.
    pub fn cancelar_conexion(&mut self) {
        self.red.origen = None;
    }

    /// Desplaza la vista del grafo sin cambiar cámara ni parámetros.
    pub fn desplazar_red(&mut self, diseno: Distribucion, muescas: f32) -> bool {
        if !muescas.is_finite() || !self.red.activa {
            return false;
        }
        let anterior = self.red.desplazamiento;
        let altura = 80.0;
        let maximo = (self.nodos.iter().map(|p| p[1]).fold(0.0, f32::max) * altura + diseno.dimensiones_nodo()[1]
            - diseno.grafo.alto)
            .max(0.0);
        self.red.desplazamiento = (anterior - muescas * 48.0).clamp(0.0, maximo);
        anterior.to_bits() != self.red.desplazamiento.to_bits()
    }

    /// Área de selección de un puerto; fuente solo produce y Salida solo consume.
    #[must_use]
    pub fn puerto(&self, diseno: Distribucion, id: usize, entrada: bool) -> Option<Rectangulo> {
        if id >= self.cantidad_nodos() || (entrada && id == 0) || (!entrada && id == 3) {
            return None;
        }
        let rect = self.nodo(diseno, id);
        Some(Rectangulo {
            x: rect.x + rect.ancho * 0.5 - 7.0,
            y: if entrada { rect.y - 7.0 } else { rect.y + rect.alto - 7.0 },
            ancho: 14.0,
            alto: 14.0,
        })
    }

    pub(crate) fn pulsar_red(&mut self, diseno: Distribucion, x: f32, y: f32) -> bool {
        for i in 0..3 {
            if diseno.accion_red(i).contiene(x, y) {
                self.soltar();
                let resultado = match i {
                    0 | 1 => self.crear_nodo(if i == 0 { TipoNodo::Onda } else { TipoNodo::Relieve }).map(|id| {
                        let rect = self.nodo(diseno, id);
                        self.red.desplazamiento += (rect.y + rect.alto - diseno.grafo.y - diseno.grafo.alto).max(0.0);
                    }),
                    _ => self.habilitar_nodo(self.seleccionado, !self.nodo_habilitado(self.seleccionado)),
                };
                self.mensaje_proyecto = resultado.map_or_else(Into::into, |()| "Red modificada".into());
                return true;
            }
        }
        if !diseno.grafo.contiene(x, y) {
            return false;
        }
        for id in (0..self.cantidad_nodos()).rev() {
            for entrada in [false, true] {
                if self.puerto(diseno, id, entrada).is_some_and(|r| r.contiene(x, y)) {
                    if entrada {
                        if let Some(desde) = self.red.origen {
                            match self.conectar_nodos(desde, id) {
                                Ok(()) => {
                                    self.red.origen = None;
                                    self.mensaje_proyecto = "Conexión aplicada".into();
                                }
                                Err(error) => self.mensaje_proyecto = error.into(),
                            }
                        }
                    } else {
                        self.red.origen = Some(id);
                        self.mensaje_proyecto = "Elige puerto de entrada".into();
                    }
                    self.seleccionado = id;
                    self.soltar();
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn dibujar_acciones_red(&self, lienzo: &mut Lienzo, diseno: Distribucion) {
        for (i, texto) in
            ["+ Onda", "+ Relieve", if self.nodo_habilitado(self.seleccionado) { "Omitir" } else { "Activar" }]
                .into_iter()
                .enumerate()
        {
            let estilo = self.estilo("calidad", self.encima == Some(Objetivo::AccionRed(i)), false);
            let rect = diseno.accion_red(i);
            lienzo.caja(rect, &estilo);
            lienzo.etiqueta(rect.contenido(&estilo), texto, &estilo);
        }
    }
}
