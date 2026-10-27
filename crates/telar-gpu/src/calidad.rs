//! Selección de calidad sin conocer interfaz ni materiales.
use crate::{Calculo, Gpu};

pub(crate) fn validar_muestras(muestras: u32) -> Result<(), String> {
    if !matches!(muestras, 1 | 2 | 4 | 8 | 16 | 32 | 64) {
        return Err("MSAA requiere 1, 2, 4, 8, 16, 32 o 64 muestras".to_owned());
    }
    Ok(())
}

impl Calculo {
    /// Muestras efectivas para las capturas RGBA8, según el máximo solicitado.
    /// # Errors
    /// Si falla la consulta de capacidades Vulkan.
    pub fn muestras_de_captura(&self) -> Result<u32, String> {
        self.contexto.muestras(crate::FORMATO_CAPTURA)
    }
}

impl Gpu {
    /// Cambia el máximo de muestras en vivo y devuelve la cantidad admitida.
    /// Recrea adjuntos gráficos solo si cambia la cantidad efectiva; conserva compute.
    ///
    /// # Errors
    /// Rechaza valores distintos de potencias de dos entre 1 y 64 o errores Vulkan.
    pub fn configurar_muestras(&mut self, muestras: u32) -> Result<u32, String> {
        validar_muestras(muestras)?;
        let formato = self.presentacion.formato;
        let anterior = self.contexto.muestras(formato)?;
        let estado = (self.contexto.antialias, self.contexto.muestras_solicitadas);
        self.contexto.antialias = true;
        self.contexto.muestras_solicitadas = muestras;
        let cambiar = (|| {
            let actual = self.contexto.muestras(formato)?;
            if actual != anterior {
                self.presentacion.actualizar_muestreo(&self.contexto)?;
            }
            Ok(actual)
        })();
        if cambiar.is_err() {
            (self.contexto.antialias, self.contexto.muestras_solicitadas) = estado;
        }
        cambiar
    }
}
