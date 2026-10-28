//! .tlar v11: parámetros físicos, tiempo, esfera y pulso; migra v9/v10.
use std::{io::Read, path::Path};
use telar_ui::{ExplosionHumo, Humo, VolumenHumo};
pub(crate) fn codificar(humo: &Humo, opciones: &VolumenHumo, explosion: &ExplosionHumo) -> Result<Vec<u8>, String> {
    humo.validar()?;
    opciones.validar()?;
    explosion.validar()?;
    let mut bytes = Vec::with_capacity(88);
    for palabra in [0x5241_4c54, 11, 18, 0]
        .into_iter()
        .chain(humo.0.into_iter().chain(opciones.0).chain(explosion.0).map(f32::to_bits))
    {
        bytes.extend_from_slice(&palabra.to_le_bytes());
    }
    let crc = crate::proyecto::crc32(&bytes[16..]);
    bytes[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}
pub(crate) fn decodificar(bytes: &[u8]) -> Result<(Humo, VolumenHumo, ExplosionHumo), String> {
    if bytes.len() == 56 && bytes.get(4..8) == Some(9u32.to_le_bytes().as_slice()) {
        let mut copia = bytes.to_vec();
        copia[4..8].copy_from_slice(&8u32.to_le_bytes());
        return Ok((
            crate::humo_archivo::decodificar(&copia)?,
            VolumenHumo([1.0, 0.0, 0.52, 0.5]),
            ExplosionHumo::default(),
        ));
    }
    if ![72, 88].contains(&bytes.len()) {
        return Err("se requiere configuración 3D .tlar v9/v10/v11".into());
    }
    let palabras: Vec<_> = bytes.as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
    let (version, cantidad) = if bytes.len() == 72 { (10, 14) } else { (11, 18) };
    if palabras[..3] != [0x5241_4c54, version, cantidad] || palabras[3] != crate::proyecto::crc32(&bytes[16..]) {
        return Err("cabecera/CRC de humo 3D inválidos".into());
    }
    let mut humo = Humo::default();
    let mut opciones = VolumenHumo::default();
    let mut explosion = ExplosionHumo::default();
    for (destino, valor) in humo.0.iter_mut().chain(&mut opciones.0).chain(&mut explosion.0).zip(&palabras[4..]) {
        *destino = f32::from_bits(*valor);
    }
    humo.validar()?;
    opciones.validar()?;
    explosion.validar()?;
    Ok((humo, opciones, explosion))
}
pub(crate) fn leer(ruta: &Path) -> Result<(Humo, VolumenHumo, ExplosionHumo), String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(89)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decodificar(&bytes)
}
pub(crate) fn guardar(
    ruta: &Path,
    humo: &Humo,
    opciones: &VolumenHumo,
    explosion: &ExplosionHumo,
) -> Result<(), String> {
    crate::proyecto::guardar_bytes(ruta, &codificar(humo, opciones, explosion)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuracion_versionada_crc_y_migracion() -> Result<(), String> {
        let humo = Humo::default();
        let opciones = VolumenHumo::default();
        let explosion = ExplosionHumo([0.35, 8.0, 5.0, 1.0]);
        let bytes = codificar(&humo, &opciones, &explosion)?;
        assert_eq!(decodificar(&bytes)?.2, explosion);
        assert_eq!(decodificar(&bytes)?.0.0, humo.0);
        assert_eq!(decodificar(&bytes)?.1, opciones);
        let mut v10 = bytes[..72].to_vec();
        v10[4..8].copy_from_slice(&10u32.to_le_bytes());
        v10[8..12].copy_from_slice(&14u32.to_le_bytes());
        let crc = crate::proyecto::crc32(&v10[16..]);
        v10[12..16].copy_from_slice(&crc.to_le_bytes());
        assert_eq!(decodificar(&v10)?.2, ExplosionHumo::default());
        assert!(crate::humo_archivo::decodificar(&bytes).is_err());
        let mut vieja = crate::humo_archivo::codificar(&humo)?;
        assert!(decodificar(&vieja).is_err());
        vieja[4..8].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(decodificar(&vieja)?.1, VolumenHumo([1.0, 0.0, 0.52, 0.5]));
        for i in 0..bytes.len() {
            let mut rota = bytes.clone();
            rota[i] ^= 1;
            assert!(decodificar(&rota).is_err());
        }
        Ok(())
    }
}
