//! Caché opcional del plan fijo de deformación. Clave exacta, residente por ranura.
use crate::maquina::{Maquina, Memoria};
use hilo_spirv::{Funcion, Id, op};

/// Palabras necesarias para la extensión de caché; el prefijo ABI v1 conserva 32.
pub const PALABRAS: usize = 48;
/// Fase, amplitud y lado de la última deformación ejecutada.
pub const CLAVE: u32 = 32;
/// Una clave válida fue publicada en esta ranura.
pub const VALIDA: u32 = 35;
/// Pasadas de deformación solicitadas, contador modular.
pub const RECALCULADAS: u32 = 36;
/// Pasadas de deformación omitidas por clave idéntica, contador modular.
pub const OMITIDAS: u32 = 37;

pub(super) fn decidir(f: &mut Funcion, m: &MaquinaDatos, grupos: Id, permitido: Id) {
    let memoria = m.memoria;
    let cero = f.modulo().constante_u32(0);
    let uno = f.modulo().constante_u32(1);
    let largo = f.longitud(memoria.entero, memoria.control, 0);
    let minimo = f.modulo().constante_u32(48);
    let cabe_control = f.binaria(op::U_LESS_THAN_EQUAL, m.booleano, minimo, largo);
    let largo_parametros = f.longitud(memoria.entero, m.parametros, 0);
    let ocho = f.modulo().constante_u32(8);
    let cabe_parametros = f.binaria(op::U_LESS_THAN_EQUAL, m.booleano, ocho, largo_parametros);
    let cabe = f.binaria(op::LOGICAL_AND, m.booleano, cabe_control, cabe_parametros);
    // No acceder a la extensión si un ejecutor antiguo solo reservó el prefijo.
    let (trabajo, fin) = (f.etiqueta(), f.etiqueta());
    f.fusion(fin);
    f.saltar_si(cabe, trabajo, fin);
    f.bloque(trabajo);
    let valido = memoria.leer(f, memoria.control, VALIDA);
    let mut cambio = f.binaria(op::I_EQUAL, m.booleano, valido, cero);
    let clave = [1, 2, 0, 4, 5, 6, 7].map(|i| memoria.leer(f, m.parametros, i));
    for (indice, nuevo) in [CLAVE, CLAVE + 1, CLAVE + 2, 38, 39, 40, 41].into_iter().zip(clave) {
        let previo = memoria.leer(f, memoria.control, indice);
        let diferente = f.binaria(op::I_NOT_EQUAL, m.booleano, previo, nuevo);
        cambio = f.binaria(op::LOGICAL_OR, m.booleano, cambio, diferente);
    }
    let generar = memoria.leer(f, memoria.control, 12);
    let regenerar = f.binaria(op::I_NOT_EQUAL, m.booleano, generar, cero);
    cambio = f.binaria(op::LOGICAL_OR, m.booleano, cambio, regenerar);
    let ejecutar = f.binaria(op::LOGICAL_AND, m.booleano, permitido, cambio);
    let cantidad = f.elegir(memoria.entero, ejecutar, grupos, cero);
    memoria.guardar(f, 0, cantidad);
    for (indice, nuevo) in [CLAVE, CLAVE + 1, CLAVE + 2, 38, 39, 40, 41].into_iter().zip(clave) {
        let previo = memoria.leer(f, memoria.control, indice);
        let valor = f.elegir(memoria.entero, permitido, nuevo, previo);
        memoria.guardar(f, indice, valor);
    }
    let valido = f.elegir(memoria.entero, permitido, uno, valido);
    memoria.guardar(f, VALIDA, valido);
    let incremento = f.elegir(memoria.entero, ejecutar, uno, cero);
    memoria.incrementar(f, RECALCULADAS, incremento);
    let igual = f.binaria(op::I_EQUAL, m.booleano, cantidad, cero);
    let omitir = f.binaria(op::LOGICAL_AND, m.booleano, permitido, igual);
    let incremento = f.elegir(memoria.entero, omitir, uno, cero);
    memoria.incrementar(f, OMITIDAS, incremento);
    // GPU: propietario único del estado; barrera compute → indirecto publica el despacho.
    // Suspender no publica claves; al reanudar la decisión sigue siendo local a la ranura.
    f.saltar(fin);
    f.bloque(fin);
    // El argumento se escribió antes de entrar y solo cambia si cabe la extensión.
}

pub(super) struct MaquinaDatos {
    memoria: Memoria,
    booleano: Id,
    parametros: Id,
}

impl MaquinaDatos {
    pub(super) fn de(m: &Maquina) -> Self {
        Self { memoria: Memoria::de(m), booleano: m.booleano, parametros: m.parametros }
    }
}
