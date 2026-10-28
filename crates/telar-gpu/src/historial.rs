//! API de historial residente, independiente de cámara y modelos de iluminación.
use crate::{Gpu, Sombreadores};
impl Gpu {
    /// Historial de cuatro palabras por píxel, set 0/binding 1; compute local X=64.
    /// Inicializa a cero y añade compute antes del dibujo, con barreras entre cuadros.
    /// El shader decide cuándo detenerse. Constantes gráficas [2,3] = ancho, alto.
    /// # Errors
    /// Rechaza dimensiones fuera de 1..1024 × 1..768 o recursos/shaders inválidos.
    pub fn preparar_historial(
        &mut self,
        shaders: Sombreadores,
        compute: &[u32],
        tamano: [u32; 2],
    ) -> Result<(), String> {
        if self.visor.is_none() {
            self.preparar_visor(shaders)?;
        }
        self.visor.as_mut().ok_or("falta visor")?.preparar_historial(
            &self.contexto,
            (shaders.vertices, shaders.fragmentos),
            compute,
            tamano,
        )
    }
    /// Descarga diagnóstica explícita: espera a la GPU; nunca en el bucle vivo.
    /// # Errors
    /// Rechaza historial ausente o errores del backend.
    pub fn inspeccionar_historial(&self) -> Result<Vec<[u32; 4]>, String> {
        self.visor.as_ref().ok_or("falta visor")?.inspeccionar_historial(&self.contexto)
    }
    /// Captura diagnóstica de visor/overlay; espera y descarga RGBA8 explícitamente.
    /// Ejecuta un paso adicional si el kernel aún no llegó al límite de muestras.
    /// # Errors
    /// Rechaza constantes/área inválidas, historial ausente o errores Vulkan.
    pub fn capturar_historial_interfaz(
        &mut self,
        tamano: (u32, u32),
        fondo: [f32; 4],
        bloque: &[f32; 32],
        capa: crate::CapaInterfaz,
    ) -> Result<crate::CapturaEstudio, String> {
        if !bloque.iter().chain(&fondo).all(|v| v.is_finite()) {
            return Err("constantes no finitas".into());
        }
        self.visor.as_mut().ok_or("falta visor")?.capturar_historial(&self.contexto, tamano, fondo, bloque, capa)
    }
}
