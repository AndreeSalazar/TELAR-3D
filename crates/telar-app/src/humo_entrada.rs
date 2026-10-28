//! Interacción compartida; la cámara solo afecta al visor del laboratorio 3D.
use telar_ui::{DisenoObjetos, EditorHumo};
pub(crate) struct Entrada {
    raton: (i32, i32),
    orbitando: bool,
    camara: nudo::Camara,
}
impl Default for Entrada {
    fn default() -> Self {
        Self {
            raton: (0, 0),
            orbitando: false,
            camara: nudo::Camara {
                objetivo: [0.5, 0.48, 0.5],
                distancia: 1.9,
                giro: 32.0,
                inclinacion: 12.0,
                ..Default::default()
            },
        }
    }
}
impl Entrada {
    #[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
    pub(crate) fn mover(&mut self, editor: &mut EditorHumo, d: DisenoObjetos, x: i32, y: i32) {
        if self.orbitando {
            self.camara.orbitar(-(x - self.raton.0) as f32 * 0.3, (y - self.raton.1) as f32 * 0.3);
        } else {
            editor.mover(d, x as f32);
        }
        self.raton = (x, y);
    }
    #[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
    pub(crate) fn pulsar(&mut self, editor: &mut EditorHumo, d: DisenoObjetos) {
        let (x, y) = (self.raton.0 as f32, self.raton.1 as f32);
        self.orbitando = editor.volumen && !editor.editando() && d.visor.contiene(x, y);
        if !self.orbitando {
            editor.pulsar(d, x, y);
        }
    }
    pub(crate) fn soltar(&mut self, editor: &mut EditorHumo) {
        self.orbitando = false;
        editor.soltar();
    }
    #[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
    pub(crate) fn rueda(&mut self, editor: &EditorHumo, d: DisenoObjetos, pasos: f32) {
        if editor.volumen && !editor.editando() && d.visor.contiene(self.raton.0 as f32, self.raton.1 as f32) {
            self.camara.acercar(0.88_f32.powf(pasos));
            self.camara.distancia = self.camara.distancia.clamp(1.2, 5.0);
        }
    }
    pub(crate) fn bloque(&self, editor: &EditorHumo, srgb: bool) -> Result<[f32; 32], String> {
        if editor.volumen {
            crate::volumen_vista::bloque(editor, &self.camara, srgb)
        } else {
            Ok(crate::humo_vista::bloque(editor, srgb))
        }
    }
}
