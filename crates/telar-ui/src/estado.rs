//! Instantánea portable del estudio; no incluye recursos ni capacidades de una GPU.
use crate::{Estudio, Onda, Rejilla, Relieve, Salida};

/// Palabras de estado de los cuatro nodos; contrato de escena versión 2.
pub const PALABRAS_ESTUDIO: usize = 19;

impl Estudio {
    /// Captura parámetros y distribución sin estado transitorio de interacción.
    #[must_use]
    pub fn guardar_estado(&self) -> [u32; PALABRAS_ESTUDIO] {
        let mut estado = [0; PALABRAS_ESTUDIO];
        estado[..11].copy_from_slice(&[
            self.rejilla.lado,
            self.rejilla.capacidad,
            self.onda.amplitud.to_bits(),
            self.onda.velocidad.to_bits(),
            self.relieve.factor.to_bits(),
            self.salida.exposicion.to_bits(),
            self.salida.rugosidad.to_bits(),
            u32::try_from(self.salida.material).unwrap_or(u32::MAX),
            self.salida.muestras,
            u32::try_from(if self.red.activa { self.seleccionado.min(3) } else { self.seleccionado })
                .unwrap_or(u32::MAX),
            u32::from(self.pausado),
        ]);
        for (destino, origen) in estado[11..].iter_mut().zip(self.nodos.iter().flatten()) {
            *destino = (if self.red.activa { origen.clamp(0.0, 1.0) } else { *origen }).to_bits();
        }
        estado
    }

    /// Restaura solo después de validar todo; conserva estilo y muestras efectivas.
    /// Termina arrastres y realces, que no pertenecen a una escena.
    /// # Errors
    /// Rechaza parámetros fuera del dominio, NaN, infinitos y posiciones inválidas.
    pub fn restaurar_estado(&mut self, estado: &[u32; PALABRAS_ESTUDIO]) -> Result<(), &'static str> {
        if !(2..=1024).contains(&estado[1]) || !(2..=estado[1]).contains(&estado[0]) {
            return Err("resolución de escena inválida");
        }
        let valor = |indice, minimo, maximo| {
            let valor = f32::from_bits(estado[indice]);
            if valor.is_finite() && (minimo..=maximo).contains(&valor) {
                Ok(valor)
            } else {
                Err("parámetro de escena fuera de su dominio")
            }
        };
        let onda = Onda { amplitud: valor(2, 0.0, 0.8)?, velocidad: valor(3, 0.0, 2.0)? };
        let relieve = Relieve { factor: valor(4, 0.0, 3.0)? };
        let exposicion = valor(5, 0.25, 4.0)?;
        let rugosidad = valor(6, 0.08, 1.0)?;
        if estado[7] > 2 || !matches!(estado[8], 1 | 2 | 4 | 8 | 16 | 32 | 64) || estado[9] > 3 || estado[10] > 1 {
            return Err("opción de escena desconocida");
        }
        let mut nodos = [[0.0; 2]; 4];
        for (indice, destino) in nodos.iter_mut().flatten().enumerate() {
            *destino = valor(indice + 11, 0.0, 1.0)?;
        }
        self.rejilla = Rejilla { lado: estado[0], capacidad: estado[1] };
        self.onda = onda;
        self.relieve = relieve;
        self.salida = Salida {
            exposicion,
            rugosidad,
            material: estado[7] as usize,
            muestras: estado[8],
            muestras_reales: self.salida.muestras_reales,
        };
        self.seleccionado = estado[9] as usize;
        self.pausado = estado[10] != 0;
        self.nodos = nodos.to_vec();
        self.red = crate::red::Red::default();
        self.arrastre = None;
        self.encima = None;
        self.accion_proyecto = None;
        self.edicion = None;
        Ok(())
    }
}
