//! Guardián GPU del arranque: longitudes reales antes de leer la cabecera.
use crate::{
    abi, arranque,
    maquina::{Maquina, Memoria},
};
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Guardia {
    memoria: Memoria,
    booleano: Id,
    parametros: Id,
    buffers: [Id; 3],
}

fn buffer(modulo: &mut Modulo, entero: Id, enlace: u32) -> Id {
    let array = modulo.tipo_array_dinamico(entero, 4);
    let bloque = modulo.tipo_bloque(&[array], &[0]);
    let puntero = modulo.tipo_puntero(Clase::Almacen, bloque);
    let variable = modulo.variable(puntero, Clase::Almacen);
    modulo.decorar(variable, op::DECORACION_CONJUNTO, &[0]);
    modulo.decorar(variable, op::DECORACION_ENLACE, &[enlace]);
    modulo.decorar(variable, 24, &[]); // NonWritable: solo consulta longitudes y configuración.
    variable
}

impl Guardia {
    pub(super) fn nueva(maquina: &mut Maquina) -> Self {
        Self {
            memoria: Memoria::de(maquina),
            booleano: maquina.booleano,
            parametros: maquina.parametros,
            buffers: [1, 2, 4].map(|enlace| buffer(&mut maquina.modulo, maquina.entero, enlace)),
        }
    }

    fn es(&self, funcion: &mut Funcion, valor: Id, esperado: u32) -> Id {
        let esperado = funcion.modulo().constante_u32(esperado);
        funcion.binaria(op::I_EQUAL, self.booleano, valor, esperado)
    }

    fn minimo(&self, funcion: &mut Funcion, valor: Id, minimo: u32) -> Id {
        let minimo = funcion.modulo().constante_u32(minimo);
        funcion.binaria(op::U_LESS_THAN_EQUAL, self.booleano, minimo, valor)
    }

    fn hasta(&self, funcion: &mut Funcion, valor: Id, maximo: Id) -> Id {
        funcion.binaria(op::U_LESS_THAN_EQUAL, self.booleano, valor, maximo)
    }

    fn exigir(&self, funcion: &mut Funcion, condiciones: &[Id], error: u32) {
        let valido = condiciones.iter().copied().reduce(|a, b| funcion.binaria(op::LOGICAL_AND, self.booleano, a, b));
        let Some(valido) = valido else {
            return;
        };
        let (rechazo, siguiente) = (funcion.etiqueta(), funcion.etiqueta());
        funcion.fusion(siguiente);
        funcion.saltar_si(valido, siguiente, rechazo);
        funcion.bloque(rechazo);
        let cero = funcion.modulo().constante_u32(0);
        let uno = funcion.modulo().constante_u32(1);
        let codigo = funcion.modulo().constante_u32(error);
        // GPU: esta invocación anula todos los argumentos; el ejecutor publica
        // esas escrituras antes de los lectores indirectos, servicios incluidos.
        for indice in [0, 3, 5, 6, 7, 9, 10, 12, 26, 27] {
            self.memoria.guardar(funcion, indice, cero);
        }
        for indice in [1, 2, 4, 8, 13, 14, 28, 29] {
            self.memoria.guardar(funcion, indice, uno);
        }
        self.memoria.guardar(funcion, abi::ERROR_ARRANQUE, codigo);
        funcion.volver();
        funcion.bloque(siguiente);
    }

    pub(super) fn prologo(&self, funcion: &mut Funcion) {
        let memoria = self.memoria;
        let tamano = funcion.longitud(memoria.entero, self.buffers[2], 0);
        let parametros = funcion.longitud(memoria.entero, self.parametros, 0);
        let largo = self.minimo(funcion, tamano, 32);
        let parametros_validos = self.minimo(funcion, parametros, 4);
        // El backend garantiza control de 32 palabras antes del envío. Un header
        // truncado sí es una entrada admitida para verificar rechazo dentro de GPU.
        self.exigir(funcion, &[largo, parametros_validos], 1);
        let cabecera: [Id; 32] =
            core::array::from_fn(|indice| memoria.leer(funcion, self.buffers[2], u32::try_from(indice).unwrap_or(0)));
        let firmas = [
            self.es(funcion, cabecera[0], arranque::FIRMA),
            self.es(funcion, cabecera[1], arranque::VERSION),
            self.es(funcion, cabecera[2], 32),
        ];
        self.exigir(funcion, &firmas, 2);
        let capacidad = memoria.leer(funcion, self.parametros, 3);
        let maximo = funcion.modulo().constante_u32(1024);
        let coincide = funcion.binaria(op::I_EQUAL, self.booleano, capacidad, cabecera[16]);
        let limites = [coincide, self.minimo(funcion, capacidad, 2), self.hasta(funcion, capacidad, maximo)];
        self.exigir(funcion, &limites, 3);
        self.regiones(funcion, &cabecera, capacidad);
        self.capacidades(funcion, &cabecera, capacidad);
        let cero = funcion.modulo().constante_u32(0);
        let uno = funcion.modulo().constante_u32(1);
        memoria.guardar(funcion, abi::ERROR_ARRANQUE, cero);
        memoria.guardar(funcion, abi::LISTO, uno);
        for indice in 27..30 {
            memoria.guardar(funcion, indice, uno);
        }
        memoria.guardar(funcion, abi::GENERACION_PLAN, cabecera[24]);
        memoria.guardar(funcion, abi::FABRICANTE, cabecera[5]);
    }

