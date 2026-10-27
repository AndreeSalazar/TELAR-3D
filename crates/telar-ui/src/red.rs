//! Grafo unary acotado: identidades estables y validación previa a compilar.
use crate::Estudio;
use alloc::vec::Vec;

/// Tipos que el visor residente sabe compilar; no admite recursos arbitrarios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoNodo {
    /// Fuente única de topología.
    Rejilla,
    /// Onda sumada a la altura actual.
    Onda,
    /// Escala altura y derivadas.
    Relieve,
    /// Destino único del visor.
    Salida,
}

/// Configuración independiente de una operación geométrica.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OperacionRed {
    /// Seno/coseno del plano x/z; cada instancia conserva su velocidad.
    Onda {
        /// Altura en [0,0.8].
        amplitud: f32,
        /// Multiplicador del reloj común en [0,2].
        velocidad: f32,
    },
    /// Factor en [0,3].
    Relieve {
        /// Escala vertical.
        factor: f32,
    },
}

pub(crate) struct Red {
    pub(crate) entradas: Vec<usize>,
    pub(crate) habilitados: Vec<bool>,
    pub(crate) extras: Vec<OperacionRed>,
    pub(crate) activa: bool,
    pub(crate) cambio: bool,
    pub(crate) origen: Option<usize>,
    pub(crate) desplazamiento: f32,
}
impl Default for Red {
    fn default() -> Self {
        Self {
            entradas: alloc::vec![usize::MAX, 0, 1, 2],
            habilitados: alloc::vec![true; 4],
            extras: Vec::new(),
            activa: false,
            cambio: false,
            origen: None,
            desplazamiento: 0.0,
        }
    }
}

impl Estudio {
    /// Cantidad de identidades del estudio, incluyendo fuente y salida.
    #[must_use]
    pub fn cantidad_nodos(&self) -> usize {
        self.red.entradas.len()
    }

    /// Indica que se usa el compilador de conexiones editables.
    #[must_use]
    pub fn red_activa(&self) -> bool {
        self.red.activa
    }

    /// Extrae una petición de recompilar, solo producida por ediciones confirmadas.
    pub fn tomar_cambio_red(&mut self) -> bool {
        core::mem::take(&mut self.red.cambio)
    }

    /// Solicita reconstruir también el plan histórico al abrir otra escena.
    pub fn solicitar_plan(&mut self) {
        self.red.cambio = true;
    }

    /// Tipo de una identidad existente.
    #[must_use]
    pub fn tipo_nodo(&self, id: usize) -> Option<TipoNodo> {
        Some(match id {
            0 => TipoNodo::Rejilla,
            1 => TipoNodo::Onda,
            2 => TipoNodo::Relieve,
            3 => TipoNodo::Salida,
            _ => match self.red.extras.get(id.checked_sub(4)?)? {
                OperacionRed::Onda { .. } => TipoNodo::Onda,
                OperacionRed::Relieve { .. } => TipoNodo::Relieve,
            },
        })
    }

    /// Entrada única de un nodo; la fuente no tiene entrada.
    #[must_use]
    pub fn entrada_nodo(&self, id: usize) -> Option<usize> {
        self.red.entradas.get(id).copied().filter(|id| *id != usize::MAX)
    }

    /// Estado de ejecución; un nodo desactivado deja pasar su entrada.
    #[must_use]
    pub fn nodo_habilitado(&self, id: usize) -> bool {
        self.red.habilitados.get(id).copied().unwrap_or(false)
    }

    pub(crate) fn operacion_nodo(&self, id: usize) -> Option<OperacionRed> {
        match id {
            1 => Some(OperacionRed::Onda { amplitud: self.onda.amplitud, velocidad: self.onda.velocidad }),
            2 => Some(OperacionRed::Relieve { factor: self.relieve.factor }),
            _ => self.red.extras.get(id.checked_sub(4)?).copied(),
        }
    }

