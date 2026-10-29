//! Referencia escalar para el acoplamiento; solo de ensayo.
use alloc::vec::Vec;
/// Avanza plazas con caras MAC ya proyectadas y reloj congelado antes del subpaso.
/// # Errors
/// Rechaza dimensiones, constantes, tiempo y estado MAC no finito.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "lado validado <=64, conversiones tras clamp"
)]
pub fn paso(
    lado: usize,
    estado: &mut [f32],
    radio_fuente: f32,
    duracion: f32,
    esfera: [f32; 2],
) -> Result<(), &'static str> {
    let n = lado.saturating_pow(3);
    if !(8..=64).contains(&lado)
        || estado.len() < 17 * n + 1536
        || !radio_fuente.is_finite()
        || !(0.02..=0.18).contains(&radio_fuente)
        || !duracion.is_finite()
        || !(0.0..=2.0).contains(&duracion)
        || esfera.iter().any(|v| !v.is_finite())
        || !(0.0..=0.3).contains(&esfera[0])
        || !(0.31..=0.69).contains(&esfera[1])
        || estado[..17 * n].iter().any(|v| !v.is_finite())
    {
        return Err("contrato de trazadores inválido");
    }
    let tiempo = estado[16 * n + 16];
    let dt = estado[16 * n + 10];
    if dt < 0.0 || dt > 0.04 || tiempo < 0.0 {
        return Err("reloj de trazadores inválido");
    }
    let fluidos: Vec<_> = (0..n)
        .map(|i| {
            let c = [i % lado, (i / lado) % lado, i / (lado * lado)];
            let r2: f32 = c
                .into_iter()
                .zip([0.5, esfera[1], 0.5])
                .map(|(c, o)| {
                    let delta = (c as f32 + 0.5) / lado as f32 - o;
                    delta * delta
                })
                .sum();
            esfera[0] == 0.0 || r2 > esfera[0] * esfera[0]
        })
        .collect();
    for id in 0..128 {
        let mut q = core::array::from_fn::<_, 11, _>(|i| estado[17 * n + i * 128 + id]);
        if !q[..10].iter().all(|v| v.is_finite() && v.abs() <= 1e20)
            || !matches!(q[10], 0.0 | 1.0)
            || [6, 7, 8, 9].into_iter().any(|i| q[i] < 0.0)
            || q[8] > 65535.0
            || q[8] % 1.0 != 0.0
        {
            estado[17 * n + 10 * 128 + id] = -1.0;
            continue;
        }
        if estado[16 * n + 5] >= 3.0 || dt == 0.0 {
            continue;
        }
        let nace = q[10] == 0.0 && q[8] < 65535.0 && (duracion == 0.0 || tiempo < duracion);
        if nace {
            let c = [id % 4, (id / 4) % 8, id / 32];
            for eje in 0..3 {
                q[eje] = (c[eje] as f32 / if eje == 1 { 7.0 } else { 3.0 } - 0.5) * radio_fuente
                    + if eje == 1 { 0.18 + 0.5 / lado as f32 } else { 0.5 };
            }
            q[6] = 0.0;
            q[8] += 1.0;
            q[9] = tiempo;
        }
        if nace || q[10] > 0.0 {
            let p = [q[0], q[1], q[2]];
            for eje in 0..3 {
                let coords = core::array::from_fn(|i| p[i] * lado as f32 - if i == eje { 1.0 } else { 0.5 });
                q[3 + eje] = crate::humo3d::referencia::muestra(estado, lado, coords, eje, &fluidos);
                q[eje] += q[eje + 3] * dt;
            }
            q[6] = (q[6] + dt).min(2.0);
            q[7] = 2.0;
            let indices = [q[0], q[1], q[2]].map(|v| ((v.clamp(0.0, 1.0) * lado as f32) as usize).min(lado - 1));
            let celda = (indices[2] * lado + indices[1]) * lado + indices[0];
            q[10] =
                f32::from(u8::from(q[6] < q[7] && q[..3].iter().all(|v| (0.0..=1.0).contains(v)) && fluidos[celda]));
        }
        for i in 0..11 {
            estado[17 * n + i * 128 + id] = q[i];
        }
    }
    Ok(())
}
