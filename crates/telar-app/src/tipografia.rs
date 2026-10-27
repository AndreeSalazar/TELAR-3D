//! Adaptación de datos del sistema al modelo puro de la interfaz.
pub(crate) fn crear() -> Result<telar_ui::Fuente, String> {
    let fuente = telar_ventana::fuente_interfaz()?;
    let fuente = telar_ui::Fuente {
        ancho: fuente.ancho,
        alto: fuente.alto,
        celda: fuente.celda,
        altura: fuente.altura,
        primero: fuente.primero,
        avances: fuente.avances,
        cobertura: fuente.cobertura,
    };
    if !fuente.valida() {
        return Err("la fuente del sistema tiene métricas inválidas".to_owned());
    }
    Ok(fuente)
}
