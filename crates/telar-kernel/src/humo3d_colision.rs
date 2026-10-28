//! Obstáculo esférico voxelizado: caras cerradas y presión Neumann compatible.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
impl Memoria {
    #[allow(clippy::cast_precision_loss, reason = "lado <=64")]
    pub(super) fn fluido(&self, f: &mut Funcion, pixel: Id) -> Id {
        let coords = self.coords3(f, pixel);
        let mut distancia = Self::c(f, 0.0);
        let altura = self.parametro(f, 15);
        let medio = Self::c(f, 0.5);
        for (eje, coord) in coords.into_iter().enumerate() {
            let coord = f.unaria(op::U_A_F, self.real, coord);
            let coord = self.add(f, coord, medio);
            let coord = self.escala(f, coord, 1.0 / self.lado as f32);
            let delta = self.sub(f, coord, if eje == 1 { altura } else { medio });
            let cuadrado = self.mul(f, delta, delta);
            distancia = self.add(f, distancia, cuadrado);
        }
        let radio = self.parametro(f, 14);
        let radio2 = self.mul(f, radio, radio);
        let fuera = f.binaria(op::F_ORD_GREATER_THAN, self.booleano, distancia, radio2);
        let cero = Self::c(f, 0.0);
        let desactivado = f.binaria(188, self.booleano, radio, cero);
        f.binaria(166, self.booleano, fuera, desactivado) // OpLogicalOr.
    }
    pub(super) fn vecinos_fluidos(&self, f: &mut Funcion, pixel: Id) -> [(Id, Id); 6] {
        let actual = self.fluido(f, pixel);
        self.vecinos3(f, pixel).map(|(vecino, existe)| {
            let otro = self.fluido(f, vecino);
            let abierto = f.binaria(op::LOGICAL_AND, self.booleano, actual, otro);
            (vecino, f.binaria(op::LOGICAL_AND, self.booleano, existe, abierto))
        })
    }
}
