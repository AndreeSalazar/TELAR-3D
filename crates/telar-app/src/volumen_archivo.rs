//! .tlar v9 identifica configuración de humo 3D; no confunde v8 de humo 2D.
use std::{io::Read, path::Path};
use telar_ui::Humo;
pub(crate) fn codificar(humo: &Humo) -> Result<Vec<u8>, String> {
    let mut bytes = crate::humo_archivo::codificar(humo)?;
    bytes[4..8].copy_from_slice(&9u32.to_le_bytes());
    Ok(bytes)
}
pub(crate) fn decodificar(bytes: &[u8]) -> Result<Humo, String> {
    if bytes.len() != 56 || bytes.get(4..8) != Some(9u32.to_le_bytes().as_slice()) {
        return Err("se requiere configuración 3D .tlar v9".into());
    }
    let mut copia = bytes.to_vec();
    copia[4..8].copy_from_slice(&8u32.to_le_bytes());
    crate::humo_archivo::decodificar(&copia)
}
pub(crate) fn leer(ruta: &Path) -> Result<Humo, String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(57)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decodificar(&bytes)
}
pub(crate) fn guardar(ruta: &Path, humo: &Humo) -> Result<(), String> {
    crate::proyecto::guardar_bytes(ruta, &codificar(humo)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuracion_3d_no_se_interpreta_como_2d() -> Result<(), String> {
        let humo = Humo::default();
        let bytes = codificar(&humo)?;
        assert_eq!(decodificar(&bytes)?.0, humo.0);
        assert!(crate::humo_archivo::decodificar(&bytes).is_err());
        assert!(decodificar(&crate::humo_archivo::codificar(&humo)?).is_err());
        for i in 0..bytes.len() {
            let mut rota = bytes.clone();
            rota[i] ^= 1;
            assert!(decodificar(&rota).is_err());
        }
        Ok(())
    }
}