    fn regiones(&self, funcion: &mut Funcion, h: &[Id; 32], capacidad: Id) {
        let entero = self.memoria.entero;
        let uno = funcion.modulo().constante_u32(1);
        let cuatro = funcion.modulo().constante_u32(4);
        let seis = funcion.modulo().constante_u32(6);
        let puntos = funcion.binaria(op::I_MUL, entero, capacidad, capacidad);
        let vertices = funcion.binaria(op::I_MUL, entero, puntos, seis);
        let borde = funcion.binaria(op::I_SUB, entero, capacidad, uno);
        let celdas = funcion.binaria(op::I_MUL, entero, borde, borde);
        let indices = funcion.binaria(op::I_MUL, entero, celdas, seis);
        let rango = funcion.binaria(op::U_DIV, entero, h[13], cuatro);
        let reales = [
            funcion.longitud(entero, self.buffers[0], 0),
            funcion.longitud(entero, self.buffers[1], 0),
            funcion.longitud(entero, self.memoria.control, 0),
        ];
        let coincide_v = funcion.binaria(op::I_EQUAL, self.booleano, h[18], vertices);
        let coincide_i = funcion.binaria(op::I_EQUAL, self.booleano, h[19], indices);
        let condiciones = [
            coincide_v,
            coincide_i,
            self.es(funcion, h[17], 4),
            self.es(funcion, h[20], 32),
            self.hasta(funcion, vertices, reales[0]),
            self.hasta(funcion, indices, reales[1]),
            self.minimo(funcion, reales[2], 32),
            self.hasta(funcion, vertices, rango),
            self.hasta(funcion, indices, rango),
            self.minimo(funcion, rango, 32),
        ];
        self.exigir(funcion, &condiciones, 3);
    }

    fn capacidades(&self, funcion: &mut Funcion, h: &[Id; 32], capacidad: Id) {
        let entero = self.memoria.entero;
        let presentes = funcion.binaria(op::BITWISE_AND, entero, h[3], h[4]);
        let cumple = funcion.binaria(op::I_EQUAL, self.booleano, presentes, h[4]);
        let condiciones = [
            cumple,
            self.es(funcion, h[4], arranque::CAPACIDADES),
            self.minimo(funcion, h[8], 0x0040_1000),
            self.minimo(funcion, h[10], 64),
            self.minimo(funcion, h[11], 64),
            self.minimo(funcion, h[25], 5),
            self.es(funcion, h[21], 64),
        ];
        self.exigir(funcion, &condiciones, 4);
        let puntos = funcion.binaria(op::I_MUL, entero, capacidad, capacidad);
        let redondeo = funcion.modulo().constante_u32(63);
        let grupo = funcion.modulo().constante_u32(64);
        let redondeo = funcion.binaria(op::I_ADD, entero, puntos, redondeo);
        let grupos = funcion.binaria(op::U_DIV, entero, redondeo, grupo);
        let max_etapas = funcion.modulo().constante_u32(32);
        let condiciones = [
            self.hasta(funcion, grupos, h[9]),
            self.hasta(funcion, grupos, h[23]),
            self.minimo(funcion, h[22], 2),
            self.hasta(funcion, h[22], max_etapas),
            self.minimo(funcion, h[24], 1),
        ];
        self.exigir(funcion, &condiciones, 5);
        let reservados: alloc::vec::Vec<_> = h[27..].iter().map(|valor| self.es(funcion, *valor, 0)).collect();
        self.exigir(funcion, &reservados, 2);
    }
}
