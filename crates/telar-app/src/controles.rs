//! Entrada y cámara separadas del bucle de presentación y del diseño visual.
use nudo::Camara;
use std::time::{Duration, Instant};
use telar_ui::{Distribucion, Estudio};
use telar_ventana::{Boton, Evento, Tecla};

pub(crate) struct Controles {
    pub(crate) camara: Camara,
    pub(crate) estudio: Estudio,
    pub(crate) sucio: bool,
    pub(crate) fase: f32,
    interfaz: bool,
    arrastrando: bool,
    raton: Option<(i32, i32)>,
    tocado: Option<Instant>,
    espera_giro: Option<Duration>,
}

impl Controles {
    pub(crate) fn restaurar_interaccion(&mut self) {
        self.arrastrando = false;
        self.raton = None;
        self.tocado = Some(Instant::now());
        self.espera_giro = None;
        self.sucio = true;
    }

    pub(crate) fn nuevo(interfaz: bool) -> Self {
        Self {
            camara: Camara { distancia: if interfaz { 7.0 } else { 5.0 }, ..Camara::default() },
            estudio: Estudio::default(),
            sucio: true,
            fase: 0.0,
            interfaz,
            arrastrando: false,
            raton: None,
            tocado: None,
            espera_giro: Some(Duration::from_secs(3)),
        }
    }

    #[allow(clippy::cast_precision_loss, reason = "píxeles de la ventana y del arrastre")]
    pub(crate) fn evento(&mut self, evento: Evento, diseno: Distribucion) {
        match evento {
            Evento::Boton { boton: Boton::Izquierdo, pulsado } => {
                if pulsado {
                    let (x, y) = self.raton.unwrap_or((0, 0));
                    let consumido = self.interfaz && self.estudio.pulsar(diseno, x as f32, y as f32);
                    self.arrastrando = !consumido;
                    self.sucio |= consumido;
                } else {
                    self.estudio.soltar();
                    self.arrastrando = false;
                }
                self.tocado = Some(Instant::now());
            }
            Evento::Raton { x, y } => {
                if self.interfaz {
                    self.sucio |= self.estudio.apuntar(diseno, x as f32, y as f32);
                }
                if self.interfaz && self.estudio.mover_en(diseno, x as f32, y as f32) {
                    self.sucio = true;
                }
                if let (true, Some((anterior_x, anterior_y))) = (self.arrastrando, self.raton) {
                    self.camara.orbitar(-(x - anterior_x) as f32 * 0.3, (y - anterior_y) as f32 * 0.3);
                    self.tocado = Some(Instant::now());
                }
                self.raton = Some((x, y));
            }
            Evento::Rueda(muescas) => {
                let (x, y) = self.raton.unwrap_or((0, 0));
                if !self.interfaz || diseno.visor.contiene(x as f32, y as f32) {
                    self.camara.acercar(0.88f32.powf(muescas));
                    self.tocado = Some(Instant::now());
                }
            }
            Evento::Tecla(Tecla::Otra(32)) if self.interfaz => {
                self.estudio.pausado = !self.estudio.pausado;
                self.sucio = true;
            }
            Evento::Tecla(Tecla::Otra(9)) if self.interfaz => {
                self.estudio.soltar();
                self.estudio.seleccionado = (self.estudio.seleccionado + 1) % 4;
                self.sucio = true;
            }
            Evento::Tecla(Tecla::Otra(tecla @ (116 | 120))) if self.interfaz => {
                let accion =
                    if tecla == 116 { telar_ui::AccionProyecto::Guardar } else { telar_ui::AccionProyecto::Abrir };
                self.estudio.soltar();
                self.arrastrando = false;
                self.estudio.solicitar_proyecto(accion);
            }
            _ => {}
        }
    }

    pub(crate) fn avanzar(&mut self, dt: f32) {
        if !self.estudio.pausado {
            self.fase = (self.fase + dt * self.estudio.onda.velocidad) % 1000.0;
        }
        if !self.arrastrando && self.espera_giro.is_some_and(|espera| self.tocado.is_none_or(|t| t.elapsed() >= espera))
        {
            self.camara.orbitar(12.0 * dt, 0.0);
        }
    }
}
