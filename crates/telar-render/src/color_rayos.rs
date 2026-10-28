//! Conversión de revisión en GPU: exposición, Reinhard y sRGB, empaquetado RGB8.
use crate::trazado::Matematica;
use hilo_spirv::{Funcion, Id, op};

pub(crate) fn convertir(f: &mut Funcion, mat: &Matematica, lineal: Id, exposicion: f32) -> Id {
    let mut empaquetado = f.modulo().constante_u32(0);
    let uno = Matematica::escalar(f, 1.0);
    let exposicion = Matematica::escalar(f, exposicion);
    for canal in 0..3 {
        let valor = f.extraer(mat.real, lineal, canal);
        let valor = f.binaria(op::F_MUL, mat.real, valor, exposicion);
        let divisor = f.binaria(op::F_ADD, mat.real, valor, uno);
        let valor = f.binaria(op::F_DIV, mat.real, valor, divisor);
        let exponente = Matematica::escalar(f, 1.0 / 2.4);
        let potencia = f.glsl(mat.real, 26, &[valor, exponente]);
        let escala = Matematica::escalar(f, 1.055);
        let resta = Matematica::escalar(f, 0.055);
        let alta = f.binaria(op::F_MUL, mat.real, potencia, escala);
        let alta = f.binaria(op::F_SUB, mat.real, alta, resta);
        let escala = Matematica::escalar(f, 12.92);
        let baja = f.binaria(op::F_MUL, mat.real, valor, escala);
        let umbral = Matematica::escalar(f, 0.003_130_8);
        let pequena = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, umbral, valor);
        let srgb = f.elegir(mat.real, pequena, baja, alta);
        let escala = Matematica::escalar(f, 255.0);
        let medio = Matematica::escalar(f, 0.5);
        let srgb = f.binaria(op::F_MUL, mat.real, srgb, escala);
        let srgb = f.binaria(op::F_ADD, mat.real, srgb, medio);
        let byte = f.unaria(op::F_A_U, mat.entero, srgb);
        let giro = f.modulo().constante_u32(canal * 8);
        let desplazado = f.binaria(op::SHIFT_LEFT_LOGICAL, mat.entero, byte, giro);
        empaquetado = f.binaria(op::I_ADD, mat.entero, empaquetado, desplazado);
    }
    empaquetado
}
