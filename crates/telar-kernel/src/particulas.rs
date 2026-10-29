//! Emisor de plazas reutilizables; dinámica y ciclo de vida residentes, sin acoplamiento al gas.
use crate::particulas_memoria::Estado;
use alloc::vec::Vec;
use hilo_spirv::{Modulo, op};
/// Plazas reservadas, con una invocación propietaria por partícula.
pub const CAPACIDAD: u32 = 128;
/// Posición(3), velocidad(3), edad, vida, generación, nacimiento, activa y control.
pub const CANALES: u32 = 12;
/// Plano de resumen; reloj, vivas, nacimientos, cuarentena, velocidad máxima y agotadas.
pub const CONTROL: u32 = 11;
/// Compila actualización y reducción/reloj; orden fijo [0,1] con barrera del backend.
#[must_use]
pub fn compilar() -> Vec<Vec<u32>> {
    (0..2).map(programa).collect()
}
fn programa(etapa: u32) -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let e = Estado::nuevo(&mut m);
    let v3 = m.tipo_vector(e.entero, 3);
    let ptr = m.tipo_puntero(op::Clase::Entrada, v3);
    let global = m.variable(ptr, op::Clase::Entrada);
    m.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let id = f.cargar(v3, global);
    let id = f.extraer(e.entero, id, 0);
    let cantidad = Estado::u(&mut f, if etapa == 0 { CAPACIDAD } else { 1 });
    let valido = f.binaria(op::U_LESS_THAN, e.booleano, id, cantidad);
    let longitud = f.longitud(e.entero, e.buffer, 0);
    let minimo = Estado::u(&mut f, CAPACIDAD * CANALES);
    let cabe = f.binaria(op::U_LESS_THAN_EQUAL, e.booleano, minimo, longitud);
    let valido = f.binaria(op::LOGICAL_AND, e.booleano, valido, cabe);
    let cuerpo = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(valido, cuerpo, fin);
    f.bloque(cuerpo);
    if etapa == 0 {
        crate::particulas_paso::emitir(&mut f, &e, id);
    } else {
        crate::particulas_resumen::emitir(&mut f, &e);
    }
    f.saltar(fin);
    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    m.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    m.ensamblar()
}
/// Referencia escalar solo de pruebas; el producto no la ejecuta por cuadro.
#[path = "particulas_referencia.rs"]
pub mod referencia;
