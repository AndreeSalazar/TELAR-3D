//! Pulso finito y divergencia objetivo compensada en el dominio cerrado.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn pulso(f: &mut Funcion, m: &Memoria) -> (Id, Id) {
    let duracion = m.parametro(f, 8);
    let cero = Memoria::c(f, 0.0);
    let activo = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, duracion, cero);
    let indice = Memoria::u(f, 16);
    let inicio = m.leer(f, indice, m.control());
    let indice = Memoria::u(f, 10);
    let dt = m.leer(f, indice, m.control());
    let restante = m.sub(f, duracion, inicio);
    let restante = f.glsl(m.real, op::GLSL_MAXIMO, &[restante, cero]);
    let solapamiento = f.glsl(m.real, 37, &[restante, dt]);
    let minimo = Memoria::c(f, 1e-8);
    let divisor = f.glsl(m.real, op::GLSL_MAXIMO, &[dt, minimo]);
    let fraccion = f.binaria(op::F_DIV, m.real, solapamiento, divisor);
    let uno = Memoria::c(f, 1.0);
    (activo, f.elegir(m.real, activo, fraccion, uno))
}
pub(super) fn mascara(f: &mut Funcion, m: &Memoria, pixel: Id) -> Id {
    let coords = m.coords3(f, pixel).map(|v| f.unaria(op::U_A_F, m.real, v));
    crate::humo3d_pasos::mascara(f, m, coords)
}
pub(super) fn objetivo(f: &mut Funcion, m: &Memoria, pixel: Id) -> Id {
    let local = mascara(f, m, pixel);
    let indice = Memoria::u(f, 15);
    let media = m.leer(f, indice, m.control());
    let local = m.sub(f, local, media);
    let (activo, pulso) = pulso(f, m);
    let magnitud = m.parametro(f, 23);
    let magnitud = m.mul(f, magnitud, pulso);
    let valor = m.mul(f, local, magnitud);
    let fluido = m.fluido(f, pixel);
    let valido = f.binaria(op::LOGICAL_AND, m.booleano, activo, fluido);
    let cero = Memoria::c(f, 0.0);
    f.elegir(m.real, valido, valor, cero)
}
/// Suma espacial de máscara y cantidad de celdas fluidas por grupo, sin atómicos.
pub(super) fn parciales(f: &mut Funcion, m: &Memoria, grupo: Id) {
    let cero = Memoria::c(f, 0.0);
    let inicial = f.construir(m.vector, &[cero; 4]);
    let sesenta = Memoria::u(f, 64);
    let base = f.binaria(op::I_MUL, m.entero, grupo, sesenta);
    let cantidad = Memoria::u(f, m.celdas());
    let maximo = Memoria::u(f, m.celdas() - 1);
    let resultado = f.repetir(64, m.vector, inicial, |f, i, previo| {
        let pixel = f.binaria(op::I_ADD, m.entero, base, i);
        let existe = f.binaria(op::U_LESS_THAN, m.booleano, pixel, cantidad);
        let pixel = f.glsl(m.entero, 38, &[pixel, maximo]);
        let fluido = m.fluido(f, pixel);
        let valido = f.binaria(op::LOGICAL_AND, m.booleano, existe, fluido);
        let valor = mascara(f, m, pixel);
        let valor = f.elegir(m.real, valido, valor, cero);
        let uno = Memoria::c(f, 1.0);
        let cuenta = f.elegir(m.real, valido, uno, cero);
        let vector = f.construir(m.vector, &[valor, cuenta, cero, cero]);
        f.binaria(op::F_ADD, m.vector, previo, vector)
    });
    let seis = Memoria::u(f, 6);
    let base = f.binaria(op::I_MUL, m.entero, grupo, seis);
    for i in 0..2 {
        let indice = Memoria::u(f, i + 4);
        let indice = f.binaria(op::I_ADD, m.entero, base, indice);
        let valor = f.extraer(m.real, resultado, i);
        m.guardar(f, indice, m.parciales(), valor);
    }
}
pub(super) fn media(f: &mut Funcion, m: &Memoria) -> Id {
    let cero = Memoria::c(f, 0.0);
    let inicial = f.construir(m.vector, &[cero; 4]);
    let suma = f.repetir(m.celdas().div_ceil(64), m.vector, inicial, |f, grupo, previo| {
        let seis = Memoria::u(f, 6);
        let base = f.binaria(op::I_MUL, m.entero, grupo, seis);
        let mut valores = [cero; 4];
        for i in 0..2 {
            let indice = Memoria::u(f, i + 4);
            let indice = f.binaria(op::I_ADD, m.entero, base, indice);
            valores[i as usize] = m.leer(f, indice, m.parciales());
        }
        let vector = f.construir(m.vector, &valores);
        f.binaria(op::F_ADD, m.vector, previo, vector)
    });
    let valor = f.extraer(m.real, suma, 0);
    let cuenta = f.extraer(m.real, suma, 1);
    let uno = Memoria::c(f, 1.0);
    let cuenta = f.glsl(m.real, op::GLSL_MAXIMO, &[cuenta, uno]);
    f.binaria(op::F_DIV, m.real, valor, cuenta)
}
