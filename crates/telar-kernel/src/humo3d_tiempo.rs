//! Predictor CFL residente: reducción de máximos por componente antes de cada subpaso.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn emitir(f: &mut Funcion, m: &Memoria, pixel: Id, etapa: u32) {
    if etapa == 15 {
        parciales(f, m, pixel);
        crate::humo3d_fuente::parciales(f, m, pixel);
    } else {
        decidir(f, m);
    }
}
fn parciales(f: &mut Funcion, m: &Memoria, grupo: Id) {
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
        let valores = [0, 1, 2, 4].map(|canal| {
            let valor = m.leer(f, pixel, canal);
            let valor = f.glsl(m.real, 4, &[valor]);
            let limite = Memoria::c(f, 1e20);
            let valido = f.binaria(188, m.booleano, valor, limite);
            let enorme = Memoria::c(f, 1e30);
            let valor = f.elegir(m.real, valido, valor, enorme);
            f.elegir(m.real, existe, valor, cero)
        });
        let valores = f.construir(m.vector, &valores);
        f.glsl(m.vector, op::GLSL_MAXIMO, &[previo, valores])
    });
    let cuatro = Memoria::u(f, 6);
    let base = f.binaria(op::I_MUL, m.entero, grupo, cuatro);
    for i in 0..4 {
        let indice = Memoria::u(f, i);
        let indice = f.binaria(op::I_ADD, m.entero, base, indice);
        let valor = f.extraer(m.real, resultado, i);
        m.guardar(f, indice, m.parciales(), valor);
    }
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn decidir(f: &mut Funcion, m: &Memoria) {
    let cero = Memoria::c(f, 0.0);
    let inicial = f.construir(m.vector, &[cero; 4]);
    let maximos = f.repetir(m.celdas().div_ceil(64), m.vector, inicial, |f, grupo, previo| {
        let cuatro = Memoria::u(f, 6);
        let base = f.binaria(op::I_MUL, m.entero, grupo, cuatro);
        let valores = core::array::from_fn::<_, 4, _>(|i| {
            let indice = Memoria::u(f, u32::try_from(i).unwrap_or(0));
            let indice = f.binaria(op::I_ADD, m.entero, base, indice);
            m.leer(f, indice, m.parciales())
        });
        let valores = f.construir(m.vector, &valores);
        f.glsl(m.vector, op::GLSL_MAXIMO, &[previo, valores])
    });
    let media = crate::humo3d_fuente::media(f, m);
    let indice = Memoria::u(f, 15);
    m.guardar(f, indice, m.control(), media);
    let indice = Memoria::u(f, 13);
    let inicio = m.leer(f, indice, m.control());
    let indice = Memoria::u(f, 16);
    m.guardar(f, indice, m.control(), inicio);
    let dt = m.parametro(f, 5);
    let subpasos = m.parametro(f, 13);
    let uno = Memoria::c(f, 1.0);
    let subpasos = f.glsl(m.real, op::GLSL_MAXIMO, &[subpasos, uno]);
    let solicitado = f.binaria(op::F_DIV, m.real, dt, subpasos);
    let mut velocidad = cero;
    for eje in 0..3 {
        let componente = f.extraer(m.real, maximos, eje);
        velocidad = m.add(f, velocidad, componente);
    }
    let calor = f.extraer(m.real, maximos, 3);
    let flotacion = m.parametro(f, 2);
    let viento = m.parametro(f, 3);
    let viento = f.glsl(m.real, 4, &[viento]);
    let fuerzas = m.add(f, flotacion, viento);
    let fuerzas = if m.vfx {
        let detalle = m.ajuste(f, 24);
        let detalle = m.escala(f, detalle, 4.5);
        m.add(f, fuerzas, detalle)
    } else {
        fuerzas
    };
    let fuerzas = m.mul(f, fuerzas, calor);
    let impulso = m.mul(f, fuerzas, solicitado);
    let cota = m.add(f, velocidad, impulso);
    let minimo = Memoria::c(f, 1e-6);
    let denominador = f.glsl(m.real, op::GLSL_MAXIMO, &[cota, minimo]);
    let denominador = m.escala(f, denominador, m.lado as f32);
    let cfl = m.parametro(f, 19);
    let limitado = f.binaria(op::F_DIV, m.real, cfl, denominador);
    let efectivo = f.glsl(m.real, 37, &[solicitado, limitado]);
    let limite = Memoria::c(f, 1e20);
    let valido = f.binaria(188, m.booleano, cota, limite);
    let efectivo = f.elegir(m.real, valido, efectivo, cero);
    let cortado = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, solicitado, efectivo);
    let estado = f.elegir(m.real, cortado, uno, cero);
    let dos = Memoria::c(f, 2.0);
    let estado = f.elegir(m.real, valido, estado, dos);
    let desplazamiento = m.mul(f, efectivo, denominador);
    for (i, valor) in [(10, efectivo), (11, cota), (12, desplazamiento), (14, estado)] {
        let indice = Memoria::u(f, i);
        m.guardar(f, indice, m.control(), valor);
    }
}
