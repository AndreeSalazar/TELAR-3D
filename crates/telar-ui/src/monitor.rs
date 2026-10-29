//! Presentación CSS del monitor; recibe observaciones, no consulta hardware.
use crate::{Rectangulo, lienzo::Lienzo};
use alloc::{string::String, vec::Vec};
use telar_estilo::Estilo;
/// Monitor transitorio: no forma parte de escenas ni cachés.
#[derive(Clone, Debug, Default)]
pub struct Monitor {
    /// Líneas informativas, compuestas por la aplicación.
    pub lineas: Vec<String>,
    /// 0 verde, 1 amarillo, 2 rojo; umbrales documentados por la aplicación.
    pub estado: usize,
    /// Historial de tiempo GPU total por cuadro, en milisegundos.
    pub historial: Vec<f32>,
}
#[allow(clippy::cast_precision_loss, reason = "panel acotado a 32 líneas y 60 muestras")]
pub(crate) fn dibujar(lienzo: &mut Lienzo, monitor: Option<&Monitor>, alto: f32, panel: &Estilo, texto: &Estilo) {
    let Some(m) = monitor else {
        return;
    };
    let filas = m.lineas.len().min(32);
    let altura = (filas as f32 * 17.0 + 72.0).min(alto - 140.0);
    let caja = Rectangulo { x: 20.0, y: 84.0, ancho: 244.0, alto: altura };
    lienzo.caja(caja, panel);
    let color = [[0.3, 0.8, 0.65], [0.95, 0.7, 0.25], [0.95, 0.25, 0.25]][m.estado.min(2)];
    lienzo.rect(caja.x, caja.y, caja.ancho, 3.0, color);
    for (i, linea) in m.lineas.iter().take(filas).enumerate() {
        if i as f32 * 17.0 + 60.0 > altura {
            break;
        }
        lienzo.etiqueta(
            Rectangulo { x: caja.x + 10.0, y: caja.y + 10.0 + i as f32 * 17.0, ancho: 224.0, alto: 18.0 },
            linea,
            texto,
        );
    }
    let muestras = m.historial.iter().rev().take(60).collect::<Vec<_>>();
    for (i, ms) in muestras.iter().rev().enumerate() {
        let valor = if ms.is_finite() { ms.clamp(0.0, 50.0) / 50.0 * 32.0 } else { 0.0 };
        lienzo.rect(caja.x + 10.0 + i as f32 * 3.7, caja.y + altura - 10.0 - valor, 2.5, valor, color);
    }
}
