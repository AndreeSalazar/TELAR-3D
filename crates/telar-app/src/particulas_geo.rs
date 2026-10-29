//! Lectura explícita de partículas vivas y exportación de edad/vida; no reejecuta dinámica.
use std::{io::Write, path::Path};
use telar_intercambio::{Particula, Punto};
pub(crate) fn convertir(datos: &[u32], tiempo: f32) -> Result<Vec<Particula>, String> {
    const N: usize = 128;
    if datos.len() != 11 * N || !tiempo.is_finite() || tiempo < 0.0 {
        return Err("estado de partículas incompatible".into());
    }
    let mut puntos = Vec::new();
    for id in 0..N {
        let flag = f32::from_bits(datos[10 * N + id]);
        if flag.to_bits() == 0.0f32.to_bits() {
            continue;
        }
        if flag.to_bits() != 1.0f32.to_bits() {
            return Err("partícula en cuarentena o indicador inválido".into());
        }
        let leer = |canal: usize| f32::from_bits(datos[canal * N + id]);
        let generacion = leer(8);
        if !generacion.is_finite() || !(1.0..=65535.0).contains(&generacion) || generacion % 1.0 != 0.0 {
            return Err("generación de partícula inválida".into());
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "generación entera validada <=65535"
        )]
        let identidad = (generacion as u32) * 128 + u32::try_from(id).map_err(|_| "id inválido")?;
        let edad = leer(6);
        let vida = leer(7);
        let nacimiento = leer(9);
        let punto = Punto {
            posicion: [leer(0), leer(1), leer(2)],
            velocidad: [leer(3), leer(4), leer(5)],
            densidad: 1.0,
            calor: (1.0 - edad / vida).clamp(0.0, 1.0),
            id: identidad,
            tiempo,
        };
        puntos.push(Particula { punto, edad, vida, nacimiento });
    }
    telar_intercambio::validar_particulas(&puntos)?;
    Ok(puntos)
}
pub(crate) fn exportar(gpu: &telar_gpu::Gpu, ruta: &Path) -> Result<usize, String> {
    crate::intercambio_archivo::validar_destino(ruta)?;
    let control = gpu.inspeccionar_rango_campos(11 * 128, 8)?;
    if control.iter().any(|v| !f32::from_bits(*v).is_finite()) || f32::from_bits(control[3]) > 0.0 {
        return Err("resumen de partículas inválido; reinicia antes de exportar".into());
    }
    let datos = gpu.inspeccionar_rango_campos(0, 11 * 128)?;
    let puntos = convertir(&datos, f32::from_bits(control[0]))?;
    let texto = telar_intercambio::codificar_particulas_geo(&puntos)?;
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(ruta).map_err(|e| e.to_string())?;
    f.write_all(texto.as_bytes()).map_err(|e| e.to_string())?;
    f.sync_all().map_err(|e| e.to_string())?;
    Ok(puntos.len())
}
