//! Envoltura del ABI de rejilla: acceso exclusivo a altura/normal del punto actual.
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Interfaz {
    pub(super) modulo: Modulo,
    pub(super) entero: Id,
    pub(super) real: Id,
    pub(super) vector: Id,
    pub(super) booleano: Id,
    pub(super) void: Id,
    pub(super) tipo_main: Id,
    pub(super) entrada: Id,
    pub(super) global: Id,
    pub(super) buffers: [Id; 4],
    pub(super) punteros: [Id; 4],
}

impl Interfaz {
    pub(super) fn nueva() -> Self {
        let mut modulo = Modulo::nuevo();
        let entero = modulo.tipo_u32();
        let real = modulo.tipo_f32();
        let vector = modulo.tipo_vector(real, 3);
        let booleano = modulo.tipo_bool();
        let void = modulo.tipo_void();
        let tipo_main = modulo.tipo_funcion(void, &[]);
        let entrada = modulo.tipo_puntero(Clase::Entrada, entero);
        let vector_entero = modulo.tipo_vector(entero, 3);
        let puntero = modulo.tipo_puntero(Clase::Entrada, vector_entero);
        let global = modulo.variable(puntero, Clase::Entrada);
        modulo.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
        let tipos = [entero, real, entero, entero];
        let punteros = tipos.map(|tipo| modulo.tipo_puntero(Clase::Almacen, tipo));
        let buffers = core::array::from_fn(|indice| {
            let array = modulo.tipo_array_dinamico(tipos[indice], 4);
            let bloque = modulo.tipo_bloque(&[array], &[0]);
            let puntero = modulo.tipo_puntero(Clase::Almacen, bloque);
            let variable = modulo.variable(puntero, Clase::Almacen);
            modulo.decorar(variable, op::DECORACION_CONJUNTO, &[0]);
            modulo.decorar(variable, op::DECORACION_ENLACE, &[u32::try_from(indice).unwrap_or(0)]);
            variable
        });
        Self { modulo, entero, real, vector, booleano, void, tipo_main, entrada, global, buffers, punteros }
    }

    pub(super) fn leer(&self, funcion: &mut Funcion, enlace: usize, indice: Id) -> Id {
        let cero = funcion.modulo().constante_u32(0);
        let direccion = funcion.acceso(self.punteros[enlace], self.buffers[enlace], &[cero, indice]);
        funcion.cargar(if enlace == 1 { self.real } else { self.entero }, direccion)
    }
}
