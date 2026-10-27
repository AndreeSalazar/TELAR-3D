//! Buzón GPU de una entrada; productores y consumidor se ejecutan por pasadas.
use crate::{
    Buzon,
    abi::{self, Servicio},
    maquina::{Maquina, Memoria},
};
use alloc::vec::Vec;
use hilo_spirv::{Funcion, Id, op};

/// Emite un productor GPU que solicita un servicio en una vuelta concreta.
/// `codigo` es el número ABI; también admite códigos desconocidos para verificar
/// productores externos. Un buzón ocupado conserva su solicitud y cuenta rechazo.
/// Debe ejecutarse como una única invocación en dominio Servicio, nunca por punto.
#[must_use]
pub fn solicitar_en(codigo: u32, vuelta: u32) -> Vec<u32> {
    let mut maquina = Maquina::nueva();
    let memoria = Memoria::de(&maquina);
    let buzon = Buzon::nuevo(&mut maquina.modulo, memoria.control);
    let numero = maquina.modulo.constante_u32(codigo);
    let vuelta_pedida = maquina.modulo.constante_u32(vuelta);
    let mut funcion = maquina.modulo.funcion(maquina.vacio, maquina.firma);
    let (entrada, cuerpo, fin) = (funcion.etiqueta(), funcion.etiqueta(), funcion.etiqueta());
    funcion.bloque(entrada);
    let actual = memoria.leer(&mut funcion, memoria.control, abi::VUELTA);
    let diferente = funcion.binaria(op::I_NOT_EQUAL, maquina.booleano, actual, vuelta_pedida);
    funcion.fusion(fin);
    funcion.saltar_si(diferente, fin, cuerpo);
    funcion.bloque(cuerpo);
    buzon.solicitar(&mut funcion, numero);
    funcion.saltar(fin);
    funcion.bloque(fin);
    funcion.volver();
    let main = funcion.terminar();
    maquina.modulo.entrada_calculo(main, "main", &[], [1, 1, 1]);
    maquina.modulo.ensamblar()
}

pub(super) fn atender(funcion: &mut Funcion, memoria: Memoria, booleano: Id) {
    let cero = funcion.modulo().constante_u32(0);
    let uno = funcion.modulo().constante_u32(1);
    let peticion = memoria.leer(funcion, memoria.control, abi::SOLICITUD);
    let hay = funcion.binaria(op::I_NOT_EQUAL, booleano, peticion, cero);
    let (cuerpo, fin) = (funcion.etiqueta(), funcion.etiqueta());
    funcion.fusion(fin);
    funcion.saltar_si(hay, cuerpo, fin);
    funcion.bloque(cuerpo);
    let condiciones = [Servicio::RehacerTopologia, Servicio::Suspender, Servicio::Reanudar].map(|servicio| {
        let codigo = funcion.modulo().constante_u32(servicio as u32);
        let distinto = funcion.binaria(op::I_NOT_EQUAL, booleano, peticion, codigo);
        (distinto, funcion.elegir(memoria.entero, distinto, cero, uno))
    });
    let estado = memoria.leer(funcion, memoria.control, abi::SUSPENDIDO);
    let estado = funcion.elegir(memoria.entero, condiciones[1].0, estado, uno);
    let estado = funcion.elegir(memoria.entero, condiciones[2].0, estado, cero);
    memoria.guardar(funcion, abi::SUSPENDIDO, estado);
    let previo = memoria.leer(funcion, memoria.control, 11);
    let previo = funcion.elegir(memoria.entero, condiciones[0].0, previo, cero);
    memoria.guardar(funcion, 11, previo);
    let conocido = condiciones
        .into_iter()
        .fold(cero, |total, (_, valor)| funcion.binaria(op::I_ADD, memoria.entero, total, valor));
    memoria.incrementar(funcion, abi::ATENDIDAS, conocido);
    let error = funcion.binaria(op::I_SUB, memoria.entero, uno, conocido);
    memoria.guardar(funcion, abi::RESULTADO, error);
    memoria.incrementar(funcion, abi::DESCONOCIDAS, error);
    memoria.guardar(funcion, abi::ULTIMA, peticion);
    memoria.guardar(funcion, abi::SOLICITUD, cero);
    funcion.saltar(fin);
    funcion.bloque(fin);
}
