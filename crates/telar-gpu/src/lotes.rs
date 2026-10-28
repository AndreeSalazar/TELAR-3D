//! Ejecución por lotes sobre una reserva GPU, con descarga únicamente al terminar.
use crate::Calculo;

/// Contrato del kernel: SSBO u32 en set 0, binding 0 y grupo X fijo.
#[derive(Clone, Copy)]
pub struct PlanLotes {
    /// Palabras de salida; máximo 16 millones, distinto del número de invocaciones.
    pub elementos: u32,
    /// Invocaciones lógicas; el shader debe comprobar su límite y el del buffer.
    pub invocaciones: u32,
    /// Tamaño X declarado en SPIR-V, 1..64.
    pub grupo: u32,
    /// Máximo de grupos por envío; limita trabajo, no garantiza tiempo de ejecución.
    pub grupos_por_lote: u32,
}
/// Progreso confirmado por valla; no requiere leer contadores desde la GPU.
#[derive(Clone, Copy)]
pub struct ProgresoLotes {
    /// Invocaciones cuyo lote ya terminó.
    pub completadas: u32,
    /// Número de invocaciones de la petición.
    pub total: u32,
}
impl Calculo {
    /// El shader debe escribir toda la salida válida; no se inicializan palabras sin dueño.
    /// Conserva un pipeline, dos buffers y un comando reutilizable por trabajo.
    /// El callback se ejecuta antes de reservar y después de cada lote; falso cancela.
    /// Devuelve None al cancelar y una descarga final cuando termina. Esta función
    /// espera vallas: ejecútala en un trabajador, no en el bucle de una ventana.
    /// # Errors
    /// Rechaza presupuestos, límites físicos o errores de Vulkan.
    pub fn ejecutar_lotes(
        &self,
        codigo: &[u32],
        plan: PlanLotes,
        continuar: impl FnMut(ProgresoLotes) -> bool,
    ) -> Result<Option<Vec<u32>>, String> {
        if !(1..=16_777_216).contains(&plan.elementos)
            || !(1..=16_777_216).contains(&plan.invocaciones)
            || !(1..=64).contains(&plan.grupo)
            || plan.grupos_por_lote == 0
        {
            return Err("plan de lotes fuera de contrato".into());
        }
        if codigo.len() < 5 || codigo[0] != 0x0723_0203 {
            return Err("cabecera SPIR-V inválida para lotes".into());
        }
        crate::ffi::ejecutar_lotes(&self.contexto, codigo, plan, continuar)
    }
}
