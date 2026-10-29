//! Referencia escalar de plazas; no se usa en el bucle del producto.
use super::{CANALES, CAPACIDAD, CONTROL};
/// Avanza el sistema con los diez parámetros públicos de emisión/dinámica/visor.
/// # Errors
/// Rechaza longitud, constantes inválidas y reloj fuera de dominio.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "plazas <=128, semilla <=65535 y generación <=65535"
)]
pub fn paso(estado: &mut [f32], p: &[f32; 10]) -> Result<(), &'static str> {
    let n = CAPACIDAD as usize;
    let control = CONTROL as usize * n;
    if estado.len() != CANALES as usize * n
        || p.iter().any(|v| !v.is_finite())
        || !(1.0..=128.0).contains(&p[0])
        || p[0] % 1.0 != 0.0
        || !(0.1..=5.0).contains(&p[1])
        || !(0.05..=2.0).contains(&p[2])
        || !(0.0..=8.0).contains(&p[3])
        || !(0.0..=5.0).contains(&p[4])
        || !(0.001..=0.04).contains(&p[5])
        || !(0.002..=0.03).contains(&p[6])
        || !(0.0..=65535.0).contains(&p[7])
        || p[7] % 1.0 != 0.0
        || !(0.0..=2.0).contains(&p[8])
        || !(0.0..=1.0).contains(&p[9])
        || !estado[control].is_finite()
        || !(0.0..=3600.0).contains(&estado[control])
    {
        return Err("estado o parámetros de partículas inválidos");
    }
    let tiempo = estado[control];
    let dt = p[5].min((3600.0 - tiempo).max(0.0));
    for id in 0..n {
        let mut q = core::array::from_fn::<_, 11, _>(|i| estado[i * n + id]);
        let valido = q[..10].iter().all(|v| v.is_finite() && v.abs() <= 1e20)
            && matches!(q[10], 0.0 | 1.0)
            && [6, 7, 8, 9].into_iter().all(|i| q[i] >= 0.0)
            && q[8] <= 65535.0
            && q[8] % 1.0 == 0.0;
        if !valido {
            estado[10 * n + id] = -1.0;
            continue;
        }
        let habilitado = (id as f32) < p[0];
        let fase = id as f32 / p[0] * if p[8] == 0.0 { 0.5 } else { p[8] };
        let inicio = if q[8] == 0.0 { fase.max(tiempo) } else { tiempo };
        let nace = habilitado
            && (p[8] == 0.0 || tiempo < p[8])
            && tiempo + dt >= inicio
            && q[10] == 0.0
            && q[8] < 65535.0
            && dt > 0.0;
        let delta = if nace { tiempo + dt - inicio } else { dt };
        if nace {
            q = nacer(id as u32, q[8] as u32 + 1, inicio, p);
        }
        if nace || (q[10] > 0.0 && habilitado) {
            q[4] -= p[3] * delta;
            let conservar = (1.0 - p[4] * delta).max(0.0);
            for eje in 0..3 {
                q[eje + 3] *= conservar;
                q[eje] += q[eje + 3] * delta;
            }
            if q[1] < 0.0 {
                q[1] = 0.0;
                q[4] *= -0.4;
            }
            q[6] = (q[6] + delta).min(q[7]);
            q[10] = f32::from(u8::from(
                q[6] < q[7] && (0.0..=1.0).contains(&q[0]) && (0.0..=1.0).contains(&q[2]) && q[1] <= 1.0,
            ));
        } else {
            q[10] = 0.0;
        }
        for i in 0..11 {
            estado[i * n + id] = q[i];
        }
    }
    let mut resumen = [0.0f32; 4];
    for id in 0..n {
        let activa = estado[10 * n + id];
        if activa > 0.0 {
            resumen[0] += 1.0;
            let velocidad = (3..6).map(|i| estado[i * n + id].abs()).sum::<f32>();
            resumen[3] = resumen[3].max(velocidad);
        }
        if activa < 0.0 {
            resumen[2] += 1.0;
        } else {
            resumen[1] += estado[8 * n + id];
        }
    }
    estado[control] = tiempo + dt;
    estado[control + 1..control + 5].copy_from_slice(&resumen);
    estado[control + 5] = p[0];
    estado[control + 6] = dt;
    estado[control + 7] = f32::from(u8::from(tiempo + dt >= 3600.0));
    Ok(())
}
#[allow(clippy::cast_precision_loss, reason = "generación <=65535")]
fn nacer(id: u32, generacion: u32, inicio: f32, p: &[f32; 10]) -> [f32; 11] {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "semilla validada")]
    let mut clave = id.wrapping_mul(747_796_405) ^ generacion.wrapping_mul(2_891_336_453) ^ (p[7] as u32);
    let azar = core::array::from_fn::<_, 3, _>(|_| {
        clave = clave.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (clave >> 16) as f32 * (1.0 / 65535.0)
    });
    let vertical = 0.75 + azar[2] * 0.5;
    [
        0.5,
        0.18,
        0.5,
        (azar[0] * 2.0 - 1.0) * p[2] * p[9],
        p[2] * vertical,
        (azar[1] * 2.0 - 1.0) * p[2] * p[9],
        0.0,
        p[1] * vertical,
        generacion as f32,
        inicio,
        1.0,
    ]
}
