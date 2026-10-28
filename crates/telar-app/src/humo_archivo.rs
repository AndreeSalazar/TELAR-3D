//! .tlar v7: ocho parámetros del volumen procedural, CRC y escritura transaccional.
use std::{io::Read, path::Path};
use telar_ui::Humo;

pub(crate) fn codificar(humo: &Humo) -> Result<Vec<u8>, String> {
    humo.validar()?;
    let mut bytes = Vec::with_capacity(48);
    for palabra in [0x5241_4c54, 7, 8, 0].into_iter().chain(humo.0.map(f32::to_bits)) {
        bytes.extend_from_slice(&palabra.to_le_bytes());
    }
    let crc = crate::proyecto::crc32(&bytes[16..]);
    bytes[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}
pub(crate) fn decodificar(bytes: &[u8]) -> Result<Humo, String> {
    if bytes.len() != 48 {
        return Err("tamaño de escena de humo inválido".into());
    }
    let palabras: Vec<_> = bytes.as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
    if palabras[..3] != [0x5241_4c54, 7, 8] || palabras[3] != crate::proyecto::crc32(&bytes[16..]) {
        return Err("cabecera/CRC de humo .tlar v7 inválidos".into());
    }
    let humo = Humo(core::array::from_fn(|i| f32::from_bits(palabras[i + 4])));
    humo.validar()?;
    Ok(humo)
}
pub(crate) fn leer(ruta: &Path) -> Result<Humo, String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(49)
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
    fn contrato_y_corrupcion() -> Result<(), String> {
        let humo = Humo::default();
        let bytes = codificar(&humo)?;
        assert_eq!(decodificar(&bytes)?.0, humo.0);
        for i in 0..bytes.len() {
            let mut rota = bytes.clone();
            rota[i] ^= 1;
            assert!(decodificar(&rota).is_err());
        }
        assert!(decodificar(&bytes[..47]).is_err());
        let mut extra = bytes;
        extra.push(0);
        assert!(decodificar(&extra).is_err());
        Ok(())
    }
}
