//! Esquema MAC 3D: tres caras, cinco campos transportados y stencil de seis vecinos.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn emitir(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32, iteraciones: u32) {
    match etapa {
        0 => advectar(f, m, pixel),
        1 => {
            let div = divergencia(f, m, pixel, 5);
            m.guardar(f, pixel, 10, div);
        }
        2 | 3 => jacobi(f, m, pixel, etapa == 2),
        4 => proyectar(f, m, pixel, if iteraciones.is_multiple_of(2) { 12 } else { 11 }),
        _ => {
            let div = divergencia(f, m, pixel, 0);
            m.guardar(f, pixel, 13, div);
            tiempo(f, m, pixel);
        }
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn divergencia(f: &mut Funcion, m: &Memoria, pixel: Id, canal: u32) -> Id {
    let vecinos = m.vecinos_fluidos(f, pixel);
    let cero = Memoria::c(f, 0.0);
    let mut suma = cero;
    for eje in 0..3 {
        let anterior = m.leer(f, vecinos[eje * 2].0, canal + u32::try_from(eje).unwrap_or(0));
        let anterior = f.elegir(m.real, vecinos[eje * 2].1, anterior, cero);
        let actual = m.leer(f, pixel, canal + u32::try_from(eje).unwrap_or(0));
        let delta = m.sub(f, actual, anterior);
        suma = m.add(f, suma, delta);
    }
    m.escala(f, suma, m.lado as f32)
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn jacobi(f: &mut Funcion, m: &Memoria, pixel: Id, primera: bool) {
    let (entrada, salida) = if primera { (12, 11) } else { (11, 12) };
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let mut suma = cero;
    let mut cantidad = cero;
    for (vecino, existe) in m.vecinos_fluidos(f, pixel) {
        let valor = m.leer(f, vecino, entrada);
        let valor = f.elegir(m.real, existe, valor, cero);
        suma = m.add(f, suma, valor);
        let incremento = f.elegir(m.real, existe, uno, cero);
        cantidad = m.add(f, cantidad, incremento);
    }
    let div = m.leer(f, pixel, 10);
    let rhs = m.escala(f, div, 1.0 / (m.lado * m.lado) as f32);
    let candidata = m.sub(f, suma, rhs);
    let cantidad = f.glsl(m.real, op::GLSL_MAXIMO, &[cantidad, uno]);
    let candidata = f.binaria(op::F_DIV, m.real, candidata, cantidad);
    let candidata = m.escala(f, candidata, 2.0 / 3.0);
    let antes = m.leer(f, pixel, entrada);
    let antes = m.escala(f, antes, 1.0 / 3.0);
    let valor = m.add(f, candidata, antes);
    let fluido = m.fluido(f, pixel);
    let valor = f.elegir(m.real, fluido, valor, cero);
    m.guardar(f, pixel, salida, valor);
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn proyectar(f: &mut Funcion, m: &Memoria, pixel: Id, presion: u32) {
    let vecinos = m.vecinos_fluidos(f, pixel);
    let centro = m.leer(f, pixel, presion);
    let cero = Memoria::c(f, 0.0);
    for eje in 0..3 {
        let canal = u32::try_from(eje).unwrap_or(0);
        let (vecino, existe) = vecinos[eje * 2 + 1];
        let otro = m.leer(f, vecino, presion);
        let grad = m.sub(f, otro, centro);
        let grad = m.escala(f, grad, m.lado as f32);
        let previo = m.leer(f, pixel, canal + 5);
        let nuevo = m.sub(f, previo, grad);
        let nuevo = f.elegir(m.real, existe, nuevo, cero);
        m.guardar(f, pixel, canal, nuevo);
    }
    for canal in 3..5 {
        let valor = m.leer(f, pixel, canal + 5);
        m.guardar(f, pixel, canal, valor);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn advectar(f: &mut Funcion, m: &Memoria, pixel: Id) {
    let coords = m.coords3(f, pixel).map(|v| f.unaria(op::U_A_F, m.real, v));
    let dt = m.parametro(f, 5);
    let subpasos = m.parametro(f, 13);
    let cero = Memoria::c(f, 0.0);
    let temporal = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, subpasos, cero);
    let indice = Memoria::u(f, 10);
    let efectivo = m.leer(f, indice, m.control());
    let dt = f.elegir(m.real, temporal, efectivo, dt);
    let escala = m.escala(f, dt, m.lado as f32);
    let vecinos = m.vecinos_fluidos(f, pixel);
    let cero = Memoria::c(f, 0.0);
    let medio = Memoria::c(f, 0.5);
    let fuente = mascara(f, m, coords);
    for canal in 0..5 {
        let mut punto = coords;
        if canal < 3 {
            punto[canal as usize] = m.add(f, punto[canal as usize], medio);
        }
        let velocidad = core::array::from_fn::<_, 3, _>(|eje| {
            let mut p = punto;
            p[eje] = m.sub(f, p[eje], medio);
            m.muestra3(f, p, u32::try_from(eje).unwrap_or(0))
        });
        let mut anterior = core::array::from_fn(|eje| {
            let delta = m.mul(f, velocidad[eje], escala);
            m.sub(f, punto[eje], delta)
        });
        if canal < 3 {
            anterior[canal as usize] = m.sub(f, anterior[canal as usize], medio);
        }
        let mut valor = m.muestra3(f, anterior, canal);
        if canal < 3 {
            let fuerza = if canal < 2 {
                let calor = m.muestra3(f, coords, 4);
                let factor = m.parametro(f, if canal == 0 { 3 } else { 2 });
                m.mul(f, calor, factor)
            } else {
                cero
            };
            let impulso = m.mul(f, fuerza, dt);
            valor = m.add(f, valor, impulso);
            valor = m.limitar(f, valor, -0.8, 0.8);
            valor = f.elegir(m.real, vecinos[canal as usize * 2 + 1].1, valor, cero);
        } else {
            let disipacion = if canal == 3 { m.parametro(f, 4) } else { Memoria::c(f, 0.2) };
            let perdida = m.mul(f, disipacion, dt);
            let uno = Memoria::c(f, 1.0);
            let conservar = m.sub(f, uno, perdida);
            valor = m.mul(f, valor, conservar);
            let tasa = m.parametro(f, 1);
            let tasa = m.mul(f, tasa, dt);
            let aporte = m.mul(f, tasa, fuente);
            valor = m.add(f, valor, aporte);
            valor = m.limitar(f, valor, 0.0, 10.0);
        }
        let fluido = m.fluido(f, pixel);
        let valor = f.elegir(m.real, fluido, valor, cero);
        m.guardar(f, pixel, canal + 5, valor);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn mascara(f: &mut Funcion, m: &Memoria, coords: [Id; 3]) -> Id {
    let centro = [(m.lado as f32 - 1.0) * 0.5, m.lado as f32 * 0.18, (m.lado as f32 - 1.0) * 0.5];
    let mut r2 = Memoria::c(f, 0.0);
    for (coord, centro) in coords.into_iter().zip(centro) {
        let centro = Memoria::c(f, centro);
        let delta = m.sub(f, coord, centro);
        let cuadrado = m.mul(f, delta, delta);
        r2 = m.add(f, r2, cuadrado);
    }
    let radio = m.parametro(f, 0);
    let radio = m.escala(f, radio, m.lado as f32);
    let radio2 = m.mul(f, radio, radio);
    let relativo = f.binaria(op::F_DIV, m.real, r2, radio2);
    let uno = Memoria::c(f, 1.0);
    let peso = m.sub(f, uno, relativo);
    let peso = m.limitar(f, peso, 0.0, 1.0);
    m.mul(f, peso, peso)
}

fn tiempo(f: &mut Funcion, m: &Memoria, pixel: Id) {
    let cero_u = Memoria::u(f, 0);
    let primero = f.binaria(op::I_EQUAL, m.booleano, pixel, cero_u);
    let cuerpo = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(primero, cuerpo, fin);
    f.bloque(cuerpo);
    let indice = Memoria::u(f, 10);
    let dt = m.leer(f, indice, m.control());
    let indice = Memoria::u(f, 5);
    let estado = m.leer(f, indice, m.control());
    let tres = Memoria::c(f, 3.0);
    let valido = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, tres, estado);
    let cero = Memoria::c(f, 0.0);
    let dt = f.elegir(m.real, valido, dt, cero);
    let indice = Memoria::u(f, 13);
    let previo = m.leer(f, indice, m.control());
    let nuevo = m.add(f, previo, dt);
    m.guardar(f, indice, m.control(), nuevo);
    f.saltar(fin);
    f.bloque(fin);
}
