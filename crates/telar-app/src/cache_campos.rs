//! Instantánea exacta de campos, configuración y cámara; formato binario propio acotado.
use std::{
    io::{Read, Write},
    path::Path,
};
const MAX_BYTES: usize = 16_777_216;
const MAGIC: u32 = 0x4843_4c54;
/// Revisión del ABI de simulación; debe cambiar si cambia la interpretación de campos/programas.
const REVISION: u32 = 2;
pub(crate) struct Cache {
    pub trazadores: bool,
    pub pasos: u32,
    pub perfil: [u32; 4],
    pub escena: Vec<u8>,
    pub camara: [f32; 8],
    pub campos: Vec<u32>,
}
impl Cache {
    pub(crate) fn codificar(&self) -> Result<Vec<u8>, String> {
        self.validar()?;
        let mut bytes = Vec::new();
        for v in [
            MAGIC,
            REVISION,
            32,
            if self.escena.len() == 200 { 2 } else { u32::from(self.trazadores) },
            u32::try_from(self.campos.len()).map_err(|_| "campos excesivos")?,
            self.pasos,
            self.perfil[0],
            self.perfil[1],
            self.perfil[2],
            self.perfil[3],
            u32::try_from(self.escena.len()).map_err(|_| "escena excesiva")?,
            0,
        ] {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.extend_from_slice(&self.escena);
        for v in self.camara.map(f32::to_bits).into_iter().chain(self.campos.iter().copied()) {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        let crc = crc(&bytes);
        bytes[44..48].copy_from_slice(&crc.to_le_bytes());
        Ok(bytes)
    }
    pub(crate) fn decodificar(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 168 || bytes.len() > MAX_BYTES || !bytes.len().is_multiple_of(4) {
            return Err("tamaño de caché inválido".into());
        }
        let h: Vec<_> = bytes[..48].as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
        if h[0] != MAGIC
            || h[1] != REVISION
            || h[2] != 32
            || h[3] > 2
            || !matches!(h[10], 88 | 200)
            || h[11] != crc(bytes)
        {
            return Err("cabecera, revisión o CRC de caché inválidos".into());
        }
        if (h[3] == 2) != (h[10] == 200) {
            return Err("modo y escena de caché no coinciden".into());
        }
        let cuenta = usize::try_from(h[4]).map_err(|_| "campos excesivos")?;
        if cuenta > MAX_BYTES / 4 || bytes.len() != 80 + h[10] as usize + 4 * cuenta {
            return Err("recuento de caché incompatible".into());
        }
        let fin = 48 + h[10] as usize;
        let palabras: Vec<_> = bytes[fin..].as_chunks::<4>().0.iter().map(|v| u32::from_le_bytes(*v)).collect();
        let resultado = Self {
            trazadores: h[3] > 0,
            pasos: h[5],
            perfil: [h[6], h[7], h[8], h[9]],
            escena: bytes[48..fin].to_vec(),
            camara: core::array::from_fn(|i| f32::from_bits(palabras[i])),
            campos: palabras[8..].to_vec(),
        };
        resultado.validar()?;
        Ok(resultado)
    }
    fn validar(&self) -> Result<(), String> {
        if self.escena.len() == 200 {
            crate::vfx_archivo::decodificar(&self.escena)?;
        } else {
            crate::volumen_archivo::decodificar(&self.escena)?;
        }
        let canales = if self.escena.len() == 200 {
            telar_kernel::vfx::canales(32) + telar_render::vfx_final::PALABRAS_IMAGEN.div_ceil(32u32.pow(3))
        } else if self.trazadores {
            telar_kernel::trazadores::canales(32)
        } else {
            17
        };
        if !matches!(self.escena.len(), 88 | 200)
            || self.campos.len() != usize::try_from(canales * 32u32.pow(3)).map_err(|_| "campos excesivos")?
            || self.campos.iter().any(|v| !f32::from_bits(*v).is_finite())
            || self.camara.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
            || !(1.2..=5.0).contains(&self.camara[3])
            || !(-89.0..=89.0).contains(&self.camara[5])
            || !(1.0..=170.0).contains(&self.camara[6])
            || !(0.0001..=1.0).contains(&self.camara[7])
        {
            return Err("estado o cámara de caché incompatible".into());
        }
        let base = 16 * 32usize.pow(3);
        if self.campos[base + 5] == 3.0f32.to_bits()
            || f32::from_bits(self.campos[base + 14]) >= 2.0
            || f32::from_bits(self.campos[base + 13]) < 0.0
        {
            return Err("caché con simulación inválida".into());
        }
        Ok(())
    }
}
fn crc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes[..44].iter().chain(&bytes[48..]) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
pub(crate) fn leer(ruta: &Path) -> Result<Cache, String> {
    extension(ruta)?;
    let mut b = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    Cache::decodificar(&b)
}
pub(crate) fn guardar(ruta: &Path, cache: &Cache) -> Result<(), String> {
    extension(ruta)?;
    let bytes = cache.codificar()?;
    let id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let nombre = ruta.file_name().ok_or("caché sin nombre")?.to_string_lossy();
    let temporal = ruta.with_file_name(format!(".{nombre}.{}.{id}.tmp", std::process::id()));
    let mut f = std::fs::OpenOptions::new().create_new(true).write(true).open(&temporal).map_err(|e| e.to_string())?;
    let resultado = (|| {
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        drop(f);
        // Publicación exclusiva en el mismo volumen: el destino existente nunca se reemplaza.
        std::fs::hard_link(&temporal, ruta).map_err(|e| e.to_string())
    })();
    let _ = std::fs::remove_file(&temporal);
    resultado
}
fn extension(ruta: &Path) -> Result<(), String> {
    if ruta.extension().is_some_and(|v| v.eq_ignore_ascii_case("tlcache")) {
        Ok(())
    } else {
        Err("se requiere extensión .tlcache".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binario_protege_metadatos_y_campos() -> Result<(), String> {
        let escena = crate::volumen_archivo::codificar(
            &telar_ui::Humo::default(),
            &telar_ui::VolumenHumo::default(),
            &telar_ui::ExplosionHumo::default(),
        )?;
        let cache = Cache {
            trazadores: false,
            pasos: 37,
            perfil: [1, 2, 3, 4],
            escena,
            camara: [0.5, 0.48, 0.5, 1.9, 32.0, 12.0, 45.0, 0.01],
            campos: vec![0; 17 * 32usize.pow(3)],
        };
        let bytes = cache.codificar()?;
        let otro = Cache::decodificar(&bytes)?;
        assert_eq!(otro.campos, cache.campos);
        assert_eq!(otro.pasos, 37);
        assert_eq!(otro.camara, cache.camara);
        for i in [0, 4, 8, 12, 16, 20, 24, 40, 44, 48, 136, 168, bytes.len() / 2, bytes.len() - 1] {
            let mut rota = bytes.clone();
            rota[i] ^= 1;
            assert!(Cache::decodificar(&rota).is_err());
        }
        assert!(Cache::decodificar(&bytes[..bytes.len() - 1]).is_err());
        Ok(())
    }
}
