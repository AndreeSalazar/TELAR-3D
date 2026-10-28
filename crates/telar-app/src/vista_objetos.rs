//! Recursos y cambios del visor, independientes del trabajo final asíncrono.
use telar_gpu::Gpu;
use telar_ui::{DisenoObjetos, EditorObjetos, Fuente, PALABRAS_OBJETOS};
pub(crate) struct Vista {
    aplicada: [u32; PALABRAS_OBJETOS],
    tamano: (u32, u32),
    formato: u32,
    ui: Vec<[f32; 8]>,
    ui_tamano: (u32, u32),
}
impl Vista {
    pub(crate) fn nueva(gpu: &Gpu, editor: &EditorObjetos) -> Self {
        Self {
            aplicada: editor.escena.guardar(),
            tamano: gpu.imagenes().1,
            formato: gpu.imagenes().0,
            ui: Vec::new(),
            ui_tamano: (0, 0),
        }
    }
    pub(crate) fn dibujar(
        &mut self,
        gpu: &mut Gpu,
        editor: &EditorObjetos,
        diseno: DisenoObjetos,
        fuente: &Fuente,
        ocupado: bool,
        sucio: bool,
    ) -> Result<bool, String> {
        let estado = editor.escena.guardar();
        let actual = gpu.imagenes();
        if !ocupado && (estado[..40] != self.aplicada[..40] || self.tamano != actual.1 || self.formato != actual.0) {
            crate::render_objetos::preparar(gpu, &editor.escena, diseno.visor.ancho / diseno.visor.alto)?;
            self.aplicada = estado;
            self.tamano = actual.1;
            self.formato = actual.0;
        }
        if sucio || self.ui_tamano != actual.1 {
            let nueva = editor.dibujar(diseno, fuente)?;
            if nueva != self.ui || self.ui_tamano != actual.1 {
                gpu.actualizar_interfaz(&nueva, diseno.area())?;
                self.ui = nueva;
            }
            self.ui_tamano = actual.1;
        }
        gpu.dibujar_bloque_interfaz([0.04, 0.05, 0.07, 1.0], &[0.0; 32])
    }
}
