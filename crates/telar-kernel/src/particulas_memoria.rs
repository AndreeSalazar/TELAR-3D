//! ABI propio de partículas: doce planos de 128 f32 y constantes de 128 B.
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
pub(super) struct Estado {
    pub real: Id,
    pub entero: Id,
    pub booleano: Id,
    pub vector: Id,
    pub buffer: Id,
    puntero: Id,
    push: Id,
    constantes: Id,
}
impl Estado {
    pub(super) fn nuevo(m: &mut Modulo) -> Self {
        let real = m.tipo_f32();
        let entero = m.tipo_u32();
        let booleano = m.tipo_bool();
        let vector = m.tipo_vector(real, 4);
        let array = m.tipo_array_dinamico(real, 4);
        let bloque = m.tipo_bloque(&[array], &[0]);
        let ptr = m.tipo_puntero(Clase::Almacen, bloque);
        let buffer = m.variable(ptr, Clase::Almacen);
        m.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
        m.decorar(buffer, op::DECORACION_ENLACE, &[1]);
        let bloque = m.tipo_bloque(&[vector; 8], &[0, 16, 32, 48, 64, 80, 96, 112]);
        let ptr = m.tipo_puntero(Clase::Constantes, bloque);
        let constantes = m.variable(ptr, Clase::Constantes);
        Self {
            real,
            entero,
            booleano,
            vector,
            buffer,
            puntero: m.tipo_puntero(Clase::Almacen, real),
            push: m.tipo_puntero(Clase::Constantes, vector),
            constantes,
        }
    }
    pub(super) fn c(f: &mut Funcion, v: f32) -> Id {
        f.modulo().constante_f32(v)
    }
    pub(super) fn u(f: &mut Funcion, v: u32) -> Id {
        f.modulo().constante_u32(v)
    }
    pub(super) fn p(&self, f: &mut Funcion, i: u32) -> Id {
        let indice = Self::u(f, i / 4);
        let ptr = f.acceso(self.push, self.constantes, &[indice]);
        let vector = f.cargar(self.vector, ptr);
        f.extraer(self.real, vector, i % 4)
    }
    fn direccion(&self, f: &mut Funcion, id: Id, canal: u32) -> Id {
        let cero = Self::u(f, 0);
        let base = Self::u(f, canal * crate::particulas::CAPACIDAD);
        let indice = f.binaria(op::I_ADD, self.entero, base, id);
        f.acceso(self.puntero, self.buffer, &[cero, indice])
    }
    pub(super) fn leer(&self, f: &mut Funcion, id: Id, canal: u32) -> Id {
        let ptr = self.direccion(f, id, canal);
        f.cargar(self.real, ptr)
    }
    pub(super) fn guardar(&self, f: &mut Funcion, id: Id, canal: u32, valor: Id) {
        let ptr = self.direccion(f, id, canal);
        // GPU: una invocación por plaza; el resumen tiene un único propietario y se ejecuta tras barrera.
        f.guardar(ptr, valor);
    }
    pub(super) fn control(&self, f: &mut Funcion, i: u32) -> Id {
        let id = Self::u(f, i);
        self.leer(f, id, crate::particulas::CONTROL)
    }
    pub(super) fn add(&self, f: &mut Funcion, a: Id, b: Id) -> Id {
        f.binaria(op::F_ADD, self.real, a, b)
    }
    pub(super) fn sub(&self, f: &mut Funcion, a: Id, b: Id) -> Id {
        f.binaria(op::F_SUB, self.real, a, b)
    }
    pub(super) fn mul(&self, f: &mut Funcion, a: Id, b: Id) -> Id {
        f.binaria(op::F_MUL, self.real, a, b)
    }
    pub(super) fn dt(&self, f: &mut Funcion) -> Id {
        let tiempo = self.control(f, 0);
        let limite = Self::c(f, 3600.0);
        let cero = Self::c(f, 0.0);
        let queda = self.sub(f, limite, tiempo);
        let queda = f.glsl(self.real, op::GLSL_MAXIMO, &[queda, cero]);
        let solicitado = self.p(f, 5);
        f.glsl(self.real, 37, &[solicitado, queda])
    }
}