    /// Crea e inserta una operación antes de Salida; hasta ocho operaciones.
    /// # Errors
    /// Rechaza fuente/salida adicionales y superar diez identidades totales.
    pub fn crear_nodo(&mut self, tipo: TipoNodo) -> Result<usize, &'static str> {
        if self.cantidad_nodos() >= 10 {
            return Err("límite: ocho operaciones por escena");
        }
        let operacion = match tipo {
            TipoNodo::Onda => OperacionRed::Onda { amplitud: 0.2, velocidad: 0.7 },
            TipoNodo::Relieve => OperacionRed::Relieve { factor: 1.0 },
            _ => return Err("el visor admite una Rejilla y una Salida"),
        };
        let id = self.cantidad_nodos();
        self.red.extras.push(operacion);
        self.red.entradas.push(self.red.entradas[3]);
        self.red.entradas[3] = id;
        self.red.habilitados.push(true);
        self.nodos.push([0.05, self.nodos[3][1]]);
        self.nodos[3][1] += 0.9;
        self.seleccionado = id;
        self.soltar();
        self.edicion = None;
        self.activar_red();
        Ok(id)
    }

    /// Reemplaza una conexión sin permitir referencias ausentes ni ciclos.
    /// # Errors
    /// Rechaza conectar a fuente, desde salida, a sí mismo o formar un ciclo.
    pub fn conectar_nodos(&mut self, desde: usize, hasta: usize) -> Result<(), &'static str> {
        let n = self.cantidad_nodos();
        if desde >= n || hasta >= n || hasta == 0 || desde == 3 {
            return Err("puertos incompatibles");
        }
        let mut cursor = desde;
        for _ in 0..n {
            if cursor == hasta {
                return Err("la conexión formaría un ciclo");
            }
            if cursor == 0 {
                self.red.entradas[hasta] = desde;
                self.activar_red();
                return Ok(());
            }
            cursor = self.red.entradas[cursor];
        }
        Err("grafo sin raíz válida")
    }

    /// Desactiva una operación por paso directo sin eliminar sus parámetros.
    /// # Errors
    /// Fuente, salida e identidades ausentes no se pueden desactivar.
    pub fn habilitar_nodo(&mut self, id: usize, habilitado: bool) -> Result<(), &'static str> {
        if id == 0 || id == 3 || id >= self.cantidad_nodos() {
            return Err("selecciona una operación");
        }
        self.red.habilitados[id] = habilitado;
        self.activar_red();
        Ok(())
    }

    /// Solo operaciones alcanzables desde Salida, en orden fuente → destino.
    /// El compilador consume esta lista al editar; no se recorre por fotograma.
    /// # Errors
    /// Si los datos de conexiones no llegan a la fuente dentro del límite.
    pub fn operaciones_red(&self) -> Result<Vec<OperacionRed>, &'static str> {
        let mut resultado = Vec::new();
        let mut cursor = self.red.entradas[3];
        for _ in 0..self.cantidad_nodos() {
            if cursor == 0 {
                resultado.reverse();
                return Ok(resultado);
            }
            if self.nodo_habilitado(cursor) {
                resultado.push(self.operacion_nodo(cursor).ok_or("operación desconocida")?);
            }
            cursor = *self.red.entradas.get(cursor).ok_or("entrada ausente")?;
        }
        Err("ciclo o raíz ausente")
    }

    pub(crate) fn activar_red(&mut self) {
        if !self.red.activa {
            let cantidad = self.cantidad_nodos();
            for (id, posicion) in self.nodos.iter_mut().enumerate() {
                let orden = if id == 3 {
                    cantidad - 1
                } else if id >= 4 {
                    id - 1
                } else {
                    id
                };
                #[allow(clippy::cast_precision_loss, reason = "hasta diez nodos")]
                let y = orden as f32 * 0.9;
                *posicion = [if orden.is_multiple_of(2) { 0.05 } else { 0.8 }, y];
            }
        }
        self.red.activa = true;
        self.red.cambio = true;
    }
    pub(crate) fn marcar_red(&mut self) {
        self.red.cambio |=
            self.red.activa && matches!(self.tipo_nodo(self.seleccionado), Some(TipoNodo::Onda | TipoNodo::Relieve));
    }
}
