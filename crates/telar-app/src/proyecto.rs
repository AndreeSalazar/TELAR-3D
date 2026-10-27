//! Escenas .tlar v2/v3: datos acotados y grafo, sin código de archivo ni identidad GPU.
use nudo::Camara;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use telar_ui::{Estudio, PALABRAS_ESTUDIO};

const FIRMA: u32 = 0x5241_4c54;
const PALABRAS: usize = PALABRAS_ESTUDIO + 9;
const BYTES: usize = 16 + PALABRAS * 4;
const MAX_BYTES: usize = 416;
static TEMPORALES: AtomicU64 = AtomicU64::new(0);

/// Datos de una rejilla y sus operaciones conectables; sin recursos arbitrarios.
pub(crate) struct Proyecto {
    pub(crate) estudio: [u32; PALABRAS_ESTUDIO],
    pub(crate) camara: Camara,
    pub(crate) fase: f32,
    pub(crate) red: Vec<u32>,
}

impl Proyecto {
    pub(crate) fn capturar(estudio: &Estudio, camara: &Camara, fase: f32) -> Self {
        Self { estudio: estudio.guardar_estado(), camara: camara.clone(), fase, red: estudio.guardar_red() }
    }

    pub(crate) fn validar(&self) -> Result<(), String> {
        let mut estudio = Estudio::default();
        estudio.restaurar_estado(&self.estudio)?;
        if !self.red.is_empty() {
            estudio.restaurar_red(&self.red)?;
        }
        let c = &self.camara;
        let valido = |v: f32, a, b| v.is_finite() && (a..=b).contains(&v);
        if !c.objetivo.iter().all(|v| valido(*v, -1.0e6, 1.0e6))
            || !valido(c.distancia, 0.001, 1.0e6)
            || !valido(c.giro, -360.0, 360.0)
            || !valido(c.inclinacion, -89.0, 89.0)
            || !valido(c.campo, 1.0, 179.0)
            || !valido(c.cerca, 1.0e-6, 1.0e3)
            || c.distancia < c.cerca * 2.0
            || !valido(self.fase, 0.0, 1000.0)
            || self.fase >= 1000.0
        {
            return Err("cámara o fase de escena inválida".into());
        }
        Ok(())
    }

    pub(crate) fn codificar(&self) -> Result<Vec<u8>, String> {
        self.validar()?;
        let c = &self.camara;
        let camara = [
            c.objetivo[0],
            c.objetivo[1],
            c.objetivo[2],
            c.distancia,
            c.giro,
            c.inclinacion,
            c.campo,
            c.cerca,
            self.fase,
        ];
        let cantidad = u32::try_from(PALABRAS + self.red.len()).map_err(|_| "escena excesiva")?;
        let version = if self.red.is_empty() { 2 } else { 3 };
        let mut bytes = Vec::with_capacity(BYTES + self.red.len() * 4);
        for palabra in [FIRMA, version, cantidad, 0]
            .into_iter()
            .chain(self.estudio)
            .chain(camara.map(f32::to_bits))
            .chain(self.red.iter().copied())
        {
            bytes.extend(palabra.to_le_bytes());
        }
        let crc = crc32(&bytes[16..]);
        bytes[12..16].copy_from_slice(&crc.to_le_bytes());
        Ok(bytes)
    }

    pub(crate) fn decodificar(bytes: &[u8]) -> Result<Self, String> {
        if !(BYTES..=MAX_BYTES).contains(&bytes.len()) || !bytes.len().is_multiple_of(4) {
            return Err("tamaño de escena .tlar inválido".into());
        }
        let palabras: Vec<u32> = bytes.as_chunks::<4>().0.iter().map(|b| u32::from_le_bytes(*b)).collect();
        if palabras[0] != FIRMA
            || !matches!(palabras[1], 2 | 3)
            || palabras[2] as usize != palabras.len() - 4
            || (palabras[1] == 2 && bytes.len() != BYTES)
            || (palabras[1] == 3 && bytes.len() == BYTES)
        {
            return Err("se requiere escena .tlar v2/v3; el arranque v1 no es una escena".into());
        }
        if palabras[3] != crc32(&bytes[16..]) {
            return Err("CRC de escena incorrecto".into());
        }
        let estudio = core::array::from_fn(|i| palabras[i + 4]);
        let v = |i| f32::from_bits(palabras[4 + PALABRAS_ESTUDIO + i]);
        let proyecto = Self {
            estudio,
            camara: Camara {
                objetivo: [v(0), v(1), v(2)],
                distancia: v(3),
                giro: v(4),
                inclinacion: v(5),
                campo: v(6),
                cerca: v(7),
            },
            fase: v(8),
            red: palabras[4 + PALABRAS..].to_vec(),
        };
        proyecto.validar()?;
        Ok(proyecto)
    }
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

pub(crate) fn validar_ruta(ruta: &Path) -> Result<(), String> {
    if !ruta.extension().is_some_and(|e| e.eq_ignore_ascii_case("tlar")) {
        return Err("la escena requiere extensión .tlar".into());
    }
    Ok(())
}

pub(crate) fn leer(ruta: &Path) -> Result<Proyecto, String> {
    validar_ruta(ruta)?;
    let mut bytes = Vec::new();
    fs::File::open(ruta)
        .map_err(|e| format!("abrir {}: {e}", ruta.display()))?
        .take(417)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("leer escena: {e}"))?;
    Proyecto::decodificar(&bytes)
}

pub(crate) fn guardar(ruta: &Path, proyecto: &Proyecto) -> Result<(), String> {
    validar_ruta(ruta)?;
    let bytes = proyecto.codificar()?;
    // Temporal exclusivo en el mismo directorio: no truncar el proyecto anterior.
    let id = TEMPORALES.fetch_add(1, Ordering::Relaxed);
    let nombre = ruta.file_name().ok_or("ruta de escena sin nombre")?.to_string_lossy();
    let temporal = ruta.with_file_name(format!(".{nombre}.{}.{id}.tmp", std::process::id()));
    let mut archivo = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporal)
        .map_err(|e| format!("crear temporal de escena: {e}"))?;
    let resultado = (|| {
        archivo.write_all(&bytes).map_err(|e| format!("escribir escena: {e}"))?;
        archivo.sync_all().map_err(|e| format!("sincronizar escena: {e}"))?;
        drop(archivo);
        fs::rename(&temporal, ruta).map_err(|e| format!("reemplazar escena {}: {e}", ruta.display()))
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }
    resultado
}

pub(crate) fn argumento(args: &[String], nombre: &str) -> Result<Option<String>, String> {
    let mut encontrados = args.iter().enumerate().filter(|(_, a)| a.as_str() == nombre);
    let Some((indice, _)) = encontrados.next() else { return Ok(None) };
    if encontrados.next().is_some() {
        return Err(format!("{nombre} repetido"));
    }
    let ruta =
        args.get(indice + 1).filter(|r| !r.starts_with("--")).ok_or_else(|| format!("{nombre} requiere ruta"))?;
    validar_ruta(Path::new(ruta))?;
    Ok(Some(ruta.clone()))
}
