//! ABI v1: conserva las primeras 16 palabras de la rejilla y añade el buzón.

/// Versión de la interfaz GPU interna, no versión de Vulkan.
pub const VERSION: u32 = 1;
/// Palabras u32 del buffer de control; el host reserva e inicializa 128 bytes.
pub const PALABRAS: usize = 32;
/// Número de servicio pendiente; cero significa buzón vacío.
pub const SOLICITUD: u32 = 16;
/// Estado persistente: cero ejecutando, uno suspendido.
pub const SUSPENDIDO: u32 = 17;
/// Resultado de la última solicitud consumida: cero correcta, uno desconocida.
pub const RESULTADO: u32 = 18;
/// Cantidad de servicios válidos atendidos, contador modular u32.
pub const ATENDIDAS: u32 = 19;
/// Versión escrita por el coordinador.
pub const VERSION_CONTROL: u32 = 20;
/// Código de la última solicitud consumida, incluso si es desconocida.
pub const ULTIMA: u32 = 21;
/// Vueltas del coordinador, contador modular u32; empieza en uno.
pub const VUELTA: u32 = 22;
/// Solicitudes rechazadas porque el buzón estaba ocupado.
pub const OCUPADAS: u32 = 23;
/// Solicitudes con código desconocido consumidas y rechazadas.
pub const DESCONOCIDAS: u32 = 24;

/// Servicios internos. No invocan llamadas al sistema del host.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Servicio {
    /// Obliga a generar de nuevo la topología cuando la superficie esté activa.
    RehacerTopologia = 1,
    /// Publica despacho y dibujo vacíos, conservando los buffers y su caché.
    Suspender = 2,
    /// Vuelve a habilitar el trabajo; la capacidad sigue verificándose.
    Reanudar = 3,
}
