//! Decisiones residentes: servicios, límites, caché, dispatch y dibujo indirectos.
use crate::{
    abi,
    maquina::{Maquina, Memoria},
    servicios::atender,
};
use alloc::vec::Vec;
use hilo_spirv::{Funcion, Id, op};

/// Emite el coordinador de una invocación para la ruta procedural de rejilla.
/// Lee parámetros en enlace 0 y control de 32 u32 en enlace 3. No escribe malla.
#[must_use]
pub fn compilar() -> Vec<u32> {
    emitir(false)
}

/// Coordinador con guardián de arranque. Requiere cabecera en enlace 4 y servicios
/// indirectos en control[27..30], configurados con `PlanProcedural::con_arranque`.
#[must_use]
pub fn compilar_con_arranque() -> Vec<u32> {
    emitir(true)
}

fn emitir(arranque: bool) -> Vec<u32> {
    let mut maquina = Maquina::nueva();
    let guardia = arranque.then(|| crate::verificacion::Guardia::nueva(&mut maquina));
    let memoria = Memoria::de(&maquina);
    let [cero, uno, seis, sesenta_y_tres, sesenta_y_cuatro] =
        [0, 1, 6, 63, 64].map(|n| maquina.modulo.constante_u32(n));
    let version = maquina.modulo.constante_u32(abi::VERSION);
    let mut funcion = maquina.modulo.funcion(maquina.vacio, maquina.firma);
    let entrada = funcion.etiqueta();
    funcion.bloque(entrada);
    if let Some(guardia) = guardia {
        guardia.prologo(&mut funcion);
    }
    atender(&mut funcion, memoria, maquina.booleano);
    memoria.guardar(&mut funcion, abi::VERSION_CONTROL, version);
    memoria.incrementar(&mut funcion, abi::VUELTA, uno);
    let lado = memoria.leer(&mut funcion, maquina.parametros, 0);
    let capacidad = memoria.leer(&mut funcion, maquina.parametros, 3);
    let suspendido = memoria.leer(&mut funcion, memoria.control, abi::SUSPENDIDO);
    let suspendido = funcion.binaria(op::I_NOT_EQUAL, maquina.booleano, suspendido, cero);
    let excede = funcion.binaria(op::U_LESS_THAN, maquina.booleano, capacidad, lado);
    let suficiente = funcion.binaria(op::U_LESS_THAN, maquina.booleano, uno, lado);
    let falso = funcion.binaria(op::U_LESS_THAN, maquina.booleano, cero, cero);
    let permitido = funcion.elegir(maquina.booleano, excede, falso, suficiente);
    let permitido = funcion.elegir(maquina.booleano, suspendido, falso, permitido);
    let lado_seguro = funcion.elegir(maquina.entero, permitido, lado, uno);
    let cuadrado = funcion.binaria(op::I_MUL, maquina.entero, lado_seguro, lado_seguro);
    let puntos = funcion.elegir(maquina.entero, permitido, cuadrado, cero);
    let borde = funcion.binaria(op::I_SUB, maquina.entero, lado_seguro, uno);
    let celdas = funcion.binaria(op::I_MUL, maquina.entero, borde, borde);
    let indices = funcion.binaria(op::I_MUL, maquina.entero, celdas, seis);
    let redondeo = funcion.binaria(op::I_ADD, maquina.entero, puntos, sesenta_y_tres);
    let grupos = funcion.binaria(op::U_DIV, maquina.entero, redondeo, sesenta_y_cuatro);
    cache(&mut funcion, memoria, maquina.booleano, lado, permitido, grupos);
    let error = funcion.elegir(maquina.entero, excede, uno, cero);
    for (indice, valor) in [
        (0, grupos),
        (1, uno),
        (2, uno),
        (3, indices),
        (4, uno),
        (5, cero),
        (6, cero),
        (7, cero),
        (8, lado_seguro),
        (9, puntos),
        (10, error),
    ] {
        memoria.guardar(&mut funcion, indice, valor);
    }
    funcion.volver();
    let main = funcion.terminar();
    maquina.modulo.entrada_calculo(main, "main", &[], [1, 1, 1]);
    maquina.modulo.ensamblar()
}

fn cache(funcion: &mut Funcion, memoria: Memoria, booleano: Id, lado: Id, permitido: Id, grupos: Id) {
    let cero = funcion.modulo().constante_u32(0);
    let uno = funcion.modulo().constante_u32(1);
    let previo = memoria.leer(funcion, memoria.control, 11);
    let cambio = funcion.binaria(op::I_NOT_EQUAL, booleano, lado, previo);
    let generar = funcion.binaria(op::LOGICAL_AND, booleano, permitido, cambio);
    let cantidad = funcion.elegir(memoria.entero, generar, grupos, cero);
    let incremento = funcion.elegir(memoria.entero, generar, uno, cero);
    memoria.incrementar(funcion, 15, incremento);
    let ultimo = funcion.elegir(memoria.entero, permitido, lado, previo);
    for (indice, valor) in [(11, ultimo), (12, cantidad), (13, uno), (14, uno)] {
        memoria.guardar(funcion, indice, valor);
    }
}
