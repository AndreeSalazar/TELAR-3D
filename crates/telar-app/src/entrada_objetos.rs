//! Eventos del editor; Enter se confirma con `WM_CHAR`, después del texto pendiente.
use telar_ui::{AccionEditor, DisenoObjetos, EditorObjetos, EntradaNumero};
use telar_ventana::{Boton, Evento, Tecla};

#[derive(Default)]
pub(crate) struct Entrada {
    raton: Option<(i32, i32)>,
    pub(crate) orbitando: bool,
    camara: Option<nudo::Camara>,
    pendiente: Option<AccionEditor>,
}
impl Entrada {
    #[allow(clippy::cast_precision_loss, reason = "píxeles del ratón")]
    pub(crate) fn evento(&mut self, editor: &mut EditorObjetos, diseno: DisenoObjetos, evento: Evento) -> bool {
        if editor.editando() {
            let numero = match evento {
                Evento::Texto('\r' | '\n') => Some(EntradaNumero::Confirmar),
                Evento::Texto('\u{8}') => Some(EntradaNumero::Retroceso),
                Evento::Texto(c) => Some(EntradaNumero::Caracter(c)),
                Evento::Tecla(Tecla::Escape) => Some(EntradaNumero::Cancelar),
                Evento::Tecla(Tecla::Otra(46)) => Some(EntradaNumero::Borrar),
                _ => None,
            };
            if let Some(numero) = numero {
                editor.numero(numero);
                return false;
            }
            if let Evento::Tecla(Tecla::Otra(tecla)) = evento {
                self.atajo(tecla);
                return false;
            }
        }
        match evento {
            Evento::Cerrar | Evento::Tecla(Tecla::Escape) => return true,
            Evento::Tecla(Tecla::Otra(tecla)) => self.atajo(tecla),
            Evento::Raton { x, y } => {
                editor.mover(diseno, x as f32);
                if let (true, Some((antes_x, antes_y)), Some(camara)) = (self.orbitando, self.raton, &mut self.camara) {
                    camara.orbitar(-(x - antes_x) as f32 * 0.3, (y - antes_y) as f32 * 0.3);
                    aplicar_camara(editor, camara);
                }
                self.raton = Some((x, y));
            }
            Evento::Boton { boton: Boton::Izquierdo, pulsado: true } => {
                let (x, y) = self.raton.unwrap_or((0, 0));
                self.orbitando = !editor.pulsar(diseno, x as f32, y as f32);
                if self.orbitando {
                    self.camara = Some(camara(editor));
                }
            }
            Evento::Boton { boton: Boton::Izquierdo, pulsado: false } => {
                self.orbitando = false;
                editor.soltar();
            }
            Evento::Rueda(pasos) => {
                let (x, y) = self.raton.unwrap_or((0, 0));
                if diseno.visor.contiene(x as f32, y as f32) && !editor.editando() {
                    let mut camara = camara(editor);
                    camara.acercar(0.88_f32.powf(pasos));
                    aplicar_camara(editor, &camara);
                }
            }
            _ => {}
        }
        false
    }
    fn atajo(&mut self, tecla: u32) {
        self.pendiente = match tecla {
            116 => Some(AccionEditor::Guardar),
            120 => Some(AccionEditor::Abrir),
            117 => Some(AccionEditor::Renderizar),
            _ => self.pendiente,
        };
    }
    pub(crate) fn terminar_lote(&mut self, editor: &mut EditorObjetos) {
        if let Some(accion) = self.pendiente.take() {
            editor.solicitar(accion);
        }
    }
}

fn camara(editor: &EditorObjetos) -> nudo::Camara {
    let v = editor.escena.valores();
    let delta: [f32; 3] = core::array::from_fn(|i| v[34 + i] - v[37 + i]);
    let distancia = delta.iter().map(|v| v * v).sum::<f32>().sqrt();
    nudo::Camara {
        objetivo: [v[37], v[38], v[39]],
        distancia,
        giro: delta[0].atan2(delta[2]).to_degrees(),
        inclinacion: (delta[1] / distancia).clamp(-1.0, 1.0).asin().to_degrees(),
        ..nudo::Camara::default()
    }
}
fn aplicar_camara(editor: &mut EditorObjetos, camara: &nudo::Camara) {
    let mut palabras = editor.escena.guardar();
    for (i, v) in camara.posicion().into_iter().enumerate() {
        palabras[34 + i] = v.to_bits();
    }
    if let Err(error) = editor.escena.restaurar(&palabras) {
        editor.mensaje = error.into();
    }
}
