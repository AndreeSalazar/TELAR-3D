//! Referencia escalar independiente de la emisión SSA; solo para ensayos pequeños.
use alloc::vec::Vec;
/// Avanza advección/fuerzas y el número indicado de iteraciones de presión.
/// # Errors
/// Rechaza dominios, datos no finitos y parámetros físicos fuera de rango.
pub fn paso(lado: usize, iteraciones: u32, estado: &mut [f32], p: &[f32; 8]) -> Result<(), &'static str> {
    paso_con_obstaculo(lado, iteraciones, estado, p, [0.0, 0.52])
}
/// Referencia con esfera voxelizada de radio/altura normalizados; radio cero la desactiva.
/// # Errors
/// Rechaza estado, dt o esfera fuera del dominio.
pub fn paso_con_obstaculo(
    lado: usize,
    iteraciones: u32,
    estado: &mut [f32],
    p: &[f32; 8],
    esfera: [f32; 2],
) -> Result<(), &'static str> {
    if !(4..=64).contains(&lado)
        || iteraciones > 128
        || estado.len() != 17 * lado * lado * lado
        || estado.iter().chain(p).any(|v| !v.is_finite())
        || !(0.02..=0.18).contains(&p[0])
        || !(0.0..=10.0).contains(&p[1])
        || !(0.0..=8.0).contains(&p[2])
        || !(-1.0..=1.0).contains(&p[3])
        || !(0.0..=2.0).contains(&p[4])
        || !(0.000_000_1..=0.04).contains(&p[5])
        || esfera.iter().any(|v| !v.is_finite())
        || !(0.0..=0.3).contains(&esfera[0])
        || !(0.31..=0.69).contains(&esfera[1])
    {
        return Err("referencia 3D inválida");
    }
    let n = lado * lado * lado;
    let fluidos: Vec<_> = (0..n)
        .map(|i| {
            let coords = coords(lado, i);
            #[allow(clippy::cast_precision_loss, reason = "lado <=64")]
            let distancia = coords
                .into_iter()
                .zip([0.5, esfera[1], 0.5])
                .map(|(v, c)| {
                    let d = (v as f32 + 0.5) / lado as f32 - c;
                    d * d
                })
                .sum::<f32>();
            esfera[0] == 0.0 || distancia > esfera[0] * esfera[0]
        })
        .collect();
    let temporal = advectar(lado, estado, p, &fluidos);
    estado[5 * n..10 * n].copy_from_slice(&temporal);
    let div = divergencia(lado, &temporal);
    estado[10 * n..11 * n].copy_from_slice(&div);
    let mut anterior = estado[12 * n..13 * n].to_vec();
    let mut siguiente = anterior.clone();
    for iter in 0..iteraciones {
        resolver(lado, &anterior, &div, &mut siguiente, &fluidos);
        let plano = if iter % 2 == 0 { 11 } else { 12 };
        estado[plano * n..(plano + 1) * n].copy_from_slice(&siguiente);
        core::mem::swap(&mut anterior, &mut siguiente);
    }
    proyectar(lado, &temporal, &anterior, estado, &fluidos);
    estado[3 * n..5 * n].copy_from_slice(&temporal[3 * n..5 * n]);
    let posterior = divergencia(lado, estado);
    estado[13 * n..14 * n].copy_from_slice(&posterior);
    Ok(())
}
fn coords(lado: usize, pixel: usize) -> [usize; 3] {
    [pixel % lado, (pixel / lado) % lado, pixel / (lado * lado)]
}
fn vecinos(lado: usize, pixel: usize) -> [Option<usize>; 6] {
    let posicion = coords(lado, pixel);
    let mut salida = [None; 6];
    for eje in 0..3 {
        let paso = [1, lado, lado * lado][eje];
        if posicion[eje] > 0 {
            salida[eje * 2] = Some(pixel - paso);
        }
        if posicion[eje] + 1 < lado {
            salida[eje * 2 + 1] = Some(pixel + paso);
        }
    }
    salida
}
fn abiertos(lado: usize, pixel: usize, fluidos: &[bool]) -> [Option<usize>; 6] {
    vecinos(lado, pixel).map(|v| v.filter(|i| fluidos[pixel] && fluidos[*i]))
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn advectar(lado: usize, estado: &[f32], p: &[f32; 8], fluidos: &[bool]) -> Vec<f32> {
    let n = lado * lado * lado;
    let mut salida = alloc::vec![0.0;5*n];
    let escala = lado as f32;
    for pixel in 0..n {
        let posicion = coords(lado, pixel).map(|v| v as f32);
        let contorno = abiertos(lado, pixel, fluidos);
        if !fluidos[pixel] {
            continue;
        }
        let centro = [(escala - 1.0) * 0.5, escala * 0.18, (escala - 1.0) * 0.5];
        let r2 = posicion.into_iter().zip(centro).map(|(v, c)| (v - c) * (v - c)).sum::<f32>();
        let radio = p[0] * escala;
        let mascara = (1.0 - r2 / (radio * radio)).clamp(0.0, 1.0);
        let fuente = mascara * mascara;
        for canal in 0..5 {
            let mut punto = posicion;
            if canal < 3 {
                punto[canal] += 0.5;
            }
            let velocidad = core::array::from_fn::<_, 3, _>(|eje| {
                let mut punto = punto;
                punto[eje] -= 0.5;
                muestra(estado, lado, punto, eje, fluidos)
            });
            let mut antes = core::array::from_fn(|eje| punto[eje] - velocidad[eje] * p[5] * escala);
            if canal < 3 {
                antes[canal] -= 0.5;
            }
            let mut valor = muestra(estado, lado, antes, canal, fluidos);
            if canal < 3 {
                let fuerza = if canal < 2 {
                    muestra(estado, lado, posicion, 4, fluidos) * p[if canal == 0 { 3 } else { 2 }]
                } else {
                    0.0
                };
                valor = (valor + fuerza * p[5]).clamp(-0.8, 0.8);
                if contorno[canal * 2 + 1].is_none() {
                    valor = 0.0;
                }
            } else {
                let disipacion = if canal == 3 { p[4] } else { 0.2 };
                valor = (valor * (1.0 - disipacion * p[5]) + p[1] * p[5] * fuente).clamp(0.0, 10.0);
            }
            salida[canal * n + pixel] = valor;
        }
    }
    salida
}
/// Divergencia de las tres caras MAC con flujo exterior cero.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
pub fn divergencia(lado: usize, estado: &[f32]) -> Vec<f32> {
    if !(1..=64).contains(&lado) || estado.len() < 3 * lado * lado * lado {
        return Vec::new();
    }
    let n = lado * lado * lado;
    let mut salida = alloc::vec![0.0;n];
    for pixel in 0..n {
        let vecinos = vecinos(lado, pixel);
        let mut suma = 0.0;
        for eje in 0..3 {
            let anterior = vecinos[eje * 2].map_or(0.0, |i| estado[eje * n + i]);
            suma += estado[eje * n + pixel] - anterior;
        }
        salida[pixel] = suma * lado as f32;
    }
    salida
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn resolver(lado: usize, anterior: &[f32], div: &[f32], salida: &mut [f32], fluidos: &[bool]) {
    for pixel in 0..anterior.len() {
        let mut suma = 0.0;
        let mut cantidad = 0.0;
        for vecino in abiertos(lado, pixel, fluidos).into_iter().flatten() {
            suma += anterior[vecino];
            cantidad += 1.0;
        }
        let candidata = (suma - div[pixel] / (lado * lado) as f32) / f32::max(cantidad, 1.0);
        if !fluidos[pixel] {
            salida[pixel] = 0.0;
            continue;
        }
        salida[pixel] = candidata * (2.0 / 3.0) + anterior[pixel] * (1.0 / 3.0);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn proyectar(lado: usize, provisional: &[f32], presion: &[f32], salida: &mut [f32], fluidos: &[bool]) {
    let n = lado * lado * lado;
    for pixel in 0..n {
        for (eje, vecino) in abiertos(lado, pixel, fluidos).into_iter().skip(1).step_by(2).enumerate() {
            salida[eje * n + pixel] =
                vecino.map_or(0.0, |i| provisional[eje * n + pixel] - (presion[i] - presion[pixel]) * lado as f32);
        }
    }
}
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "coordenadas tras clamp al dominio <=64"
)]
fn muestra(estado: &[f32], lado: usize, coords: [f32; 3], canal: usize, fluidos: &[bool]) -> f32 {
    let halo = canal < 3;
    let minimo = if halo { -1.0 } else { 0.0 };
    let coords = coords.map(|v| v.clamp(minimo, (lado - 1) as f32) + 1.0);
    let bases = coords.map(|v| v as usize);
    let pesos = core::array::from_fn::<_, 3, _>(|i| coords[i] - bases[i] as f32);
    let mut valores = [0.0; 8];
    for (i, valor) in valores.iter_mut().enumerate() {
        let indices = core::array::from_fn::<_, 3, _>(|eje| bases[eje] + ((i >> eje) & 1));
        if halo && indices.contains(&0) {
            continue;
        }
        let [x, y, z] = indices.map(|v| v.saturating_sub(1).min(lado - 1));
        let pixel = (z * lado + y) * lado + x;
        *valor = if fluidos[pixel] { estado[canal * lado * lado * lado + pixel] } else { 0.0 };
    }
    for (eje, peso) in pesos.into_iter().enumerate() {
        for i in 0..(4 >> eje) {
            valores[i] = valores[i * 2] + (valores[i * 2 + 1] - valores[i * 2]) * peso;
        }
    }
    valores[0]
}
