//! Construcción SPIR-V compartida; nada de objetos Vulkan ni llamadas al host.
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Maquina {
    pub modulo: Modulo,
    pub entero: Id,
    pub booleano: Id,
    pub vacio: Id,
    pub firma: Id,
    pub puntero: Id,
    pub parametros: Id,
    pub control: Id,
}

impl Maquina {
    pub(super) fn nueva() -> Self {
        let mut modulo = Modulo::nuevo();
        let entero = modulo.tipo_u32();
        let booleano = modulo.tipo_bool();
        let vacio = modulo.tipo_void();
        let firma = modulo.tipo_funcion(vacio, &[]);
        let puntero = modulo.tipo_puntero(Clase::Almacen, entero);
        let buffers = [0, 3].map(|enlace| {
            let array = modulo.tipo_array_dinamico(entero, 4);
            let bloque = modulo.tipo_bloque(&[array], &[0]);
            let tipo = modulo.tipo_puntero(Clase::Almacen, bloque);
            let buffer = modulo.variable(tipo, Clase::Almacen);
            modulo.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
            modulo.decorar(buffer, op::DECORACION_ENLACE, &[enlace]);
            buffer
        });
        Self { modulo, entero, booleano, vacio, firma, puntero, parametros: buffers[0], control: buffers[1] }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Memoria {
    pub entero: Id,
    pub puntero: Id,
    pub control: Id,
}

impl Memoria {
    pub(super) fn de(maquina: &Maquina) -> Self {
        Self { entero: maquina.entero, puntero: maquina.puntero, control: maquina.control }
    }

    pub(super) fn leer(self, funcion: &mut Funcion, buffer: Id, indice: u32) -> Id {
        let cero = funcion.modulo().constante_u32(0);
        let indice = funcion.modulo().constante_u32(indice);
        let direccion = funcion.acceso(self.puntero, buffer, &[cero, indice]);
        funcion.cargar(self.entero, direccion)
    }

    pub(super) fn guardar(self, funcion: &mut Funcion, indice: u32, valor: Id) {
        let cero = funcion.modulo().constante_u32(0);
        let indice = funcion.modulo().constante_u32(indice);
        let direccion = funcion.acceso(self.puntero, self.control, &[cero, indice]);
        // GPU: una invocación es dueña del buzón en esta pasada; las pasadas se
        // publican mediante barreras compute → compute/indirecto del ejecutor.
        funcion.guardar(direccion, valor);
    }

    pub(super) fn incrementar(self, funcion: &mut Funcion, indice: u32, incremento: Id) {
        let previo = self.leer(funcion, self.control, indice);
        let siguiente = funcion.binaria(op::I_ADD, self.entero, previo, incremento);
        self.guardar(funcion, indice, siguiente);
    }
}
