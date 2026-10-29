//! Eventos del laboratorio; permite procesar cierre y edición entre pasos GPU.
use telar_ui::{DisenoObjetos, EditorHumo, EntradaNumero};
use telar_ventana::{Boton, Evento, Tecla};
pub(crate) struct Resultado {
    pub sucio: bool,
    pub accion: Option<u32>,
    pub cerrar: bool,
}
pub(crate) fn procesar(
    eventos: Vec<Evento>,
    editor: &mut EditorHumo,
    entrada: &mut crate::humo_entrada::Entrada,
    gpu: &mut telar_gpu::Gpu,
    d: DisenoObjetos,
) -> Resultado {
    let mut accion = None;
    let mut sucio = false;
    for evento in eventos {
        sucio = true;
        if editor.editando()
            && let Some(numero) = numero(evento)
        {
            editor.numero(numero);
            continue;
        }
        match evento {
            Evento::Cerrar | Evento::Tecla(Tecla::Escape) => return Resultado { sucio, accion, cerrar: true },
            Evento::Tamano { ancho, alto } => gpu.redimensionar(ancho, alto),
            Evento::Tecla(Tecla::Otra(v @ (116 | 119 | 120 | 82 | 68 | 78 | 77 | 32))) => accion = Some(v),
            Evento::Raton { x, y } => entrada.mover(editor, d, x, y),
            Evento::Boton { boton: Boton::Izquierdo, pulsado: true } => entrada.pulsar(editor, d),
            Evento::Boton { boton: Boton::Izquierdo, pulsado: false } => entrada.soltar(editor),
            Evento::Rueda(pasos) => entrada.rueda(editor, d, pasos),
            _ => {}
        }
    }

    Resultado { sucio, accion, cerrar: false }
}
fn numero(evento: Evento) -> Option<EntradaNumero> {
    match evento {
        Evento::Texto('\r' | '\n') => Some(EntradaNumero::Confirmar),
        Evento::Texto('\u{8}') => Some(EntradaNumero::Retroceso),
        Evento::Texto(c) => Some(EntradaNumero::Caracter(c)),
        Evento::Tecla(Tecla::Escape) => Some(EntradaNumero::Cancelar),
        Evento::Tecla(Tecla::Otra(46)) => Some(EntradaNumero::Borrar),
        _ => None,
    }
}
