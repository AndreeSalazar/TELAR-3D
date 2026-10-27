//! Operaciones adicionales por punto. Conservar x/z permite reutilizar la
//! topología; el orden del plan define sobre qué altura se aplica cada operación.

use super::{Entorno, escribir, leer};
use crate::op;
use alloc::vec::Vec;

/// Escala la altura y transforma la normal analítica de la superficie.
/// Para `y' = k*y`, la normal es `normalize(k*nx, ny, k*nz)`.
/// Conserva x/z e índices, incluidos factores cero y negativos.
///
/// # Errors
/// Rechaza factores no finitos o con magnitud superior a 16.
pub fn escalar_altura(factor: f32) -> Result<Vec<u32>, &'static str> {
    if !factor.is_finite() || factor.abs() > 16.0 {
        return Err("factor de altura fuera del intervalo finito [-16, 16]");
    }
    let mut entorno = Entorno::nuevo();
    let [cero, uno, tres, seis, nueve] = [0, 1, 3, 6, 9].map(|n| entorno.modulo.constante_u32(n));
    let factor = entorno.modulo.constante_f32(factor);
    let vector = entorno.modulo.tipo_vector(entorno.real, 3);
    let mut funcion = entorno.modulo.funcion(entorno.void, entorno.funcion);
    let (entrada, cuerpo, fin) = (funcion.etiqueta(), funcion.etiqueta(), funcion.etiqueta());
    funcion.bloque(entrada);
    let direccion = funcion.acceso(entorno.entrada, entorno.global, &[cero]);
    let indice = funcion.cargar(entorno.entero, direccion);
    let puntos = leer(&mut funcion, entorno.puntero[3], entorno.buffers[3], entorno.entero, nueve);
    let dentro = funcion.binaria(op::U_LESS_THAN, entorno.booleano, indice, puntos);
    funcion.fusion(fin);
    funcion.saltar_si(dentro, cuerpo, fin);
    funcion.bloque(cuerpo);
    let base = funcion.binaria(op::I_MUL, entorno.entero, indice, seis);
    let altura = funcion.binaria(op::I_ADD, entorno.entero, base, uno);
    let valor = leer(&mut funcion, entorno.puntero[1], entorno.buffers[1], entorno.real, altura);
    let valor = funcion.binaria(op::F_MUL, entorno.real, valor, factor);
    escribir(&mut funcion, entorno.puntero[1], entorno.buffers[1], altura, valor);
    let inicio_normal = funcion.binaria(op::I_ADD, entorno.entero, base, tres);
    let direcciones = [0, 1, 2].map(|componente| {
        let constante = funcion.modulo().constante_u32(componente);
        funcion.binaria(op::I_ADD, entorno.entero, inicio_normal, constante)
    });
    let componentes = direcciones
        .map(|direccion| leer(&mut funcion, entorno.puntero[1], entorno.buffers[1], entorno.real, direccion));
    let normal_x = funcion.binaria(op::F_MUL, entorno.real, componentes[0], factor);
    let normal_z = funcion.binaria(op::F_MUL, entorno.real, componentes[2], factor);
    let normal = funcion.construir(vector, &[normal_x, componentes[1], normal_z]);
    let normal = funcion.glsl(vector, op::GLSL_NORMALIZAR, &[normal]);
    for (direccion, componente) in direcciones.into_iter().zip(0..3) {
        let valor = funcion.extraer(entorno.real, normal, componente);
        // GPU: cada hilo posee un vértice; escribir publica sus componentes.
        // La barrera entre operaciones la emite el ejecutor del plan.
        escribir(&mut funcion, entorno.puntero[1], entorno.buffers[1], direccion, valor);
    }
    funcion.saltar(fin);
    funcion.bloque(fin);
    funcion.volver();
    let main = funcion.terminar();
    entorno.modulo.entrada_calculo(main, "main", &[entorno.global], [64, 1, 1]);
    Ok(entorno.modulo.ensamblar())
}
