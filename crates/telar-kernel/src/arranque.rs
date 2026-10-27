//! Encabezado TELAR de solo lectura en enlace 4; separado del módulo SPIR-V.

/// Firma de cuatro bytes TLAR, interpretada como una palabra u32.
pub const FIRMA: u32 = u32::from_le_bytes(*b"TLAR");
/// Versión del contrato de arranque.
pub const VERSION: u32 = 1;
/// Tamaño fijo de la cabecera inicial.
pub const PALABRAS: usize = 32;
/// Capacidades básicas de esta ruta: cálculo e indirectos de Vulkan.
pub const CAPACIDADES: u32 = 3;

/// Datos aportados por el host desde la consulta del dispositivo.
#[derive(Clone, Copy)]
pub struct Dispositivo {
    /// Fabricante.
    pub fabricante: u32,
    /// Dispositivo.
    pub dispositivo: u32,
    /// Tipo Vulkan anunciado por el dispositivo; solo informativo.
    pub tipo: u32,
    /// Versión cruda del controlador.
    pub controlador: u32,
    /// Versión Vulkan anunciada.
    pub vulkan: u32,
    /// Grupos máximos X.
    pub grupos_x: u32,
    /// Invocaciones máximas por grupo.
    pub invocaciones: u32,
    /// Hilos máximos X por grupo.
    pub hilos_x: u32,
    /// Bytes compartidos por grupo.
    pub compartida: u32,
    /// Rango máximo de almacenamiento, bytes.
    pub rango_almacen: u32,
    /// Límite conservador de almacenamiento por etapa/conjunto.
    pub almacenes: u32,
    /// Tamaño de heaps locales, no memoria disponible para reservar.
    pub memoria_local: u64,
}

/// Construye un contrato validado en host antes de subirlo una vez.
///
/// # Errors
/// Si la capacidad, etapas o límites no admiten la ruta de grupos de 64 hilos.
#[allow(clippy::cast_possible_truncation, reason = "separación explícita de u64 en dos mitades u32")]
pub fn cabecera(gpu: Dispositivo, capacidad: u32, etapas: u32) -> Result<[u32; PALABRAS], &'static str> {
    if !(2..=1024).contains(&capacidad) || !(2..=32).contains(&etapas) {
        return Err("capacidad o etapas fuera del contrato de arranque");
    }
    let vertices = capacidad * capacidad * 6;
    let indices = (capacidad - 1).pow(2) * 6;
    let grupos = (capacidad * capacidad).div_ceil(64);
    if gpu.vulkan < 0x0040_1000
        || gpu.grupos_x < grupos
        || gpu.invocaciones < 64
        || gpu.hilos_x < 64
        || gpu.almacenes < 5
        || gpu.rango_almacen < 128
        || gpu.rango_almacen / 4 < vertices
        || gpu.rango_almacen / 4 < indices
    {
        return Err("la GPU no admite los límites solicitados");
    }
    Ok([
        FIRMA,
        VERSION,
        PALABRAS as u32,
        CAPACIDADES,
        CAPACIDADES,
        gpu.fabricante,
        gpu.dispositivo,
        gpu.controlador,
        gpu.vulkan,
        gpu.grupos_x,
        gpu.invocaciones,
        gpu.hilos_x,
        gpu.compartida,
        gpu.rango_almacen,
        gpu.memoria_local as u32,
        (gpu.memoria_local >> 32) as u32,
        capacidad,
        4,
        vertices,
        indices,
        32,
        64,
        etapas,
        grupos,
        1,
        gpu.almacenes,
        gpu.tipo,
        0,
        0,
        0,
        0,
        0,
    ])
}
