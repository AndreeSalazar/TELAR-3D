//! Shaders 2D de la interfaz nativa: posición NDC y color, sin cálculo de cámara.
use crate::{
    Id, Modulo,
    op::{self, Clase},
};
use alloc::vec::Vec;

fn variable(modulo: &mut Modulo, tipo: Id, clase: Clase, ubicacion: u32) -> Id {
    let puntero = modulo.tipo_puntero(clase, tipo);
    let variable = modulo.variable(puntero, clase);
    modulo.decorar(variable, op::DECORACION_UBICACION, &[ubicacion]);
    variable
}

/// Transfiere posición y color de los triángulos 2D.
#[must_use]
pub fn vertices() -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let real = modulo.tipo_f32();
    let vector = modulo.tipo_vector(real, 3);
    let vector4 = modulo.tipo_vector(real, 4);
    let posicion = variable(&mut modulo, vector, Clase::Entrada, 0);
    let color = variable(&mut modulo, vector, Clase::Entrada, 1);
    let salida = variable(&mut modulo, vector, Clase::Salida, 0);
    let puntero = modulo.tipo_puntero(Clase::Salida, vector4);
    let clip = modulo.variable(puntero, Clase::Salida);
    modulo.decorar(clip, op::DECORACION_INTEGRADA, &[op::INTEGRADA_POSICION]);
    let uno = modulo.constante_f32(1.0);
    let void = modulo.tipo_void();
    let tipo = modulo.tipo_funcion(void, &[]);
    let mut funcion = modulo.funcion(void, tipo);
    let bloque = funcion.etiqueta();
    funcion.bloque(bloque);
    let valor_posicion = funcion.cargar(vector, posicion);
    let componentes = [0, 1, 2].map(|i| funcion.extraer(real, valor_posicion, i));
    let valor = funcion.construir(vector4, &[componentes[0], componentes[1], componentes[2], uno]);
    funcion.guardar(clip, valor);
    let valor = funcion.cargar(vector, color);
    funcion.guardar(salida, valor);
    funcion.volver();
    let main = funcion.terminar();
    modulo.entrada(op::MODELO_VERTICES, main, "main", &[posicion, color, salida, clip]);
    modulo.ensamblar()
}

/// Escribe color opaco; la conversión sRGB se prepara al actualizar los buffers.
#[must_use]
pub fn fragmentos() -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let real = modulo.tipo_f32();
    let vector = modulo.tipo_vector(real, 3);
    let vector4 = modulo.tipo_vector(real, 4);
    let entrada = variable(&mut modulo, vector, Clase::Entrada, 0);
    let salida = variable(&mut modulo, vector4, Clase::Salida, 0);
    let uno = modulo.constante_f32(1.0);
    let void = modulo.tipo_void();
    let tipo = modulo.tipo_funcion(void, &[]);
    let mut funcion = modulo.funcion(void, tipo);
    let bloque = funcion.etiqueta();
    funcion.bloque(bloque);
    let color = funcion.cargar(vector, entrada);
    let componentes = [0, 1, 2].map(|i| funcion.extraer(real, color, i));
    let color = funcion.construir(vector4, &[componentes[0], componentes[1], componentes[2], uno]);
    funcion.guardar(salida, color);
    funcion.volver();
    let main = funcion.terminar();
    modulo.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    modulo.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    modulo.ensamblar()
}
