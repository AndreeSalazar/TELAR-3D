//! PFM lineal y PPM de revisión; descarga final explícita, sin conversión en vivo.
use std::{fs::OpenOptions, io::Write, path::Path};

pub(crate) fn codificar(datos: &[u32], tamano: [u32; 2], muestras: u32) -> Result<(Vec<u8>, Vec<u8>), String> {
    let [ancho, alto] = tamano;
    let pixeles = usize::try_from(u64::from(ancho) * u64::from(alto)).map_err(|_| "imagen demasiado grande")?;
    if !(1..=512).contains(&ancho)
        || !(1..=512).contains(&alto)
        || !(1..=1024).contains(&muestras)
        || datos.len() != pixeles * 5
    {
        return Err("tamaño de salida inválido".into());
    }
    let mut pfm = format!("PF\n{ancho} {alto}\n-1.0\n").into_bytes();
    let mut ppm = format!("P6\n{ancho} {alto}\n255\n").into_bytes();
    let ancho = usize::try_from(ancho).map_err(|_| "ancho inválido")?;
    let alto = usize::try_from(alto).map_err(|_| "alto inválido")?;
    for pixel in datos.as_chunks::<5>().0 {
        if pixel[3] != muestras {
            return Err("GPU no completó el recuento de muestras esperado".into());
        }
        for valor in &pixel[..3] {
            let lineal = f32::from_bits(*valor);
            if !lineal.is_finite() || lineal < 0.0 {
                return Err("radiancia GPU no finita o negativa".into());
            }
        }
        ppm.extend_from_slice(&pixel[4].to_le_bytes()[..3]);
    }
    // PFM almacena filas de abajo hacia arriba; PPM conserva el orden superior.
    for fila in (0..alto).rev() {
        for pixel in datos[fila * ancho * 5..(fila + 1) * ancho * 5].as_chunks::<5>().0 {
            for canal in &pixel[..3] {
                pfm.extend_from_slice(&canal.to_le_bytes());
            }
        }
    }
    Ok((pfm, ppm))
}

pub(crate) fn guardar(ruta: &Path, pfm: &[u8], ppm: &[u8], receta: &str) -> Result<(), String> {
    let rutas = [ruta.to_path_buf(), ruta.with_extension("ppm"), ruta.with_extension("txt")];
    let mut archivos = Vec::new();
    for actual in &rutas {
        match OpenOptions::new().write(true).create_new(true).open(actual) {
            Ok(archivo) => archivos.push(archivo),
            Err(error) => {
                let creados = archivos.len();
                drop(archivos);
                for propia in &rutas[..creados] {
                    let _ = std::fs::remove_file(propia);
                }
                return Err(format!("crear {} (usa un nombre nuevo): {error}", actual.display()));
            }
        }
    }
    let resultado = archivos
        .iter_mut()
        .zip([pfm, ppm, receta.as_bytes()])
        .try_for_each(|(archivo, bytes)| archivo.write_all(bytes).and_then(|()| archivo.sync_all()));
    drop(archivos);
    if let Err(error) = resultado {
        for propia in &rutas {
            let _ = std::fs::remove_file(propia);
        }
        return Err(format!("escribir render: {error}"));
    }
    Ok(())
}
