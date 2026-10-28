//! ABI de humo 2D: doce planos f32 en set 0/binding 1, constantes de 128 B.
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
pub(super) struct Memoria {
    pub real: Id,
    pub entero: Id,
    pub booleano: Id,
    pub vector: Id,
    pub buffer: Id,
    puntero: Id,
    constantes: Id,
    push: Id,
    pub lado: u32,
    pub volumen: bool,
}
impl Memoria {
    pub(super) fn nueva(m: &mut Modulo, lado: u32, volumen: bool) -> Self {
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
            lado,
            volumen,
        }
    }
    pub(super) fn celdas(&self) -> u32 {
        self.lado * self.lado * if self.volumen { self.lado } else { 1 }
    }
    pub(super) fn div(&self) -> u32 {
        if self.volumen { 10 } else { 8 }
    }
    pub(super) fn qb(&self) -> u32 {
        if self.volumen { 12 } else { 10 }
    }
    pub(super) fn residual(&self) -> u32 {
        if self.volumen { 14 } else { 12 }
    }
    pub(super) fn parciales(&self) -> u32 {
        if self.volumen { 15 } else { 13 }
    }
    pub(super) fn control(&self) -> u32 {
        if self.volumen { 16 } else { 14 }
    }
    pub(super) fn vecinos_todos(&self, f: &mut Funcion, pixel: Id) -> alloc::vec::Vec<(Id, Id)> {
        if self.volumen { self.vecinos_fluidos(f, pixel).to_vec() } else { self.vecinos(f, pixel).to_vec() }
    }
    pub(super) fn c(f: &mut Funcion, v: f32) -> Id {
        f.modulo().constante_f32(v)
    }
    pub(super) fn u(f: &mut Funcion, v: u32) -> Id {
        f.modulo().constante_u32(v)
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
    pub(super) fn escala(&self, f: &mut Funcion, a: Id, b: f32) -> Id {
        let b = Self::c(f, b);
        self.mul(f, a, b)
    }
    pub(super) fn limitar(&self, f: &mut Funcion, a: Id, min: f32, max: f32) -> Id {
        let min = Self::c(f, min);
        let max = Self::c(f, max);
        f.glsl(self.real, 43, &[a, min, max])
    }
    pub(super) fn parametro(&self, f: &mut Funcion, i: u32) -> Id {
        let indice = Self::u(f, i / 4);
        let p = f.acceso(self.push, self.constantes, &[indice]);
        let v = f.cargar(self.vector, p);
        f.extraer(self.real, v, i % 4)
    }
    fn direccion(&self, f: &mut Funcion, pixel: Id, canal: u32) -> Id {
        let cero = Self::u(f, 0);
        let inicio = Self::u(f, canal * self.celdas());
        let indice = f.binaria(op::I_ADD, self.entero, pixel, inicio);
        f.acceso(self.puntero, self.buffer, &[cero, indice])
    }
    pub(super) fn leer(&self, f: &mut Funcion, pixel: Id, canal: u32) -> Id {
        let ptr = self.direccion(f, pixel, canal);
        f.cargar(self.real, ptr)
    }
    pub(super) fn guardar(&self, f: &mut Funcion, pixel: Id, canal: u32, v: Id) {
        let ptr = self.direccion(f, pixel, canal);
        // GPU: cada invocación posee su celda; las pasadas leen planos diferentes y el ejecutor las ordena.
        f.guardar(ptr, v);
    }
    pub(super) fn coords(&self, f: &mut Funcion, pixel: Id) -> [Id; 2] {
        let lado = Self::u(f, self.lado);
        [f.binaria(op::U_MOD, self.entero, pixel, lado), f.binaria(op::U_DIV, self.entero, pixel, lado)]
    }
    /// Vecinos seguros: izquierdo, derecho, abajo y arriba; booleanos indican existencia.
    pub(super) fn vecinos(&self, f: &mut Funcion, pixel: Id) -> [(Id, Id); 4] {
        let [x, y] = self.coords(f, pixel);
        let cero = Self::u(f, 0);
        let max = Self::u(f, self.lado - 1);
        let uno = Self::u(f, 1);
        let lado = Self::u(f, self.lado);
        [(x, cero, uno, true), (x, max, uno, false), (y, cero, lado, true), (y, max, lado, false)].map(
            |(coord, borde, paso, restar)| {
                let existe = f.binaria(op::I_NOT_EQUAL, self.booleano, coord, borde);
                let otro = f.binaria(if restar { op::I_SUB } else { op::I_ADD }, self.entero, pixel, paso);
                (f.elegir(self.entero, existe, otro, pixel), existe)
            },
        )
    }
    /// Interpolación bilineal; escalares se sujetan al dominio, velocidades tienen halo cero.
    #[allow(clippy::cast_precision_loss, reason = "lado <=256")]
    pub(super) fn muestra(&self, f: &mut Funcion, coords: [Id; 2], canal: u32) -> Id {
        let cero = Self::u(f, 0);
        let uno = Self::u(f, 1);
        let max = Self::u(f, self.lado - 1);
        let lado = Self::u(f, self.lado);
        let halo = canal < 2;
        let minimo = if halo { -1.0 } else { 0.0 };
        let mut fracciones = [Self::c(f, 0.0); 2];
        let mut indices = [cero; 2];
        for i in 0..2 {
            let coord = self.limitar(f, coords[i], minimo, (self.lado - 1) as f32);
            let despl = Self::c(f, 1.0);
            let desplazada = self.add(f, coord, despl);
            indices[i] = f.unaria(op::F_A_U, self.entero, desplazada);
            let entero = f.unaria(op::U_A_F, self.real, indices[i]);
            fracciones[i] = self.sub(f, desplazada, entero);
        }
        let mut valores = [Self::c(f, 0.0); 4];
        for (i, (dx, dy)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
            let dx = Self::u(f, dx);
            let dy = Self::u(f, dy);
            let ix = f.binaria(op::I_ADD, self.entero, indices[0], dx);
            let iy = f.binaria(op::I_ADD, self.entero, indices[1], dy);
            let valido_x = f.binaria(op::I_NOT_EQUAL, self.booleano, ix, cero);
            let valido_y = f.binaria(op::I_NOT_EQUAL, self.booleano, iy, cero);
            let valido = f.binaria(op::LOGICAL_AND, self.booleano, valido_x, valido_y);
            let ix = f.glsl(self.entero, 41, &[ix, uno]);
            let iy = f.glsl(self.entero, 41, &[iy, uno]);
            let ix = f.binaria(op::I_SUB, self.entero, ix, uno);
            let iy = f.binaria(op::I_SUB, self.entero, iy, uno);
            let ix = f.glsl(self.entero, 38, &[ix, max]);
            let iy = f.glsl(self.entero, 38, &[iy, max]);
            let fila = f.binaria(op::I_MUL, self.entero, iy, lado);
            let pixel = f.binaria(op::I_ADD, self.entero, fila, ix);
            let valor = self.leer(f, pixel, canal);
            let vacio = Self::c(f, 0.0);
            valores[i] = if halo { f.elegir(self.real, valido, valor, vacio) } else { valor };
        }
        let a = self.mezclar(f, valores[0], valores[1], fracciones[0]);
        let b = self.mezclar(f, valores[2], valores[3], fracciones[0]);
        self.mezclar(f, a, b, fracciones[1])
    }
    fn mezclar(&self, f: &mut Funcion, a: Id, b: Id, peso: Id) -> Id {
        let delta = self.sub(f, b, a);
        let delta = self.mul(f, delta, peso);
        self.add(f, a, delta)
    }
}
