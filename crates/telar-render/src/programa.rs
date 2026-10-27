//! Posición mundial y normal interpoladas, sin generar geometría desde CPU.
use crate::tipos::{Tipos, variable};
use alloc::vec::Vec;
use hilo_spirv::{
    Modulo,
    op::{self, Clase},
};

/// Shader de vértices para posición+normal, seis f32 por vértice.
#[must_use]
pub fn vertices() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let posicion = variable(&mut m, t.v3, Clase::Entrada, 0);
    let normal = variable(&mut m, t.v3, Clase::Entrada, 1);
    let normal_salida = variable(&mut m, t.v3, Clase::Salida, 0);
    let mundo_salida = variable(&mut m, t.v3, Clase::Salida, 1);
    let puntero = m.tipo_puntero(Clase::Salida, t.v4);
    let clip = m.variable(puntero, Clase::Salida);
    m.decorar(clip, op::DECORACION_INTEGRADA, &[op::INTEGRADA_POSICION]);
    let vacio = m.tipo_void();
    let firma = m.tipo_funcion(vacio, &[]);
    let mut f = m.funcion(vacio, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let punto = f.cargar(t.v3, posicion);
    let mut salida = t.cargar(&mut f, 3);
    for indice in 0..3 {
        let valor = f.extraer(t.real, punto, indice);
        let columna = t.cargar(&mut f, indice);
        let termino = f.binaria(op::VECTOR_TIMES_SCALAR, t.v4, columna, valor);
        salida = f.binaria(op::F_ADD, t.v4, salida, termino);
    }
    let n = f.cargar(t.v3, normal);
    f.guardar(clip, salida);
    f.guardar(normal_salida, n);
    f.guardar(mundo_salida, punto);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_VERTICES, main, "main", &[posicion, normal, normal_salida, mundo_salida, clip]);
    m.ensamblar()
}
