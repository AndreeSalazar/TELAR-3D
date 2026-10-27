//! Autosombra aproximada: marcha acotada sobre los triángulos de una rejilla regular.
//! No duplica la fórmula de deformación: consulta las alturas finales residentes.
use crate::tipos::Tipos;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Rejilla {
    vertices: Id,
    control: Id,
    puntero_real: Id,
    puntero_entero: Id,
    entero: Id,
    booleano: Id,
}

impl Rejilla {
    pub(super) fn nueva(m: &mut Modulo, t: &Tipos) -> Self {
        let entero = m.tipo_u32();
        let booleano = m.tipo_bool();
        let [vertices, control] = [(1, t.real), (3, entero)].map(|(enlace, tipo)| {
            let array = m.tipo_array_dinamico(tipo, 4);
            let bloque = m.tipo_bloque(&[array], &[0]);
            let puntero = m.tipo_puntero(Clase::Almacen, bloque);
            let buffer = m.variable(puntero, Clase::Almacen);
            m.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
            m.decorar(buffer, op::DECORACION_ENLACE, &[enlace]);
            m.decorar(buffer, 24, &[]); // NonWritable.
            buffer
        });
        let puntero_real = m.tipo_puntero(Clase::Almacen, t.real);
        let puntero_entero = m.tipo_puntero(Clase::Almacen, entero);
        Self { vertices, control, puntero_real, puntero_entero, entero, booleano }
    }

    fn altura(&self, f: &mut Funcion, t: &Tipos, indice: Id) -> Id {
        let cero = f.modulo().constante_u32(0);
        let uno = f.modulo().constante_u32(1);
        let seis = f.modulo().constante_u32(6);
        let base = f.binaria(op::I_MUL, self.entero, indice, seis);
        let y = f.binaria(op::I_ADD, self.entero, base, uno);
        let longitud = f.longitud(self.entero, self.vertices, 0);
        let ultimo = f.binaria(op::I_SUB, self.entero, longitud, uno);
        let y = f.glsl(self.entero, 38, &[y, ultimo]); // UMin: defensa adicional.
        let direccion = f.acceso(self.puntero_real, self.vertices, &[cero, y]);
        f.cargar(t.real, direccion)
    }

    fn terreno(&self, f: &mut Funcion, t: &Tipos, punto: Id, lado: Id) -> Id {
        let uno = f.modulo().constante_u32(1);
        let borde = f.binaria(op::I_SUB, self.entero, lado, uno);
        let borde_real = f.unaria(op::U_A_F, t.real, borde);
        let cero = Tipos::constante(f, 0.0);
        let dos = Tipos::constante(f, 2.0);
        let cuarto = Tipos::constante(f, 0.25);
        let epsilon = Tipos::constante(f, 0.001);
        let maximo = f.binaria(op::F_SUB, t.real, borde_real, epsilon);
        let coordenadas = [0, 2].map(|indice| {
            let eje = f.extraer(t.real, punto, indice);
            let eje = f.binaria(op::F_ADD, t.real, eje, dos);
            let eje = f.binaria(op::F_MUL, t.real, eje, cuarto);
            let eje = f.binaria(op::F_MUL, t.real, eje, borde_real);
            f.glsl(t.real, 43, &[eje, cero, maximo])
        });
        let [columna, fila_id] = coordenadas.map(|fraccion_z| f.unaria(op::F_A_U, self.entero, fraccion_z));
        let fracciones = [(coordenadas[0], columna), (coordenadas[1], fila_id)].map(|(fraccion_z, indice)| {
            let indice = f.unaria(op::U_A_F, t.real, indice);
            f.binaria(op::F_SUB, t.real, fraccion_z, indice)
        });
        let fila = f.binaria(op::I_MUL, self.entero, fila_id, lado);
        let base = f.binaria(op::I_ADD, self.entero, fila, columna);
        let derecha = f.binaria(op::I_ADD, self.entero, base, uno);
        let abajo = f.binaria(op::I_ADD, self.entero, base, lado);
        let diagonal = f.binaria(op::I_ADD, self.entero, abajo, uno);
        let [altura_base, altura_derecha, altura_abajo, altura_diagonal] =
            [base, derecha, abajo, diagonal].map(|indice| self.altura(f, t, indice));
        let uno_real = Tipos::constante(f, 1.0);
        let [fraccion_x, fraccion_z] = fracciones;
        let iu = f.binaria(op::F_SUB, t.real, uno_real, fraccion_x);
        let iv = f.binaria(op::F_SUB, t.real, uno_real, fraccion_z);
        let interpolar = |f: &mut Funcion, origen, columna, fila_id, fraccion_x, fraccion_z| {
            let dx = f.binaria(op::F_SUB, t.real, columna, origen);
            let dz = f.binaria(op::F_SUB, t.real, fila_id, origen);
            let dx = f.binaria(op::F_MUL, t.real, dx, fraccion_x);
            let dz = f.binaria(op::F_MUL, t.real, dz, fraccion_z);
            let altura = f.binaria(op::F_ADD, t.real, origen, dx);
            f.binaria(op::F_ADD, t.real, altura, dz)
        };
        let primero = interpolar(f, altura_base, altura_derecha, altura_abajo, fraccion_x, fraccion_z);
        let segundo = interpolar(f, altura_diagonal, altura_abajo, altura_derecha, iu, iv);
        let suma = f.binaria(op::F_ADD, t.real, fraccion_x, fraccion_z);
        let inferior = f.binaria(op::F_ORD_GREATER_THAN, self.booleano, uno_real, suma);
        f.elegir(t.real, inferior, primero, segundo)
    }

