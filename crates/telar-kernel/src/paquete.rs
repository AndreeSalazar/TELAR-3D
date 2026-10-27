//! Archivo .tlar v1: arranque y coordinador, little-endian y CRC32 de integridad.
//! No representa todavía una escena ni autentica código ejecutable.
use crate::arranque;
use alloc::vec::Vec;

const MAX_PALABRAS: usize = 1_048_576;
const PREFIJO: usize = 20;

/// Contenido inspeccionable de un archivo de arranque .tlar.
pub struct Paquete {
    /// Encabezado GPU de 32 palabras.
    pub cabecera: [u32; 32],
    /// Coordinador SPIR-V; requiere validación completa antes de cualquier ejecución.
    pub codigo: Vec<u32>,
}

fn validar(cabecera: &[u32; 32], codigo: &[u32]) -> Result<(), &'static str> {
    if cabecera[..3] != [arranque::FIRMA, arranque::VERSION, 32] {
        return Err("cabecera TLAR incompatible");
    }
    if codigo.len() < 5
        || codigo.len() > MAX_PALABRAS
        || codigo[0] != 0x0723_0203
        || codigo[1] != 0x0001_0300
        || codigo[3] == 0
        || codigo[4] != 0
    {
        return Err("cabecera SPIR-V incompatible");
    }
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

/// Codifica un arranque .tlar sin escribir archivos ni invocar una GPU.
///
/// # Errors
/// Rechaza cabeceras incompatibles y módulos fuera del límite de un millón de palabras.
pub fn codificar(cabecera: &[u32; 32], codigo: &[u32]) -> Result<Vec<u8>, &'static str> {
    validar(cabecera, codigo)?;
    let mut bytes = Vec::with_capacity(PREFIJO + (32 + codigo.len()) * 4);
    for palabra in [arranque::FIRMA, 1, 32, u32::try_from(codigo.len()).map_err(|_| "módulo excesivo")?, 0] {
        bytes.extend(palabra.to_le_bytes());
    }
    for palabra in cabecera.iter().chain(codigo) {
        bytes.extend(palabra.to_le_bytes());
    }
    let crc = crc32(&bytes[PREFIJO..]);
    bytes[16..20].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}

/// Comprueba y decodifica .tlar. No ejecuta ni carga sus shaders en el controlador.
///
/// # Errors
/// Rechaza truncamiento, bytes sobrantes, versión, tamaños, CRC y cabeceras inválidos.
pub fn decodificar(bytes: &[u8]) -> Result<Paquete, &'static str> {
    if bytes.len() < PREFIJO || bytes.len() > PREFIJO + (32 + MAX_PALABRAS) * 4 {
        return Err("tamaño de archivo .tlar inválido");
    }
    let palabra = |i: usize| u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap_or([0; 4]));
    if palabra(0) != arranque::FIRMA || palabra(1) != 1 || palabra(2) != 32 {
        return Err("formato .tlar incompatible");
    }
    let cantidad = usize::try_from(palabra(3)).map_err(|_| "módulo excesivo")?;
    if !(5..=MAX_PALABRAS).contains(&cantidad) || bytes.len() != PREFIJO + (32 + cantidad) * 4 {
        return Err("longitudes .tlar incoherentes");
    }
    if palabra(4) != crc32(&bytes[PREFIJO..]) {
        return Err("integridad CRC32 incorrecta");
    }
    let cabecera = core::array::from_fn(|i| {
        u32::from_le_bytes(bytes[PREFIJO + i * 4..PREFIJO + i * 4 + 4].try_into().unwrap_or([0; 4]))
    });
    let codigo = bytes[PREFIJO + 128..].as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect::<Vec<_>>();
    validar(&cabecera, &codigo)?;
    Ok(Paquete { cabecera, codigo })
}
