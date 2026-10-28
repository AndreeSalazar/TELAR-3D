//! Convergencia global sin lectura host: máximos jerárquicos y decisión de una invocación.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn emitir(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32) {
    if etapa >= 9 {
        let cero = Memoria::u(f, 0);
        let indice = if etapa == 14 { Memoria::u(f, 5) } else { cero };
        let activo = m.leer(f, indice, m.control());
        let cero = Memoria::c(f, 0.0);
        let activo = if etapa == 14 {
            let tres = Memoria::c(f, 3.0);
            f.binaria(op::F_ORD_GREATER_THAN, m.booleano, tres, activo)
        } else {
            f.binaria(op::F_ORD_GREATER_THAN, m.booleano, activo, cero)
        };
        let cuerpo = f.etiqueta();
        let fin = f.etiqueta();
        f.fusion(fin);
        f.saltar_si(activo, cuerpo, fin);
        f.bloque(cuerpo);
        contenido(f, m, pixel, etapa);
        f.saltar(fin);
        f.bloque(fin);
    } else {
        contenido(f, m, pixel, etapa);
    }
}
fn contenido(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32) {
    match etapa {
        6 | 11 => residual(f, m, pixel),
        7 | 12 => parciales(f, m, pixel),
        9 => paso(f, m, pixel, 2),
        10 => paso(f, m, pixel, 3),
        14 => paso(f, m, pixel, 4),
        _ => decidir(f, m, etapa == 8),
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn residual(f: &mut Funcion, m: &Memoria, pixel: Id) {
    let centro = m.leer(f, pixel, m.qb());
    let mut laplaciano = Memoria::c(f, 0.0);
    let cero = laplaciano;
    for (vecino, existe) in m.vecinos_todos(f, pixel) {
        let otro = m.leer(f, vecino, m.qb());
        let delta = m.sub(f, otro, centro);
        let delta = f.elegir(m.real, existe, delta, cero);
        laplaciano = m.add(f, laplaciano, delta);
    }
    let laplaciano = m.escala(f, laplaciano, (m.lado * m.lado) as f32);
    let div = m.leer(f, pixel, m.div());
    let valor = m.sub(f, div, laplaciano);
    let valor = seguro(f, m, valor);
    m.guardar(f, pixel, m.residual(), valor);
}
fn seguro(f: &mut Funcion, m: &Memoria, valor: Id) -> Id {
    let absoluto = f.glsl(m.real, 4, &[valor]);
    let limite = Memoria::c(f, 1e20);
    let valido = f.binaria(188, m.booleano, absoluto, limite); // OpFOrdLessThanEqual: NaN queda rechazado.
    let invalido = Memoria::c(f, 1e30);
    f.elegir(m.real, valido, absoluto, invalido)
}
fn parciales(f: &mut Funcion, m: &Memoria, grupo: Id) {
    let cero = Memoria::c(f, 0.0);
    let inicial = f.construir(m.vector, &[cero; 4]);
    let paso = Memoria::u(f, 64);
    let base = f.binaria(op::I_MUL, m.entero, grupo, paso);
    let cantidad = Memoria::u(f, m.celdas());
    let max = Memoria::u(f, m.celdas() - 1);
    let resultado = f.repetir(64, m.vector, inicial, |f, indice, acumulado| {
        let pixel = f.binaria(op::I_ADD, m.entero, base, indice);
        let existe = f.binaria(op::U_LESS_THAN, m.booleano, pixel, cantidad);
        let pixel = f.glsl(m.entero, 38, &[pixel, max]);
        let valores =
            [m.div(), m.residual(), if m.volumen { 8 } else { 6 }, if m.volumen { 9 } else { 7 }].map(|canal| {
                let valor = m.leer(f, pixel, canal);
                let valor = seguro(f, m, valor);
                f.elegir(m.real, existe, valor, cero)
            });
        let vector = f.construir(m.vector, &valores);
        f.glsl(m.vector, op::GLSL_MAXIMO, &[acumulado, vector])
    });
    let cuatro = Memoria::u(f, 4);
    let base = f.binaria(op::I_MUL, m.entero, grupo, cuatro);
    for canal in 0..4 {
        let offset = Memoria::u(f, canal);
        let destino = f.binaria(op::I_ADD, m.entero, base, offset);
        let valor = f.extraer(m.real, resultado, canal);
        m.guardar(f, destino, m.parciales(), valor);
    }
}
fn resumen(f: &mut Funcion, m: &Memoria) -> Id {
    let cero = Memoria::c(f, 0.0);
    let inicial = f.construir(m.vector, &[cero; 4]);
    f.repetir(m.celdas().div_ceil(64), m.vector, inicial, |f, grupo, anterior| {
        let cuatro = Memoria::u(f, 4);
        let base = f.binaria(op::I_MUL, m.entero, grupo, cuatro);
        let valores = core::array::from_fn::<_, 4, _>(|i| {
            let offset = Memoria::u(f, u32::try_from(i).unwrap_or(0));
            let indice = f.binaria(op::I_ADD, m.entero, base, offset);
            m.leer(f, indice, m.parciales())
        });
        let vector = f.construir(m.vector, &valores);
        f.glsl(m.vector, op::GLSL_MAXIMO, &[anterior, vector])
    })
}
fn control(f: &mut Funcion, m: &Memoria, i: u32) -> Id {
    let indice = Memoria::u(f, i);
    m.leer(f, indice, m.control())
}
fn guardar(f: &mut Funcion, m: &Memoria, i: u32, v: Id) {
    let indice = Memoria::u(f, i);
    m.guardar(f, indice, m.control(), v);
}
fn decidir(f: &mut Funcion, m: &Memoria, inicial: bool) {
    let maximos = resumen(f, m);
    let div = f.extraer(m.real, maximos, 0);
    let residuo = f.extraer(m.real, maximos, 1);
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let dos = Memoria::c(f, 2.0);
    let tres = Memoria::c(f, 3.0);
    let iteraciones = if inicial {
        cero
    } else {
        let antes = control(f, m, 1);
        m.add(f, antes, dos)
    };
    let tolerancia = if inicial {
        let relativa = m.parametro(f, 11);
        let absoluta = m.parametro(f, 12);
        let relativa = m.mul(f, relativa, div);
        f.glsl(m.real, op::GLSL_MAXIMO, &[absoluta, relativa])
    } else {
        control(f, m, 4)
    };
    let convergido = f.binaria(188, m.booleano, residuo, tolerancia);
    let maximo = m.parametro(f, 10);
    let agotado = f.binaria(190, m.booleano, iteraciones, maximo); // OpFOrdGreaterThanEqual.
    let mut mayor = f.glsl(m.real, op::GLSL_MAXIMO, &[div, residuo]);
    for canal in 2..4 {
        let valor = f.extraer(m.real, maximos, canal);
        mayor = f.glsl(m.real, op::GLSL_MAXIMO, &[mayor, valor]);
    }
    let limite = Memoria::c(f, 1e20);
    let error = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, mayor, limite);
    let estado = f.elegir(m.real, agotado, dos, cero);
    let estado = f.elegir(m.real, convergido, uno, estado);
    let estado = f.elegir(m.real, error, tres, estado);
    let activo = f.binaria(180, m.booleano, estado, cero);
    let activo = f.elegir(m.real, activo, uno, cero);
    for (i, v) in [(0, activo), (1, iteraciones), (2, div), (3, residuo), (4, tolerancia), (5, estado)] {
        guardar(f, m, i, v);
    }
    for i in 2..4 {
        let valor = f.extraer(m.real, maximos, i);
        guardar(f, m, i + 4, valor);
    }
    if inicial {
        let paso = control(f, m, 8);
        let paso = m.add(f, paso, uno);
        guardar(f, m, 8, paso);
        guardar(f, m, 9, residuo);
    }
}

fn paso(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32) {
    if m.volumen {
        crate::humo3d_pasos::emitir(f, m, pixel, etapa, 2);
    } else {
        crate::humo_pasos::emitir(f, m, pixel, etapa, 2);
    }
}
