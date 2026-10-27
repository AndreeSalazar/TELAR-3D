//! Identidad y límites consultados al dispositivo; no contienen objetos Vulkan.
use crate::{Calculo, Gpu};

/// Perfil portable de la GPU seleccionada. Los tamaños son límites, no medidas de velocidad.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PerfilGpu {
    /// Identificador de fabricante Vulkan.
    pub fabricante: u32,
    /// Identificador de dispositivo Vulkan.
    pub dispositivo: u32,
    /// Tipo Vulkan: 1 integrada, 2 dedicada, 3 virtual, 4 CPU, 0 otro.
    pub tipo: u32,
    /// Versión cruda del controlador; su codificación depende del fabricante.
    pub controlador: u32,
    /// Versión Vulkan anunciada por el dispositivo.
    pub vulkan: u32,
    /// Grupos de cómputo máximos en X por despacho.
    pub grupos_x: u32,
    /// Invocaciones máximas por grupo.
    pub invocaciones: u32,
    /// Tamaño máximo del grupo en X.
    pub hilos_x: u32,
    /// Bytes compartidos máximos por grupo.
    pub compartida: u32,
    /// Rango máximo de un descriptor de almacenamiento, en bytes.
    pub rango_almacen: u32,
    /// Límite conservador de descriptores de almacenamiento por etapa/conjunto.
    pub almacenes: u32,
    /// Suma de heaps `DEVICE_LOCAL`; en UMA puede ser memoria compartida. No es memoria libre.
    pub memoria_local: u64,
}

impl Gpu {
    /// Perfil consultado de la GPU que presenta la ventana.
    #[must_use]
    pub fn perfil(&self) -> PerfilGpu {
        self.contexto.perfil
    }
}

impl Calculo {
    /// Perfil consultado de la GPU de cálculo.
    #[must_use]
    pub fn perfil(&self) -> PerfilGpu {
        self.contexto.perfil
    }
}
