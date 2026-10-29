//! Respuesta inercial al gas e historial propietario de cuatro posiciones.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn semilla(f: &mut Funcion, m: &Memoria, id: Id, generacion: Id) -> Id {
    let semilla = m.ajuste(f, 8);
    let semilla = f.unaria(op::F_A_U, m.entero, semilla);
    let generacion = f.unaria(op::F_A_U, m.entero, generacion);
    let factor = Memoria::u(f, 17);
    let generacion = f.binaria(op::I_MUL, m.entero, generacion, factor);
    let indice = f.binaria(op::I_ADD, m.entero, id, semilla);
    let indice = f.binaria(op::I_ADD, m.entero, indice, generacion);
    let cantidad = Memoria::u(f, 128);
    f.binaria(op::U_MOD, m.entero, indice, cantidad)
}
pub(super) fn velocidad(f: &mut Funcion, m: &Memoria, v: [Id; 3], previo: &[Id; 11], nace: Id, dt: Id) -> [Id; 3] {
    let tasa = m.ajuste(f, 10);
    let tasa = m.mul(f, tasa, dt);
    let cero = Memoria::c(f, 0.0);
    let negativa = m.sub(f, cero, tasa);
    let conserva = f.glsl(m.real, 27, &[negativa]);
    let uno = Memoria::c(f, 1.0);
    let mezcla = m.sub(f, uno, conserva);
    let escala = m.ajuste(f, 9);
    core::array::from_fn(|eje| {
        let objetivo = m.mul(f, v[eje], escala);
        let diferencia = m.sub(f, objetivo, previo[eje + 3]);
        let diferencia = m.mul(f, diferencia, mezcla);
        let nuevo = m.add(f, previo[eje + 3], diferencia);
        let nuevo = f.elegir(m.real, nace, objetivo, nuevo);
        if eje == 1 {
            let g = m.ajuste(f, 11);
            let g = m.mul(f, g, dt);
            m.sub(f, nuevo, g)
        } else {
            nuevo
        }
    })
}
pub(super) fn historial(f: &mut Funcion, m: &Memoria, id: Id, origen: [Id; 3], banderas: [Id; 3]) {
    let [nace, mueve, finita] = banderas;
    let cambia = f.binaria(op::LOGICAL_AND, m.booleano, mueve, finita);
    for i in (0..4).rev() {
        for eje in 0..3 {
            let offset = Memoria::u(f, 1536 + (i * 3 + eje) * 128);
            let indice = f.binaria(op::I_ADD, m.entero, offset, id);
            let antes = m.leer(f, indice, 20);
            let anterior = if i == 0 {
                origen[eje as usize]
            } else {
                let off = Memoria::u(f, 1536 + ((i - 1) * 3 + eje) * 128);
                let ind = f.binaria(op::I_ADD, m.entero, off, id);
                m.leer(f, ind, 20)
            };
            let valor = f.elegir(m.real, nace, origen[eje as usize], anterior);
            let valor = f.elegir(m.real, cambia, valor, antes);
            // GPU: cada plaza desplaza su historial en orden descendente, sin leer historial ajeno.
            m.guardar(f, indice, 20, valor);
        }
    }
}
