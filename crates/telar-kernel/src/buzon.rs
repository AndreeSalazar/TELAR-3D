//! Interfaz para emitir solicitudes desde otros kernels, sin volver al host.
use crate::{abi, maquina::Memoria};
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

/// Emisor de instrucciones GPU para el buzón ABI. No representa memoria host.
/// Solo una invocación puede escribir en cada pasada; los productores se ordenan
/// con barreras. No permite solicitudes concurrentes desde miles de hilos.
pub struct Buzon {
    memoria: Memoria,
    booleano: Id,
}

impl Buzon {
    /// Asocia el emisor al buffer `control` ya declarado en el mismo módulo.
    /// No reserva memoria; ese buffer debe tener el layout de `abi::PALABRAS`.
    pub fn nuevo(modulo: &mut Modulo, control: Id) -> Self {
        let entero = modulo.tipo_u32();
        let booleano = modulo.tipo_bool();
        let puntero = modulo.tipo_puntero(Clase::Almacen, entero);
        Self { memoria: Memoria { entero, puntero, control }, booleano }
    }

    /// Emite una solicitud y devuelve un Id booleano GPU: aceptada o rechazada.
    /// `codigo` es un Id u32 del mismo módulo; cero significa ninguna solicitud.
    /// Un buzón ocupado conserva su contenido y cuenta el rechazo.
    pub fn solicitar(&self, funcion: &mut Funcion, codigo: Id) -> Id {
        let memoria = self.memoria;
        let cero = funcion.modulo().constante_u32(0);
        let uno = funcion.modulo().constante_u32(1);
        let pendiente = memoria.leer(funcion, memoria.control, abi::SOLICITUD);
        let ocupado = funcion.binaria(op::I_NOT_EQUAL, self.booleano, pendiente, cero);
        let hay_codigo = funcion.binaria(op::I_NOT_EQUAL, self.booleano, codigo, cero);
        let falso = funcion.binaria(op::U_LESS_THAN, self.booleano, cero, cero);
        let aceptada = funcion.elegir(self.booleano, ocupado, falso, hay_codigo);
        let solicitud = funcion.elegir(memoria.entero, aceptada, codigo, pendiente);
        memoria.guardar(funcion, abi::SOLICITUD, solicitud);
        let rechazo = funcion.binaria(op::LOGICAL_AND, self.booleano, ocupado, hay_codigo);
        let rechazo = funcion.elegir(memoria.entero, rechazo, uno, cero);
        memoria.incrementar(funcion, abi::OCUPADAS, rechazo);
        aceptada
    }
}
