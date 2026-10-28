//! Humo MAC 3D denso y acotado; no añade todavía combustión ni colisiones internas.
use crate::humo::{Programas, orden_adaptativo};
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
/// Referencia escalar de ensayo; no se utiliza para producir fotogramas.
#[path = "humo3d_referencia.rs"]
pub mod referencia;
