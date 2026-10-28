//! Ejecución de campos residentes: programas y orden opacos al backend.
use crate::{Gpu, Sombreadores};
/// Un buffer de palabras compartido entre pasadas, con propietario por elemento.
#[derive(Clone, Copy)]
pub struct PlanCampos<'a> {
    /// Invocaciones por pasada; local X=64.
    pub elementos: u32,
    /// Palabras u32/f32 por elemento; el shader define su distribución.
    pub palabras: u32,
    /// SPIR-V distintos, hasta 16; set 0/binding 1 es el estado.
    pub programas: &'a [Vec<u32>],
    /// Índices de programas en orden, hasta 512 pasadas por cuadro.
    pub secuencia: &'a [usize],
}
impl PlanCampos<'_> {
    pub(crate) fn validar(&self) -> Result<u64, String> {
        if !(1..=1_048_576).contains(&self.elementos)
            || !(1..=16).contains(&self.palabras)
            || self.programas.is_empty()
            || self.programas.len() > 16
            || self.secuencia.is_empty()
            || self.secuencia.len() > 512
            || self.secuencia.iter().any(|v| *v >= self.programas.len())
            || self.programas.iter().any(|p| p.len() < 5 || p[0] != 0x0723_0203)
        {
            return Err("plan de campos fuera de contrato".into());
        }
        Ok(u64::from(self.elementos) * u64::from(self.palabras) * 4)
    }
}
impl Gpu {
    /// Inicializa a cero un estado persistente y prepara pasadas compute antes del dibujo.
    /// El buffer reside en GPU; las constantes gráficas de 128 B se comparten con compute.
    /// # Errors
    /// Rechaza presupuesto, secuencia o fallos Vulkan.
    pub fn preparar_campos(&mut self, shaders: Sombreadores, plan: PlanCampos) -> Result<(), String> {
        plan.validar()?;
        if self.visor.is_none() {
            self.preparar_visor(shaders)?;
        }
        self.visor.as_mut().ok_or("falta visor")?.preparar_campos(
            &self.contexto,
            (shaders.vertices, shaders.fragmentos),
            plan,
        )
    }
    /// Cambia el orden de pasadas para los siguientes envíos conservando todo el estado.
    /// # Errors
    /// Rechaza ausencia de campos o índices fuera del plan preparado.
    pub fn secuencia_campos(&mut self, nueva: &[usize]) -> Result<(), String> {
        self.visor.as_mut().ok_or("falta visor")?.secuencia_campos(nueva)
    }
    /// Reinicia los campos con un llenado GPU. Espera la operación; solo por solicitud.
    /// # Errors
    /// Rechaza ausencia de campos o errores Vulkan.
    pub fn limpiar_campos(&self) -> Result<(), String> {
        self.visor.as_ref().ok_or("falta visor")?.limpiar_campos(&self.contexto)
    }
    /// Detiene/reanuda compute conservando campos y dibujo; no espera una valla.
    /// # Errors
    /// Rechaza ausencia de campos.
    pub fn activar_campos(&mut self, activo: bool) -> Result<(), String> {
        self.visor.as_mut().ok_or("falta visor")?.activar_campos(activo)
    }
    /// Carga explícita para ensayos/reanudación; espera y copia, nunca por fotograma.
    /// # Errors
    /// Rechaza tamaño distinto al estado o errores Vulkan.
    pub fn cargar_campos(&self, datos: &[u32]) -> Result<(), String> {
        self.visor.as_ref().ok_or("falta visor")?.transferir_campos(&self.contexto, Some(datos)).map(|_| ())
    }
    /// Descarga de diagnóstico bajo petición; no gobierna la simulación.
    /// # Errors
    /// Rechaza ausencia de campos o errores Vulkan.
    pub fn inspeccionar_campos(&self) -> Result<Vec<u32>, String> {
        self.visor.as_ref().ok_or("falta visor")?.transferir_campos(&self.contexto, None)
    }
    /// Descarga explícita de un rango de palabras; permite leer solo el resumen del solver.
    /// # Errors
    /// Rechaza rango vacío/fuera del estado o errores Vulkan.
    pub fn inspeccionar_rango_campos(&self, desde: u32, cantidad: u32) -> Result<Vec<u32>, String> {
        self.visor.as_ref().ok_or("falta visor")?.leer_campos(&self.contexto, desde, cantidad)
    }
    /// Captura diagnóstica explícita. Pausar antes si se desea conservar el estado exacto.
    /// # Errors
    /// Rechaza ausencia de estado, constantes o destinos inválidos.
    pub fn capturar_campos_interfaz(
        &mut self,
        tamano: (u32, u32),
        fondo: [f32; 4],
        bloque: &[f32; 32],
        capa: crate::CapaInterfaz,
    ) -> Result<crate::CapturaEstudio, String> {
        if !bloque.iter().chain(&fondo).all(|v| v.is_finite()) {
            return Err("constantes no finitas".into());
        }
        self.visor.as_mut().ok_or("falta visor")?.capturar_campos(&self.contexto, tamano, fondo, bloque, capa)
    }
}
