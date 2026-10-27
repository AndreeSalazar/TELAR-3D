//! Extensión de escena v3: datos de conexiones, operaciones y disposición completa.
use crate::{Estudio, OperacionRed, red::Red};
use alloc::vec::Vec;

impl Estudio {
    /// Exporta la red editada; vacía significa la cadena histórica v2.
    #[must_use]
    pub fn guardar_red(&self) -> Vec<u32> {
        if !self.red.activa {
            return Vec::new();
        }
        let entero = |cantidad| u32::try_from(cantidad).unwrap_or(u32::MAX);
        let mut palabras = alloc::vec![entero(self.cantidad_nodos()), entero(self.seleccionado)];
        for id in 0..self.cantidad_nodos() {
            let (tipo, a, b) = match self.operacion_nodo(id) {
                Some(OperacionRed::Onda { amplitud, velocidad }) => (1, amplitud, velocidad),
                Some(OperacionRed::Relieve { factor }) => (2, factor, 0.0),
                None => (if id == 0 { 0 } else { 3 }, 0.0, 0.0),
            };
            palabras.extend([
                entero(self.red.entradas[id]),
                u32::from(self.red.habilitados[id]),
                self.nodos[id][0].to_bits(),
                self.nodos[id][1].to_bits(),
                tipo,
                a.to_bits(),
                b.to_bits(),
            ]);
        }
        palabras
    }

    /// Restaura la extensión solo tras validar tamaños, tipos, dominios y ciclos.
    /// Los parámetros base deben coincidir con el prefijo v2 ya restaurado.
    /// # Errors
    /// Rechaza datos desconocidos, referencias inválidas, ciclos y más de diez nodos.
    pub fn restaurar_red(&mut self, palabras: &[u32]) -> Result<(), &'static str> {
        if palabras.len() < 2 {
            return Err("extensión de red truncada");
        }
        let cantidad = palabras[0] as usize;
        if !(4..=10).contains(&cantidad) || palabras.len() != 2 + cantidad * 7 || palabras[1] as usize >= cantidad {
            return Err("tamaño o selección de red inválidos");
        }
        let mut red =
            Red { entradas: Vec::new(), habilitados: Vec::new(), activa: true, cambio: true, ..Red::default() };
        let mut posiciones = Vec::new();
        for (id, datos) in palabras[2..].as_chunks::<7>().0.iter().enumerate() {
            let entrada = datos[0] as usize;
            if (id == 0 && datos[0] != u32::MAX)
                || (id != 0 && (entrada >= cantidad || entrada == 3))
                || datos[1] > 1
                || (matches!(id, 0 | 3) && datos[1] != 1)
            {
                return Err("entrada o estado de nodo inválidos");
            }
            let (x, y, a, b) = (
                f32::from_bits(datos[2]),
                f32::from_bits(datos[3]),
                f32::from_bits(datos[5]),
                f32::from_bits(datos[6]),
            );
            if !x.is_finite() || !y.is_finite() || !(0.0..=1.0).contains(&x) || !(0.0..=16.0).contains(&y) {
                return Err("posición de red inválida");
            }
            let operacion = match datos[4] {
                1 if a.is_finite() && b.is_finite() && (0.0..=0.8).contains(&a) && (0.0..=2.0).contains(&b) => {
                    Some(OperacionRed::Onda { amplitud: a, velocidad: b })
                }
                2 if a.is_finite() && (0.0..=3.0).contains(&a) && datos[6] == 0 => {
                    Some(OperacionRed::Relieve { factor: a })
                }
                0 | 3 if datos[5] == 0 && datos[6] == 0 => None,
                _ => return Err("tipo o parámetros de nodo inválidos"),
            };
            if id < 4 {
                if datos[4] != u32::try_from(id).map_err(|_| "identidad inválida")?
                    || operacion != self.operacion_nodo(id)
                {
                    return Err("prefijo y red no coinciden");
                }
            } else {
                red.extras.push(operacion.ok_or("solo se duplican operaciones")?);
            }
            red.entradas.push(if id == 0 { usize::MAX } else { entrada });
            red.habilitados.push(datos[1] != 0);
            posiciones.push([x, y]);
        }
        for id in 1..cantidad {
            let mut cursor = id;
            for paso in 0..cantidad {
                if cursor == 0 {
                    break;
                }
                if paso == cantidad - 1 {
                    return Err("ciclo en red de escena");
                }
                cursor = red.entradas[cursor];
            }
        }
        self.red = red;
        self.nodos = posiciones;
        self.seleccionado = palabras[1] as usize;
        self.edicion = None;
        self.arrastre = None;
        self.encima = None;
        Ok(())
    }
}
