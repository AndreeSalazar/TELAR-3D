//! Escena v13: parámetros del volumen y 28 controles de reacción/secundarios/aspecto.
use std::{io::Read, path::Path};
use telar_ui::{ExplosionHumo, Humo, Vfx, VolumenHumo};
pub(crate) fn codificar(editor: &telar_ui::EditorHumo) -> Result<Vec<u8>, String> {
    let vfx = editor.vfx.ok_or("faltan controles VFX")?;
    vfx.validar()?;
    let anterior = crate::volumen_archivo::codificar(&editor.humo, &editor.volumen_opciones, &editor.explosion)?;
    let mut bytes = Vec::with_capacity(200);
    for v in [0x5241_4c54u32, 13, 46, 0] {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes.extend_from_slice(&anterior[16..]);
    for v in vfx.0 {
        bytes.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    let crc = crate::proyecto::crc32(&bytes[16..]);
    bytes[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}
pub(crate) fn decodificar(bytes: &[u8]) -> Result<(Humo, VolumenHumo, ExplosionHumo, Vfx), String> {
    if bytes.len() != 200 {
        return Err("plano reactivo requiere .tlar v13 de 200 B".into());
    }
    let h: Vec<_> = bytes[..16].as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
    if h[..3] != [0x5241_4c54, 13, 46] || h[3] != crate::proyecto::crc32(&bytes[16..]) {
        return Err("cabecera/CRC de plano VFX inválidos".into());
    }
    let mut anterior = bytes[..88].to_vec();
    anterior[4..8].copy_from_slice(&11u32.to_le_bytes());
    anterior[8..12].copy_from_slice(&18u32.to_le_bytes());
    let crc = crate::proyecto::crc32(&anterior[16..]);
    anterior[12..16].copy_from_slice(&crc.to_le_bytes());
    let (humo, tiempo, pulso) = crate::volumen_archivo::decodificar(&anterior)?;
    let v: Vec<_> = bytes[88..].as_chunks::<4>().0.iter().map(|v| f32::from_bits(u32::from_le_bytes(*v))).collect();
    let vfx = Vfx(v.try_into().map_err(|_| "parámetros VFX incompletos")?);
    vfx.validar()?;
    Ok((humo, tiempo, pulso, vfx))
}
pub(crate) fn leer(ruta: &Path, editor: &mut telar_ui::EditorHumo) -> Result<(), String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(201)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    aplicar(&bytes, editor)
}
pub(crate) fn aplicar(bytes: &[u8], editor: &mut telar_ui::EditorHumo) -> Result<(), String> {
    let (h, t, p, v) = decodificar(bytes)?;
    editor.humo = h;
    editor.volumen_opciones = t;
    editor.explosion = p;
    editor.vfx = Some(v);
    editor.volumen = true;
    editor.trazadores = true;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escena_completa_y_crc() -> Result<(), String> {
        let mut editor = telar_ui::EditorHumo::default();
        editor.vfx = Some(Vfx::default());
        let bytes = codificar(&editor)?;
        assert_eq!(decodificar(&bytes)?.3, editor.vfx.ok_or("vfx")?);
        for i in 0..bytes.len() {
            let mut roto = bytes.clone();
            roto[i] ^= 1;
            assert!(decodificar(&roto).is_err());
        }
        Ok(())
    }
}
