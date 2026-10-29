//! Desglose host: llamadas medidas, sin convertir tiempos de pared en ocupación CPU.
#[allow(clippy::cast_precision_loss, reason = "hasta 120 observaciones")]
pub(crate) fn promedio(muestras: &std::collections::VecDeque<f64>) -> Option<f64> {
    (!muestras.is_empty()).then(|| muestras.iter().sum::<f64>() / muestras.len() as f64)
}
pub(crate) fn resumen(muestras: &std::collections::VecDeque<f64>) -> String {
    promedio(muestras).map_or_else(|| "Host total medio: pendiente".into(), |ms| format!("Host total medio {ms:.2} ms"))
}
#[allow(clippy::cast_precision_loss, reason = "hasta 120 observaciones por ventana")]
pub(crate) fn media(
    muestras: &std::collections::VecDeque<telar_gpu::TelemetriaHost>,
) -> Option<telar_gpu::TelemetriaHost> {
    let ultimo = muestras.back()?;
    let n = muestras.len() as f64;
    Some(telar_gpu::TelemetriaHost {
        cuadros: ultimo.cuadros,
        preparacion_ms: muestras
            .iter()
            .all(|m| m.preparacion_ms.is_some())
            .then(|| muestras.iter().filter_map(|m| m.preparacion_ms).sum::<f64>() / n),
        valla_ms: muestras.iter().map(|m| m.valla_ms).sum::<f64>() / n,
        observacion_ms: muestras.iter().map(|m| m.observacion_ms).sum::<f64>() / n,
        adquirir_ms: muestras.iter().map(|m| m.adquirir_ms).sum::<f64>() / n,
        grabar_ms: muestras.iter().map(|m| m.grabar_ms).sum::<f64>() / n,
        enviar_ms: muestras.iter().map(|m| m.enviar_ms).sum::<f64>() / n,
        presentar_ms: muestras.iter().map(|m| m.presentar_ms).sum::<f64>() / n,
    })
}
pub(crate) fn lineas(host: Option<telar_gpu::TelemetriaHost>) -> Vec<String> {
    let Some(h) = host else {
        return vec!["Desglose host: pendiente".into()];
    };
    let prep = h.preparacion_ms.map_or_else(|| "N/D".into(), |v| format!("{v:.3}"));
    vec![
        format!("Host prep/obs {prep}/{:.3} ms", h.observacion_ms),
        format!("Valla/adq {:.3}/{:.3} ms", h.valla_ms, h.adquirir_ms),
        format!("Grabar/env {:.3}/{:.3} ms", h.grabar_ms, h.enviar_ms),
        format!("Presentar {:.3} ms", h.presentar_ms),
    ]
}
pub(crate) fn csv(host: Option<telar_gpu::TelemetriaHost>) -> String {
    let Some(h) = host else {
        return ",,,,,,".into();
    };
    let prep = h.preparacion_ms.map_or_else(String::new, |v| format!("{v:.6}"));
    format!(
        "{prep},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
        h.valla_ms, h.observacion_ms, h.adquirir_ms, h.grabar_ms, h.enviar_ms, h.presentar_ms
    )
}
