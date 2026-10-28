//! Recursos y cambios del visor, independientes del trabajo final asíncrono.
use telar_gpu::Gpu;
use telar_ui::{DisenoObjetos, EditorObjetos, Fuente, PALABRAS_OBJETOS};
pub(crate) struct Vista {
    aplicada: [u32; PALABRAS_OBJETOS],
    tamano: (u32, u32),
    formato: u32,
    ui: Vec<[f32; 8]>,
    ui_tamano: (u32, u32),
    muestras: u32,
}
impl Vista {
    pub(crate) fn nueva(gpu: &Gpu, editor: &EditorObjetos) -> Self {
        Self {
            aplicada: editor.escena.guardar(),
            tamano: gpu.imagenes().1,
            formato: gpu.imagenes().0,
            ui: Vec::new(),
            ui_tamano: (0, 0),
            muestras: 0,
        }
    }
    pub(crate) fn dibujar(
        &mut self,
        gpu: &mut Gpu,
        editor: &mut EditorObjetos,
        diseno: DisenoObjetos,
        fuente: &Fuente,
        ocupado: bool,
        mut sucio: bool,
    ) -> Result<bool, String> {
        let estado = editor.escena.guardar();
        let actual = gpu.imagenes();
        if (!ocupado && !editor.arrastrando() && cambia_historial(&estado, &self.aplicada))
            || self.tamano != actual.1
            || self.formato != actual.0
        {
            crate::render_objetos::preparar_progresivo(gpu, &editor.escena, [diseno.area()[2], diseno.area()[3]])?;
            self.muestras = 0;
            self.aplicada = estado;
            self.tamano = actual.1;
            self.formato = actual.0;
        }
        if editor.muestras_vista != self.muestras {
            editor.muestras_vista = self.muestras;
            sucio = true;
        }
        if sucio || self.ui_tamano != actual.1 {
            let nueva = editor.dibujar(diseno, fuente)?;
            if nueva != self.ui || self.ui_tamano != actual.1 {
                gpu.actualizar_interfaz(&nueva, diseno.area())?;
                self.ui = nueva;
            }
            self.ui_tamano = actual.1;
        }
        let mut bloque = [0.0; 32];
        #[allow(clippy::cast_precision_loss, reason = "viewport <=1024×768")]
        let dimensiones = [diseno.area()[2], diseno.area()[3]].map(|v| v as f32);
        bloque[..4].copy_from_slice(&[
            editor.escena.valores()[29],
            f32::from(u8::from(actual.0 != telar_gpu::eleccion::FORMATO_SRGB)),
            dimensiones[0],
            dimensiones[1],
        ]);
        let dibujado = gpu.dibujar_bloque_interfaz([0.04, 0.05, 0.07, 1.0], &bloque)?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "objetivo entero validado <=1024")]
        let limite = f32::from_bits(self.aplicada[30]) as u32;
        if dibujado {
            self.muestras = (self.muestras + 1).min(limite);
        }
        Ok(dibujado)
    }
}

fn cambia_historial(a: &[u32; PALABRAS_OBJETOS], b: &[u32; PALABRAS_OBJETOS]) -> bool {
    a[..29] != b[..29] || a[30..40] != b[30..40]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exposicion_y_seleccion_conservan_historial_y_los_datos_de_escena_lo_invalidan() {
        let base = telar_ui::EscenaObjetos::default().guardar();
        for i in 0..PALABRAS_OBJETOS {
            let mut nueva = base;
            nueva[i] ^= 1;
            assert_eq!(cambia_historial(&nueva, &base), !matches!(i, 29 | 40 | 41));
        }
    }
}
