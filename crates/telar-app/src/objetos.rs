//! Escena .tlar v5: 42 palabras, CRC y reemplazo mediante temporal exclusivo.
use std::{io::Read, path::Path};
use telar_ui::{EscenaObjetos, PALABRAS_OBJETOS};
const BYTES: usize = 16 + PALABRAS_OBJETOS * 4;

pub(crate) fn codificar(escena: &EscenaObjetos) -> Result<Vec<u8>, String> {
    escena.validar()?;
    let mut bytes = Vec::with_capacity(BYTES);
    for valor in [0x5241_4c54, 5, 42, 0].into_iter().chain(escena.guardar()) {
        bytes.extend_from_slice(&valor.to_le_bytes());
    }
    let crc = crate::proyecto::crc32(&bytes[16..]);
    bytes[12..16].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}
pub(crate) fn decodificar(bytes: &[u8]) -> Result<EscenaObjetos, String> {
    if bytes.len() != BYTES {
        return Err("tamaño de escena de objetos inválido".into());
    }
    let palabras: Vec<_> = bytes.as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
    if palabras[..3] != [0x5241_4c54, 5, 42] || palabras[3] != crate::proyecto::crc32(&bytes[16..]) {
        return Err("cabecera/CRC de escena .tlar v5 inválidos".into());
    }
    let estado = core::array::from_fn(|i| palabras[i + 4]);
    let mut escena = EscenaObjetos::default();
    escena.restaurar(&estado)?;
    Ok(escena)
}
pub(crate) fn leer(ruta: &Path) -> Result<EscenaObjetos, String> {
    crate::proyecto::validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| format!("abrir {}: {e}", ruta.display()))?
        .take(u64::try_from(BYTES + 1).map_err(|_| "tamaño excesivo")?)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decodificar(&bytes)
}
pub(crate) fn guardar(ruta: &Path, escena: &EscenaObjetos) -> Result<(), String> {
    crate::proyecto::guardar_bytes(ruta, &codificar(escena)?)
}
