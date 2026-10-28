//! Humo MAC 3D denso y acotado; incluye esfera estática y predictor temporal; sin combustión.
use crate::humo::{Programas, orden_adaptativo};
use alloc::vec::Vec;
/// u,v,w,densidad,calor; cinco provisionales; div,qA,qB,div final,residual,parciales,control.
pub const CANALES: u32 = 17;
/// Plano con el resumen común de convergencia, primeras dieciséis palabras.
pub const CONTROL: u32 = 16;
/// Resolución y presupuesto de compilación; las constantes físicas se envían aparte.
#[derive(Clone, Copy)]
pub struct Configuracion {
    /// Celdas por eje; 4..64, como máximo 262144 celdas.
    pub lado: u32,
    /// Máximo par de iteraciones, 2..128.
    pub iteraciones: u32,
}
impl Configuracion {
    /// Añade reducción CFL GPU y repite 1..4 subpasos por cuadro sin descargar campos.
    /// # Errors
    /// Rechaza resolución, iteraciones o subpasos fuera de contrato.
    pub fn compilar_temporal(self, subpasos: u32) -> Result<Programas, &'static str> {
        let mut plan = self.compilar()?;
        let config = crate::humo::Configuracion { lado: self.lado, iteraciones: self.iteraciones };
        plan.codigos.extend([15, 16].map(|e| crate::humo::programa(config, e, true, true)));
        plan.orden = orden_temporal(self.iteraciones, subpasos)?;
        Ok(plan)
    }
    /// Compila los kernels del volumen y el control de error compartido con 2D.
    /// # Errors
    /// Rechaza resolución o presupuesto antes de crear recursos GPU.
    pub fn compilar(self) -> Result<Programas, &'static str> {
        if !(4..=64).contains(&self.lado) {
            return Err("volumen fuera de 4..64 celdas por eje");
        }
        let orden = orden_adaptativo(self.iteraciones)?;
        let config = crate::humo::Configuracion { lado: self.lado, iteraciones: self.iteraciones };
        let codigos = (0..14).map(|etapa| crate::humo::programa(config, etapa, true, true)).collect();
        Ok(Programas { codigos, orden })
    }
}
/// Cada subpaso mide velocidad/calor, decide dt en GPU y ejecuta presión acotada.
/// # Errors
/// Rechaza subpasos fuera de 1..4 o presión impar/fuera de 2..128.
pub fn orden_temporal(iteraciones: u32, subpasos: u32) -> Result<Vec<usize>, &'static str> {
    if !(1..=4).contains(&subpasos) {
        return Err("subpasos fuera de 1..4");
    }
    let paso = orden_adaptativo(iteraciones)?;
    let mut orden = Vec::new();
    for _ in 0..subpasos {
        orden.extend([14, 15]);
        orden.extend_from_slice(&paso);
    }
    Ok(orden)
}
/// Referencia escalar de ensayo; no se utiliza para producir fotogramas.
#[path = "humo3d_referencia.rs"]
pub mod referencia;
