//! Observación de trabajo terminado; valores ausentes nunca se interpretan como cero.
use crate::Gpu;
/// Capacidad y estimaciones del driver por heap; local puede ser compartida en UMA.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoriaHeap {
    /// Capacidad anunciada en bytes, no memoria libre.
    pub capacidad: u64,
    /// Heap `DEVICE_LOCAL`.
    pub local: bool,
    /// Uso estimado del proceso, si existe `EXT_memory_budget`.
    pub uso: Option<u64>,
    /// Presupuesto estimado para el proceso, puede variar por actividad externa.
    pub presupuesto: Option<u64>,
}
/// Último cuadro terminado de la cola; retraso de hasta dos cuadros.
#[derive(Clone, Copy, Debug, Default)]
pub struct TelemetriaGpu {
    /// Número de observaciones completadas.
    pub muestras: u64,
    /// Intervalos GPU en ms: cálculo del comando, escena, overlay/finalización.
    pub milisegundos: Option<[f64; 3]>,
    /// Copia de dieciséis palabras de control, interpretadas por la aplicación.
    pub control: Option<[f32; 16]>,
}
/// Tiempos de pared del último cuadro enviado; no son porcentaje de CPU ocupada.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TelemetriaHost {
    /// Cuadros con medición host completada.
    pub cuadros: u64,
    /// Preparación del visor/pipeline, disponible en `dibujar_bloque_interfaz`.
    pub preparacion_ms: Option<f64>,
    /// Espera de la valla del juego.
    pub valla_ms: f64,
    /// Recogida del snapshot y consultas de la ranura terminada.
    pub observacion_ms: f64,
    /// Adquisición de una imagen de presentación.
    pub adquirir_ms: f64,
    /// Actualización de ranura, registro de comandos y reinicio de valla.
    pub grabar_ms: f64,
    /// Duración de vkQueueSubmit en host, puede incluir bloqueo del driver.
    pub enviar_ms: f64,
    /// Duración de vkQueuePresentKHR en host, puede incluir espera.
    pub presentar_ms: f64,
}
/// Plan de campos grabado: presupuesto de envíos, no despachos que hicieron trabajo útil.
#[derive(Clone, Copy, Debug, Default)]
pub struct CargaCampos {
    /// Bytes lógicos de campos, incluido el buffer auxiliar de cuatro bytes.
    pub bytes: u64,
    /// Máximo de grupos de 64 hilos del plan; las rutas pueden reducirlo.
    pub grupos: u32,
    /// Ranuras de la secuencia, incluidas las que el shader omite.
    pub ranuras: usize,
    /// Pipelines preparados.
    pub programas: usize,
    /// Grabaciones de secundarios compute del visor.
    pub grabaciones: u64,
    /// Ejecuciones del visor que reutilizaron un secundario.
    pub reusos: u64,
    /// Cuadros compute grabados directamente al desactivar caché.
    pub directas: u64,
    /// Programas de la ruta configurados para leer argumentos GPU.
    pub indirectos: usize,
}
impl Gpu {
    /// Último desglose host; no espera y no añade consultas Vulkan.
    #[must_use]
    pub fn telemetria_host(&self) -> Option<TelemetriaHost> {
        self.presentacion.observacion_host()
    }
    /// Activa medición y copia opcional de 16 palabras a partir del offset dado.
    /// Reutiliza las vallas del presentador; no espera otros cuadros al consultar.
    /// # Errors
    /// Rechaza rango sin campos o errores al crear recursos de observación.
    pub fn activar_telemetria(&mut self, activa: bool, control: Option<u32>) -> Result<(), String> {
        if let Some(offset) = control {
            self.visor.as_ref().ok_or("telemetría requiere campos")?.validar_control(offset)?;
        }
        self.presentacion.telemetria(&self.contexto, activa, control)
    }
    /// Última muestra terminada; no añade esperas, consultas o transferencias.
    #[must_use]
    pub fn telemetria(&self) -> TelemetriaGpu {
        self.presentacion.observacion()
    }
    /// Consulta ligera al driver; no sincroniza ni lee los campos de simulación.
    #[must_use]
    pub fn memoria_heaps(&self) -> Vec<MemoriaHeap> {
        crate::ffi::memoria_viva::consultar(&self.contexto)
    }
    /// Tamaños de la secuencia vigente, sin consultar la GPU.
    #[must_use]
    pub fn carga_campos(&self) -> CargaCampos {
        self.visor.as_ref().map_or_else(CargaCampos::default, crate::ffi::Visor::carga_campos)
    }
}
pub(crate) fn intervalos(tics: [u64; 4], bits: u32, periodo: f32) -> Option<[f64; 3]> {
    if !(1..=64).contains(&bits) || !periodo.is_finite() || periodo <= 0.0 {
        return None;
    }
    let mascara = u64::MAX >> (64 - bits);
    #[allow(clippy::cast_precision_loss, reason = "conversión de diferencias de tics a milisegundos")]
    Some(core::array::from_fn(|i| (tics[i + 1].wrapping_sub(tics[i]) & mascara) as f64 * f64::from(periodo) / 1e6))
}
#[cfg(test)]
mod tests {
    #[test]
    fn diferencia_respeta_bits_y_overflow() {
        assert_eq!(super::intervalos([254, 1, 3, 6], 8, 1000.0), Some([0.003, 0.002, 0.003]));
        assert!(super::intervalos([0; 4], 0, 1.0).is_none());
        assert!(super::intervalos([0; 4], 64, f32::NAN).is_none());
    }
}
