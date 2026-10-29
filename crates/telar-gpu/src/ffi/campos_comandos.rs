//! Grabación compute compartida por ruta directa y secundarios reutilizables.
use super::{
    ACCESO_INDIRECTO, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, ACCESO_TRANSFER_WRITE, Campos,
    Comandos, ETAPA_CALCULO, ETAPA_INDIRECTO, ETAPA_PIPELINE_FRAGMENTOS, ETAPA_SHADER_CALCULO, ETAPA_TRANSFER,
    FnDispositivo, PUNTO_CALCULO, barrera,
};
impl Campos {
    /// # Safety
    /// Comando grabándose fuera de rendering; recursos viven hasta completar la valla.
    pub(in crate::ffi) unsafe fn grabar(&self, fd: &FnDispositivo, cmd: Comandos, bloque: &[f32; 32]) {
        if !self.activo {
            return;
        }
        // SAFETY: el llamador conserva todos los programas/buffers hasta terminar el envío.
        unsafe {
            for (numero, &indice) in self.secuencia.iter().enumerate() {
                let productor = self.usa_indirectos && matches!(self.rutas[indice], crate::RutaCampos::Productor(_));
                let lectura_indirecta = self.usa_indirectos && (numero == 0 || productor);
                // GPU: protege lectura anterior de fragmentos y escritura de la pasada anterior.
                barrera(
                    fd,
                    cmd,
                    if numero == 0 {
                        ETAPA_CALCULO
                            | ETAPA_PIPELINE_FRAGMENTOS
                            | ETAPA_TRANSFER
                            | if lectura_indirecta { ETAPA_INDIRECTO } else { 0 }
                    } else {
                        ETAPA_CALCULO | if lectura_indirecta { ETAPA_INDIRECTO } else { 0 }
                    },
                    ETAPA_CALCULO | if numero == 0 && self.usa_indirectos { ETAPA_INDIRECTO } else { 0 },
                    if numero == 0 {
                        ACCESO_SHADER_READ
                            | ACCESO_SHADER_WRITE
                            | ACCESO_TRANSFER_READ
                            | ACCESO_TRANSFER_WRITE
                            | if lectura_indirecta { ACCESO_INDIRECTO } else { 0 }
                    } else {
                        ACCESO_SHADER_WRITE | if lectura_indirecta { ACCESO_INDIRECTO } else { 0 }
                    },
                    ACCESO_SHADER_READ
                        | ACCESO_SHADER_WRITE
                        | if numero == 0 && self.usa_indirectos { ACCESO_INDIRECTO } else { 0 },
                );
                let r = &self.programas[indice];
                (fd.calculo.usar_pipeline)(cmd, PUNTO_CALCULO, r.pipeline);
                (fd.calculo.usar_conjuntos)(
                    cmd,
                    PUNTO_CALCULO,
                    r.disposicion_pipeline,
                    0,
                    1,
                    &raw const r.conjunto,
                    0,
                    core::ptr::null(),
                );
                (fd.grafico.empujar_constantes)(
                    cmd,
                    r.disposicion_pipeline,
                    ETAPA_SHADER_CALCULO,
                    0,
                    128,
                    bloque.as_ptr().cast(),
                );
                match self.rutas[indice] {
                    crate::RutaCampos::Indirecto(offset) => {
                        (fd.calculo.despachar_indirecto)(cmd, self.programas[0].buffers[0], offset);
                    }
                    crate::RutaCampos::Directo(grupos) | crate::RutaCampos::Productor(grupos) => {
                        (fd.calculo.despachar)(cmd, grupos, 1, 1);
                    }
                }
                if productor {
                    // GPU: publica argumentos una vez por decisión; consumidores intermedios no los modifican.
                    barrera(fd, cmd, ETAPA_CALCULO, ETAPA_INDIRECTO, ACCESO_SHADER_WRITE, ACCESO_INDIRECTO);
                }
            }
            // GPU: publica todos los campos para el visor posterior y sus lecturas de diagnóstico.
            barrera(fd, cmd, ETAPA_CALCULO, ETAPA_PIPELINE_FRAGMENTOS, ACCESO_SHADER_WRITE, ACCESO_SHADER_READ);
        }
    }
}
