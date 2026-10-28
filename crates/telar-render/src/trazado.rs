//! Operaciones SSA para una escena pequeña; búsqueda exhaustiva determinista.
use crate::{
    escena_rayos::{Trabajo, cruz, restar},
    rayos::{Rayo, intersectar},
};
use hilo_spirv::{Funcion, Id, Modulo, op};

pub(crate) struct Matematica {
    pub real: Id,
    pub vector: Id,
    pub entero: Id,
    pub booleano: Id,
}
pub(crate) struct Cercano {
    pub t: Id,
    pub normal: Id,
    pub color: Id,
    pub existe: Id,
}

impl Matematica {
    pub(crate) fn nueva(m: &mut Modulo) -> Self {
        let real = m.tipo_f32();
        let vector = m.tipo_vector(real, 3);
        Self { real, vector, entero: m.tipo_u32(), booleano: m.tipo_bool() }
    }
    pub(crate) fn escalar(f: &mut Funcion, v: f32) -> Id {
        f.modulo().constante_f32(v)
    }
    pub(crate) fn vector(&self, f: &mut Funcion, v: [f32; 3]) -> Id {
        let ids = v.map(|v| Self::escalar(f, v));
        f.construir(self.vector, &ids)
    }
    pub(crate) fn elegir_vector(&self, f: &mut Funcion, condicion: Id, si: Id, no: Id) -> Id {
        let booleanos = f.modulo().tipo_vector(self.booleano, 3);
        let mascara = f.construir(booleanos, &[condicion; 3]);
        f.elegir(self.vector, mascara, si, no)
    }
    pub(crate) fn normalizar(&self, f: &mut Funcion, v: Id) -> Id {
        let largo = f.binaria(op::DOT, self.real, v, v);
        let minimo = Self::escalar(f, 1e-12);
        let largo = f.glsl(self.real, op::GLSL_MAXIMO, &[largo, minimo]);
        let inversa = f.glsl(self.real, 32, &[largo]);
        f.binaria(op::VECTOR_TIMES_SCALAR, self.vector, v, inversa)
    }
    pub(crate) fn cercano(&self, f: &mut Funcion, trabajo: &Trabajo, rayo: &Rayo) -> Cercano {
        let mut t = rayo.maximo;
        let mut normal = self.vector(f, [0.0, 1.0, 0.0]);
        let mut color = self.vector(f, [0.0; 3]);
        let cero = Self::escalar(f, 0.0);
        let mut existe = f.binaria(op::F_ORD_GREATER_THAN, self.booleano, cero, cero);
        for triangulo in &trabajo.triangulos {
            let vertices = triangulo.vertices.map(|v| self.vector(f, v));
            let impacto = intersectar(f, rayo, vertices);
            let antes = f.binaria(op::F_ORD_GREATER_THAN, self.booleano, t, impacto.t);
            let elegir = f.binaria(op::LOGICAL_AND, self.booleano, impacto.existe, antes);
            existe = f.binaria(op::LOGICAL_OR, self.booleano, existe, elegir);
            t = f.elegir(self.real, elegir, impacto.t, t);
            let [primero, segundo, tercero] = triangulo.vertices;
            let cara = self.vector(f, cruz(restar(segundo, primero), restar(tercero, primero)));
            normal = self.elegir_vector(f, elegir, cara, normal);
            let material = self.vector(f, triangulo.color);
            color = self.elegir_vector(f, elegir, material, color);
        }
        Cercano { t, normal: self.normalizar(f, normal), color, existe }
    }
    pub(crate) fn visible(&self, f: &mut Funcion, trabajo: &Trabajo, rayo: &Rayo) -> Id {
        let uno = Self::escalar(f, 1.0);
        let cero = Self::escalar(f, 0.0);
        let mut visible = uno;
        if trabajo.sombras {
            for triangulo in &trabajo.triangulos {
                let vertices = triangulo.vertices.map(|v| self.vector(f, v));
                let impacto = intersectar(f, rayo, vertices);
                visible = f.elegir(self.real, impacto.existe, cero, visible);
            }
        }
        visible
    }
    pub(crate) fn azar(&self, f: &mut Funcion, semilla: Id, sal: u32) -> Id {
        let sal = f.modulo().constante_u32(sal);
        let mut valor = f.binaria(op::I_ADD, self.entero, semilla, sal);
        for (desplazamiento, multiplicador) in [(16, 0x7feb_352d), (15, 0x846c_a68b)] {
            let giro = f.modulo().constante_u32(desplazamiento);
            let alto = f.binaria(op::SHIFT_RIGHT_LOGICAL, self.entero, valor, giro);
            valor = f.binaria(op::BITWISE_XOR, self.entero, valor, alto);
            let constante = f.modulo().constante_u32(multiplicador);
            valor = f.binaria(op::I_MUL, self.entero, valor, constante);
        }
        let ocho = f.modulo().constante_u32(8);
        let valor = f.binaria(op::SHIFT_RIGHT_LOGICAL, self.entero, valor, ocho);
        let valor = f.unaria(op::U_A_F, self.real, valor);
        let escala = Self::escalar(f, 1.0 / 16_777_216.0);
        f.binaria(op::F_MUL, self.real, valor, escala)
    }
}
