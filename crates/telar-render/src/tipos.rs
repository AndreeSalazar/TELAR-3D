//! Tipos y operaciones compartidos por los shaders originales del renderizador.
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Tipos {
    pub real: Id,
    pub v3: Id,
    pub v4: Id,
    pub constantes: Id,
    pub puntero: Id,
}

impl Tipos {
    pub(super) fn nuevos(modulo: &mut Modulo) -> Self {
        let real = modulo.tipo_f32();
        let v3 = modulo.tipo_vector(real, 3);
        let v4 = modulo.tipo_vector(real, 4);
        let bloque = modulo.tipo_bloque(&[v4; 8], &[0, 16, 32, 48, 64, 80, 96, 112]);
        let tipo = modulo.tipo_puntero(Clase::Constantes, bloque);
        let constantes = modulo.variable(tipo, Clase::Constantes);
        let puntero = modulo.tipo_puntero(Clase::Constantes, v4);
        Self { real, v3, v4, constantes, puntero }
    }
    pub(super) fn cargar(&self, f: &mut Funcion, indice: u32) -> Id {
        let indice = f.modulo().constante_u32(indice);
        let direccion = f.acceso(self.puntero, self.constantes, &[indice]);
        f.cargar(self.v4, direccion)
    }
    pub(super) fn xyz(&self, f: &mut Funcion, valor: Id) -> Id {
        f.barajar(self.v3, valor, valor, &[0, 1, 2])
    }
    pub(super) fn constante(f: &mut Funcion, valor: f32) -> Id {
        f.modulo().constante_f32(valor)
    }
    pub(super) fn rgb(&self, f: &mut Funcion, valores: [f32; 3]) -> Id {
        let ids = valores.map(|v| Self::constante(f, v));
        f.construir(self.v3, &ids)
    }
    pub(super) fn saturar(&self, f: &mut Funcion, valor: Id) -> Id {
        let cero = Self::constante(f, 0.0);
        let uno = Self::constante(f, 1.0);
        f.glsl(self.real, 43, &[valor, cero, uno])
    }
    pub(super) fn normalizar(&self, f: &mut Funcion, valor: Id) -> Id {
        let longitud = f.binaria(op::DOT, self.real, valor, valor);
        let epsilon = Self::constante(f, 0.000_001);
        let longitud = f.glsl(self.real, op::GLSL_MAXIMO, &[longitud, epsilon]);
        let inversa = f.glsl(self.real, 32, &[longitud]); // InverseSqrt.
        f.binaria(op::VECTOR_TIMES_SCALAR, self.v3, valor, inversa)
    }
}

pub(super) fn variable(m: &mut Modulo, tipo: Id, clase: Clase, ubicacion: u32) -> Id {
    let puntero = m.tipo_puntero(clase, tipo);
    let variable = m.variable(puntero, clase);
    m.decorar(variable, op::DECORACION_UBICACION, &[ubicacion]);
    variable
}
