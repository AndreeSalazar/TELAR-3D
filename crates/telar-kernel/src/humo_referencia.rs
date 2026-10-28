//! Referencia escalar independiente para rejillas de ensayo; no participa en el visor.
use alloc::vec::Vec;
/// Avanza un paso del esquema discreto. Los doce planos conservan el mismo ABI.
/// Parámetros: radio, emisión, flotación, viento, disipación, dt, exposición, iteraciones UI.
/// # Errors
/// Rechaza dimensiones, parámetros o estado no finitos/fuera de contrato.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "coordenadas sujetas al dominio y rejilla <=256"
)]
pub fn paso(lado: usize, iteraciones: u32, estado: &mut [f32], p: &[f32; 8]) -> Result<(), &'static str> {
    validar(lado, iteraciones, estado, p)?;
    let n = lado * lado;
    let escala = lado as f32;
    let dt = p[5];
    let mut provisional = alloc::vec![0.0;4*n];
    for y in 0..lado {
        for x in 0..lado {
            let pixel = y * lado + x;
            let punto = [x as f32, y as f32];
            let dx = punto[0] - (escala - 1.0) * 0.5;
            let dy = punto[1] - escala * 0.18;
            let radio = p[0] * escala;
            let peso = (1.0 - (dx * dx + dy * dy) / (radio * radio)).clamp(0.0, 1.0);
            let fuente = peso * peso;
            for canal in 0..4 {
                let mut posicion = punto;
                if canal < 2 {
                    posicion[canal] += 0.5;
                }
                let velocidad = [
                    muestra(estado, lado, [posicion[0] - 0.5, posicion[1]], 0),
                    muestra(estado, lado, [posicion[0], posicion[1] - 0.5], 1),
                ];
                let mut antes = core::array::from_fn(|i| posicion[i] - dt * escala * velocidad[i]);
                if canal < 2 {
                    antes[canal] -= 0.5;
                }
                let mut valor = muestra(estado, lado, antes, canal);
                if canal < 2 {
                    let calor = muestra(estado, lado, punto, 3);
                    let fuerza = calor * if canal == 0 { p[3] } else { p[2] };
                    valor = (valor + dt * fuerza).clamp(-0.8, 0.8);
                    if (canal == 0 && x == lado - 1) || (canal == 1 && y == lado - 1) {
                        valor = 0.0;
                    }
                } else {
                    let (disipacion, tasa) = if canal == 2 { (p[4], p[1]) } else { (0.2, p[1]) };
                    valor = (valor * (1.0 - disipacion * dt) + tasa * dt * fuente).clamp(0.0, 10.0);
                }
                provisional[canal * n + pixel] = valor;
            }
        }
    }
    estado[4 * n..8 * n].copy_from_slice(&provisional);
    let div = divergencia(lado, &provisional);
    estado[8 * n..9 * n].copy_from_slice(&div);
    let mut anterior = estado[10 * n..11 * n].to_vec();
    let mut siguiente = anterior.clone();
    for iter in 0..iteraciones {
        for y in 0..lado {
            for x in 0..lado {
                let pixel = y * lado + x;
                let mut suma = 0.0;
                let mut cantidad = 0.0;
                if x > 0 {
                    suma += anterior[pixel - 1];
                    cantidad += 1.0;
                }
                if x + 1 < lado {
                    suma += anterior[pixel + 1];
                    cantidad += 1.0;
                }
                if y > 0 {
                    suma += anterior[pixel - lado];
                    cantidad += 1.0;
                }
                if y + 1 < lado {
                    suma += anterior[pixel + lado];
                    cantidad += 1.0;
                }
                let candidata = (suma - div[pixel] / (escala * escala)) / cantidad;
                siguiente[pixel] = candidata * (2.0 / 3.0) + anterior[pixel] * (1.0 / 3.0);
            }
        }
        let plano = if iter % 2 == 0 { 9 } else { 10 };
        estado[plano * n..(plano + 1) * n].copy_from_slice(&siguiente);
        core::mem::swap(&mut anterior, &mut siguiente);
    }
    for y in 0..lado {
        for x in 0..lado {
            let pixel = y * lado + x;
            let presion = anterior[pixel];
            estado[pixel] =
                if x + 1 < lado { provisional[pixel] - (anterior[pixel + 1] - presion) * escala } else { 0.0 };
            estado[n + pixel] =
                if y + 1 < lado { provisional[n + pixel] - (anterior[pixel + lado] - presion) * escala } else { 0.0 };
        }
    }
    estado[2 * n..4 * n].copy_from_slice(&provisional[2 * n..4 * n]);
    let posterior = divergencia(lado, estado);
    estado[11 * n..12 * n].copy_from_slice(&posterior);
    Ok(())
}
/// Divergencia MAC: caras derechas/superiores; paredes externas con flujo cero.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
pub fn divergencia(lado: usize, estado: &[f32]) -> Vec<f32> {
    if lado == 0 || lado > 256 || estado.len() < 2 * lado * lado {
        return Vec::new();
    }
    let mut resultado = alloc::vec![0.0;lado*lado];
    let n = lado * lado;
    for y in 0..lado {
        for x in 0..lado {
            let pixel = y * lado + x;
            let izquierda = if x > 0 { estado[pixel - 1] } else { 0.0 };
            let abajo = if y > 0 { estado[n + pixel - lado] } else { 0.0 };
            resultado[pixel] = ((estado[pixel] - izquierda) + (estado[n + pixel] - abajo)) * lado as f32;
        }
    }
    resultado
}
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "índices tras clamp: 0..lado, lado <=256"
)]
fn muestra(estado: &[f32], lado: usize, coords: [f32; 2], canal: usize) -> f32 {
    let halo = canal < 2;
    let min = if halo { -1.0 } else { 0.0 };
    let coords = coords.map(|v| v.clamp(min, (lado - 1) as f32) + 1.0);
    let bases = coords.map(|v| v as usize);
    let pesos = core::array::from_fn::<_, 2, _>(|i| coords[i] - bases[i] as f32);
    let leer = |dx: usize, dy: usize| {
        let x = bases[0] + dx;
        let y = bases[1] + dy;
        if halo && (x == 0 || y == 0) {
            return 0.0;
        }
        let x = x.saturating_sub(1).min(lado - 1);
        let y = y.saturating_sub(1).min(lado - 1);
        estado[canal * lado * lado + y * lado + x]
    };
    let a = leer(0, 0);
    let b = leer(1, 0);
    let c = leer(0, 1);
    let d = leer(1, 1);
    let inferior = a + (b - a) * pesos[0];
    let superior = c + (d - c) * pesos[0];
    inferior + (superior - inferior) * pesos[1]
}

fn validar(lado: usize, iteraciones: u32, estado: &[f32], p: &[f32; 8]) -> Result<(), &'static str> {
    if !(8..=256).contains(&lado)
        || iteraciones > 128
        || estado.len() != 12 * lado * lado
        || estado.iter().chain(p).any(|v| !v.is_finite())
        || !(0.02..=0.18).contains(&p[0])
        || !(0.0..=10.0).contains(&p[1])
        || !(0.0..=8.0).contains(&p[2])
        || !(-1.0..=1.0).contains(&p[3])
        || !(0.0..=2.0).contains(&p[4])
        || !(0.001..=0.04).contains(&p[5])
    {
        return Err("referencia humo inválida");
    }
    Ok(())
}
