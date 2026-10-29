//! Configuración del emisor v12; no contiene partículas ni reloj reanudable.
use std::{io::Read, path::Path};
use telar_ui::Particulas;
pub(crate) fn codificar(p: &Particulas) -> Result<Vec<u8>, String> {
    p.validar()?;
    let mut b = Vec::with_capacity(56);
    for w in [0x5241_4c54, 12, 10, 0].into_iter().chain(p.0.map(f32::to_bits)) {
        b.extend_from_slice(&w.to_le_bytes());
    }
    let crc = crate::proyecto::crc32(&b[16..]);
    b[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(b)
}
pub(crate) fn decodificar(b: &[u8]) -> Result<Particulas, String> {
    if b.len() != 56 {
        return Err("tamaño de configuración de partículas inválido".into());
    }
    let w: Vec<_> = b.as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
    if w[..3] != [0x5241_4c54, 12, 10] || w[3] != crate::proyecto::crc32(&b[16..]) {
        return Err("cabecera/CRC de partículas inválidos".into());
    }
    let mut p = Particulas::default();
    for (d, v) in p.0.iter_mut().zip(&w[4..]) {
        *d = f32::from_bits(*v);
    }
    p.validar()?;
    Ok(p)
}
pub(crate) fn leer(ruta: &Path) -> Result<Particulas, String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut b = Vec::new();
    std::fs::File::open(ruta).map_err(|e| e.to_string())?.take(57).read_to_end(&mut b).map_err(|e| e.to_string())?;
    decodificar(&b)
}
pub(crate) fn guardar(ruta: &Path, p: &Particulas) -> Result<(), String> {
    crate::proyecto::guardar_bytes(ruta, &codificar(p)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_crc_y_dominios() -> Result<(), String> {
        let p = Particulas::default();
        let b = codificar(&p)?;
        assert_eq!(decodificar(&b)?, p);
        for i in 0..b.len() {
            let mut roto = b.clone();
            roto[i] ^= 1;
            assert!(decodificar(&roto).is_err());
        }
        assert!(crate::humo_archivo::decodificar(&b).is_err());
        Ok(())
    }
}
