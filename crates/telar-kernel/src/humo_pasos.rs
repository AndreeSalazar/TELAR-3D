//! Pasadas del solver; planos de lectura/escritura distintos entre vecindarios.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn emitir(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32, iteraciones: u32) {
    match etapa {
        0 => advectar(f, m, pixel),
        1 => {
            let valor = divergencia(f, m, pixel, 4);
            m.guardar(f, pixel, 8, valor);
        }
        2 | 3 => jacobi(f, m, pixel, etapa == 2),
        4 => proyectar(f, m, pixel, if iteraciones % 2 == 1 { 9 } else { 10 }),
        _ => {
            let valor = divergencia(f, m, pixel, 0);
            m.guardar(f, pixel, 11, valor);
        }
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn divergencia(f: &mut Funcion, m: &Memoria, pixel: Id, canal: u32) -> Id {
    let vecinos = m.vecinos(f, pixel);
    let cero = Memoria::c(f, 0.0);
    let centro_x = m.leer(f, pixel, canal);
    let centro_y = m.leer(f, pixel, canal + 1);
    let izquierda = m.leer(f, vecinos[0].0, canal);
    let abajo = m.leer(f, vecinos[2].0, canal + 1);
    let izquierda = f.elegir(m.real, vecinos[0].1, izquierda, cero);
    let abajo = f.elegir(m.real, vecinos[2].1, abajo, cero);
    let dx = m.sub(f, centro_x, izquierda);
    let dy = m.sub(f, centro_y, abajo);
    let suma = m.add(f, dx, dy);
    m.escala(f, suma, m.lado as f32)
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn jacobi(f: &mut Funcion, m: &Memoria, pixel: Id, primera: bool) {
    let (entrada, salida) = if primera { (10, 9) } else { (9, 10) };
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let mut suma = cero;
    let mut cantidad = cero;
    for (vecino, existe) in m.vecinos(f, pixel) {
        let valor = m.leer(f, vecino, entrada);
        let valor = f.elegir(m.real, existe, valor, cero);
        suma = m.add(f, suma, valor);
        let incremento = f.elegir(m.real, existe, uno, cero);
        cantidad = m.add(f, cantidad, incremento);
    }
    let div = m.leer(f, pixel, 8);
    let rhs = m.escala(f, div, 1.0 / (m.lado * m.lado) as f32);
    let valor = m.sub(f, suma, rhs);
    let valor = f.binaria(op::F_DIV, m.real, valor, cantidad);
    let candidata = m.escala(f, valor, 2.0 / 3.0);
    let previo = m.leer(f, pixel, entrada);
    let previo = m.escala(f, previo, 1.0 / 3.0);
    let valor = m.add(f, candidata, previo);
    m.guardar(f, pixel, salida, valor);
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn proyectar(f: &mut Funcion, m: &Memoria, pixel: Id, presion: u32) {
    let vecinos = m.vecinos(f, pixel);
    let centro = m.leer(f, pixel, presion);
    let cero = Memoria::c(f, 0.0);
    for (canal, (vecino, existe)) in [(0, vecinos[1]), (1, vecinos[3])] {
        let otro = m.leer(f, vecino, presion);
        let delta = m.sub(f, otro, centro);
        let grad = m.escala(f, delta, m.lado as f32);
        let previo = m.leer(f, pixel, canal + 4);
        let nuevo = m.sub(f, previo, grad);
        let nuevo = f.elegir(m.real, existe, nuevo, cero);
        m.guardar(f, pixel, canal, nuevo);
    }
    for canal in 2..4 {
        let valor = m.leer(f, pixel, canal + 4);
        m.guardar(f, pixel, canal, valor);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn advectar(f: &mut Funcion, m: &Memoria, pixel: Id) {
    let coords = m.coords(f, pixel).map(|v| f.unaria(op::U_A_F, m.real, v));
    let paso = m.parametro(f, 5);
    let escala = m.escala(f, paso, m.lado as f32);
    let vecinos = m.vecinos(f, pixel);
    let cero = Memoria::c(f, 0.0);
    let medio = Memoria::c(f, 0.5);
    let fuente = mascara(f, m, coords);
    for canal in 0..4 {
        let mut punto = coords;
        if canal < 2 {
            punto[canal as usize] = m.add(f, punto[canal as usize], medio);
        }
        let ux = m.sub(f, punto[0], medio);
        let vy = m.sub(f, punto[1], medio);
        let velocidad = [m.muestra(f, [ux, punto[1]], 0), m.muestra(f, [punto[0], vy], 1)];
        let anterior = core::array::from_fn(|i| {
            let delta = m.mul(f, velocidad[i], escala);
            m.sub(f, punto[i], delta)
        });
        let anterior = if canal < 2 {
            let mut p = anterior;
            p[canal as usize] = m.sub(f, p[canal as usize], medio);
            p
        } else {
            anterior
        };
        let mut valor = m.muestra(f, anterior, canal);
        if canal < 2 {
            let fuerza = if canal == 0 {
                let viento = m.parametro(f, 3);
                let calor = m.muestra(f, coords, 3);
                m.mul(f, viento, calor)
            } else {
                let calor = m.muestra(f, coords, 3);
                let flotacion = m.parametro(f, 2);
                m.mul(f, calor, flotacion)
            };
            let impulso = m.mul(f, fuerza, paso);
            valor = m.add(f, valor, impulso);
            valor = m.limitar(f, valor, -0.8, 0.8);
            let borde = vecinos[if canal == 0 { 1 } else { 3 }].1;
            valor = f.elegir(m.real, borde, valor, cero);
        } else {
            let disipacion = if canal == 2 { m.parametro(f, 4) } else { Memoria::c(f, 0.2) };
            let perdida = m.mul(f, disipacion, paso);
            let uno = Memoria::c(f, 1.0);
            let conservar = m.sub(f, uno, perdida);
            valor = m.mul(f, valor, conservar);
            let tasa = m.parametro(f, 1);
            let cantidad = m.mul(f, tasa, paso);
            let cantidad = m.mul(f, cantidad, fuente);
            valor = m.add(f, valor, cantidad);
            valor = m.limitar(f, valor, 0.0, 10.0);
        }
        m.guardar(f, pixel, canal + 4, valor);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn mascara(f: &mut Funcion, m: &Memoria, coords: [Id; 2]) -> Id {
    let centro = [Memoria::c(f, (m.lado as f32 - 1.0) * 0.5), Memoria::c(f, m.lado as f32 * 0.18)];
    let delta = core::array::from_fn::<_, 2, _>(|i| m.sub(f, coords[i], centro[i]));
    let cuadrados = delta.map(|v| m.mul(f, v, v));
    let r2 = m.add(f, cuadrados[0], cuadrados[1]);
    let radio = m.parametro(f, 0);
    let radio = m.escala(f, radio, m.lado as f32);
    let radio2 = m.mul(f, radio, radio);
    let relativo = f.binaria(op::F_DIV, m.real, r2, radio2);
    let uno = Memoria::c(f, 1.0);
    let peso = m.sub(f, uno, relativo);
    let peso = m.limitar(f, peso, 0.0, 1.0);
    m.mul(f, peso, peso)
}
