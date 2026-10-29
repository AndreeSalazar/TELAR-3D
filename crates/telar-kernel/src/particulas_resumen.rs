//! Reducción determinista sin atómicos, después de actualizar todas las plazas.
use crate::particulas_memoria::Estado;
use hilo_spirv::{Funcion, op};
pub(super) fn emitir(f: &mut Funcion, e: &Estado) {
    let cero = Estado::c(f, 0.0);
    let uno = Estado::c(f, 1.0);
    let inicial = f.construir(e.vector, &[cero; 4]);
    let resumen = f.repetir(crate::particulas::CAPACIDAD, e.vector, inicial, |f, id, previo| {
        let activa = e.leer(f, id, 10);
        let viva = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, activa, cero);
        let mala = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, cero, activa);
        let generacion = e.leer(f, id, 8);
        let generacion = f.elegir(e.real, mala, cero, generacion);
        let cuenta = f.elegir(e.real, viva, uno, cero);
        let malas = f.elegir(e.real, mala, uno, cero);
        let mut velocidad = cero;
        for eje in 3..6 {
            let v = e.leer(f, id, eje);
            let v = f.glsl(e.real, 4, &[v]);
            velocidad = e.add(f, velocidad, v);
        }
        let velocidad = f.elegir(e.real, viva, velocidad, cero);
        let mut valores = [cuenta, generacion, malas, velocidad];
        for (i, valor) in valores.iter_mut().enumerate() {
            let antes = f.extraer(e.real, previo, u32::try_from(i).unwrap_or(0));
            *valor = if i == 3 { f.glsl(e.real, op::GLSL_MAXIMO, &[antes, *valor]) } else { e.add(f, antes, *valor) };
        }
        f.construir(e.vector, &valores)
    });
    let dt = e.dt(f);
    let tiempo = e.control(f, 0);
    let tiempo = e.add(f, tiempo, dt);
    guardar(f, e, 0, tiempo);
    for i in 0..4 {
        let valor = f.extraer(e.real, resumen, i);
        guardar(f, e, i + 1, valor);
    }
    let cantidad = e.p(f, 0);
    guardar(f, e, 5, cantidad);
    guardar(f, e, 6, dt);
    let limite = Estado::c(f, 3600.0);
    let terminado = f.binaria(190, e.booleano, tiempo, limite);
    let terminado = f.elegir(e.real, terminado, uno, cero);
    guardar(f, e, 7, terminado);
}
fn guardar(f: &mut Funcion, e: &Estado, i: u32, v: hilo_spirv::Id) {
    let id = Estado::u(f, i);
    e.guardar(f, id, crate::particulas::CONTROL, v);
}
