//! Índices y muestreo MAC trilineal; extiende la memoria común sin cambiar el ABI 2D.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
impl Memoria {
    pub(super) fn coords3(&self, f: &mut Funcion, pixel: Id) -> [Id; 3] {
        let lado = Self::u(f, self.lado);
        let plano = Self::u(f, self.lado * self.lado);
        let fila = f.binaria(op::U_DIV, self.entero, pixel, lado);
        [
            f.binaria(op::U_MOD, self.entero, pixel, lado),
            f.binaria(op::U_MOD, self.entero, fila, lado),
            f.binaria(op::U_DIV, self.entero, pixel, plano),
        ]
    }
    pub(super) fn vecinos3(&self, f: &mut Funcion, pixel: Id) -> [(Id, Id); 6] {
        let coords = self.coords3(f, pixel);
        let cero = Self::u(f, 0);
        let max = Self::u(f, self.lado - 1);
        let mut salida = [(pixel, cero); 6];
        for eje in 0..3 {
            let paso = Self::u(f, [1, self.lado, self.lado * self.lado][eje]);
            for direccion in 0..2 {
                let existe =
                    f.binaria(op::I_NOT_EQUAL, self.booleano, coords[eje], if direccion == 0 { cero } else { max });
                let otro = f.binaria(if direccion == 0 { op::I_SUB } else { op::I_ADD }, self.entero, pixel, paso);
                salida[eje * 2 + direccion] = (f.elegir(self.entero, existe, otro, pixel), existe);
            }
        }
        salida
    }
    #[allow(clippy::cast_precision_loss, reason = "lado <=64")]
    pub(super) fn muestra3(&self, f: &mut Funcion, coords: [Id; 3], canal: u32) -> Id {
        let halo = canal < 3;
        let minimo = if halo { -1.0 } else { 0.0 };
        let cero = Self::u(f, 0);
        let uno = Self::u(f, 1);
        let max = Self::u(f, self.lado - 1);
        let mut pesos = [Self::c(f, 0.0); 3];
        let mut bases = [cero; 3];
        for eje in 0..3 {
            let coord = self.limitar(f, coords[eje], minimo, (self.lado - 1) as f32);
            let real_uno = Self::c(f, 1.0);
            let coord = self.add(f, coord, real_uno);
            bases[eje] = f.unaria(op::F_A_U, self.entero, coord);
            let base = f.unaria(op::U_A_F, self.real, bases[eje]);
            pesos[eje] = self.sub(f, coord, base);
        }
        let mut valores = [Self::c(f, 0.0); 8];
        for (i, valor) in valores.iter_mut().enumerate() {
            let mut valido = f.binaria(op::I_EQUAL, self.booleano, cero, cero);
            let indices = core::array::from_fn::<_, 3, _>(|eje| {
                let offset = Self::u(f, u32::try_from((i >> eje) & 1).unwrap_or(0));
                let indice = f.binaria(op::I_ADD, self.entero, bases[eje], offset);
                let dentro = f.binaria(op::I_NOT_EQUAL, self.booleano, indice, cero);
                valido = f.binaria(op::LOGICAL_AND, self.booleano, valido, dentro);
                let indice = f.glsl(self.entero, 41, &[indice, uno]);
                let indice = f.binaria(op::I_SUB, self.entero, indice, uno);
                f.glsl(self.entero, 38, &[indice, max])
            });
            let lado = Self::u(f, self.lado);
            let plano = f.binaria(op::I_MUL, self.entero, indices[2], lado);
            let fila = f.binaria(op::I_ADD, self.entero, plano, indices[1]);
            let fila = f.binaria(op::I_MUL, self.entero, fila, lado);
            let pixel = f.binaria(op::I_ADD, self.entero, fila, indices[0]);
            let muestra = self.leer(f, pixel, canal);
            let fluido = self.fluido(f, pixel);
            let vacio = Self::c(f, 0.0);
            let muestra = f.elegir(self.real, fluido, muestra, vacio);
            *valor = if halo { f.elegir(self.real, valido, muestra, vacio) } else { muestra };
        }
        for (eje, peso) in pesos.into_iter().enumerate() {
            for i in 0..(4 >> eje) {
                let a = valores[i * 2];
                let b = valores[i * 2 + 1];
                let delta = self.sub(f, b, a);
                let delta = self.mul(f, delta, peso);
                valores[i] = self.add(f, a, delta);
            }
        }
        valores[0]
    }
}
