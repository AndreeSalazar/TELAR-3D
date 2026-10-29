//! Muestreo explícito del estado MAC: centros escalares y promedio de caras para v.
use telar_intercambio::{MAX_PUNTOS, Punto};
pub(crate) fn convertir(lado: u32, campos: &[u32], tiempo: f32) -> Result<Vec<Punto>, String> {
    let lado = usize::try_from(lado).map_err(|_| "lado de puntos inválido")?;
    let cantidad = lado.checked_pow(3).filter(|v| *v <= MAX_PUNTOS).ok_or("demasiadas muestras de volumen")?;
    if lado < 4 || campos.len() != cantidad * 5 || !tiempo.is_finite() || tiempo < 0.0 {
        return Err("campos/tiempo incompatibles con exportación de puntos".into());
    }
    if campos.iter().any(|v| !f32::from_bits(*v).is_finite()) {
        return Err("campo no finito; no se exporta".into());
    }
    let mut puntos = Vec::with_capacity(cantidad);
    for pixel in 0..cantidad {
        let coords = [pixel % lado, (pixel / lado) % lado, pixel / (lado * lado)];
        #[allow(clippy::cast_precision_loss, reason = "lado acotado por 65536 muestras")]
        let posicion = coords.map(|v| (v as f32 + 0.5) / lado as f32);
        let velocidad = core::array::from_fn(|eje| {
            let paso = [1, lado, lado * lado][eje];
            let actual = f32::from_bits(campos[eje * cantidad + pixel]);
            let anterior = if coords[eje] > 0 { f32::from_bits(campos[eje * cantidad + pixel - paso]) } else { 0.0 };
            actual * 0.5 + anterior * 0.5
        });
        puntos.push(Punto {
            posicion,
            velocidad,
            densidad: f32::from_bits(campos[3 * cantidad + pixel]),
            calor: f32::from_bits(campos[4 * cantidad + pixel]),
            id: u32::try_from(pixel).map_err(|_| "id fuera de dominio")?,
            tiempo,
        });
    }
    telar_intercambio::validar(&puntos)?;
    Ok(puntos)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn velocidad_de_centro_no_confunde_caras_con_puntos() -> Result<(), String> {
        let mut campos = vec![0.0f32.to_bits(); 5 * 64];
        for pixel in 0..64 {
            campos[pixel] = 2.0f32.to_bits();
            campos[64 + pixel] = 4.0f32.to_bits();
            campos[128 + pixel] = 6.0f32.to_bits();
            campos[192 + pixel] = 0.25f32.to_bits();
            campos[256 + pixel] = 0.5f32.to_bits();
        }
        let puntos = convertir(4, &campos, 0.125)?;
        assert_eq!(puntos[0].posicion, [0.125; 3]);
        assert_eq!(puntos[0].velocidad, [1.0, 2.0, 3.0]);
        assert_eq!(puntos[21].velocidad, [2.0, 4.0, 6.0]);
        assert_eq!(puntos[63].posicion, [0.875; 3]);
        assert_eq!(puntos[21].id, 21);
        assert!(convertir(64, &[], 0.0).is_err());
        assert!(convertir(4, &campos[..319], 0.0).is_err());
        campos[0] = f32::NAN.to_bits();
        assert!(convertir(4, &campos, 0.0).is_err());
        Ok(())
    }
}