    pub(super) fn visibilidad(&self, f: &mut Funcion, t: &Tipos, mundo: Id, destino: Id, pasos: u32) -> Id {
        let cero = f.modulo().constante_u32(0);
        let ocho = f.modulo().constante_u32(8);
        let direccion = f.acceso(self.puntero_entero, self.control, &[cero, ocho]);
        let lado = f.cargar(self.entero, direccion);
        let dos = f.modulo().constante_u32(2);
        let maximo = f.modulo().constante_u32(1024);
        let lado = f.glsl(self.entero, 44, &[lado, dos, maximo]); // UClamp.
        let uno = Tipos::constante(f, 1.0);
        let nulo = Tipos::constante(f, 0.0);
        let margen = Tipos::constante(f, 0.015);
        let ancho = Tipos::constante(f, 2.0);
        let delta = f.binaria(op::F_SUB, t.v3, destino, mundo);
        #[allow(clippy::cast_precision_loss, reason = "presupuesto <=64")]
        let divisor = Tipos::constante(f, (pasos + 1) as f32);
        f.repetir(pasos, t.real, uno, |f, indice, visible| {
            let indice = f.unaria(op::U_A_F, t.real, indice);
            let paso = f.binaria(op::F_ADD, t.real, indice, uno);
            let paso = f.binaria(op::F_DIV, t.real, paso, divisor);
            let avance = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, delta, paso);
            let punto = f.binaria(op::F_ADD, t.v3, mundo, avance);
            let altura = self.terreno(f, t, punto, lado);
            let y = f.extraer(t.real, punto, 1);
            let y = f.binaria(op::F_ADD, t.real, y, margen);
            let tapado = f.binaria(op::F_ORD_GREATER_THAN, self.booleano, altura, y);
            let dentro = [0, 2].map(|i| {
                let eje = f.extraer(t.real, punto, i);
                let eje = f.glsl(t.real, 4, &[eje]);
                f.binaria(op::F_ORD_GREATER_THAN, self.booleano, ancho, eje)
            });
            let dentro = f.binaria(op::LOGICAL_AND, self.booleano, dentro[0], dentro[1]);
            let tapado = f.binaria(op::LOGICAL_AND, self.booleano, tapado, dentro);
            f.elegir(t.real, tapado, nulo, visible)
        })
    }
}
