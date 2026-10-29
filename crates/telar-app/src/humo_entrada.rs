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
    pub(crate) fn encuadrar_vfx(&mut self) {
        self.camara.objetivo = [0.5, 0.35, 0.5];
        self.camara.distancia = 1.3;
    }
    pub(crate) fn camara_guardable(&self) -> [f32; 8] {
        let c = &self.camara;
        [c.objetivo[0], c.objetivo[1], c.objetivo[2], c.distancia, c.giro, c.inclinacion, c.campo, c.cerca]
    }
    pub(crate) fn restaurar_camara(&mut self, c: [f32; 8]) {
        self.camara = nudo::Camara {
            objetivo: [c[0], c[1], c[2]],
            distancia: c[3],
            giro: c[4],
            inclinacion: c[5],
            campo: c[6],
            cerca: c[7],
        };
    }
    #[allow(clippy::cast_precision_loss, reason = "píxeles de ratón")]
    pub(crate) fn mover(&mut self, editor: &mut EditorHumo, d: DisenoObjetos, x: i32, y: i32) -> bool {
        let propiedades = !self.orbitando && editor.arrastrando_control();
        if self.orbitando {
            self.camara.orbitar(-(x - self.raton.0) as f32 * 0.3, (y - self.raton.1) as f32 * 0.3);
        } else {
            editor.mover(d, x as f32);
        }
        self.raton = (x, y);
        propiedades
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
        if editor.particulas.is_some() {
            return crate::particulas_vista::bloque(editor, &self.camara, srgb);
        }
        if editor.volumen {
            crate::volumen_vista::bloque(editor, &self.camara, srgb)
        } else {
            Ok(crate::humo_vista::bloque(editor, srgb))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(clippy::cast_possible_truncation, reason = "coordenadas del ensayo acotadas a 1280x800")]
    fn solo_el_deslizador_invalida_el_overlay_al_mover() -> Result<(), String> {
        let mut editor = telar_ui::EditorHumo::default();
        editor.volumen = true;
        let d = crate::humo_vista::diseno_editor(&editor, (1280, 800));
        let mut entrada = super::Entrada::default();
        assert!(!entrada.mover(&mut editor, d, 10, 10));
        let x = (d.visor.x + d.visor.ancho * 0.5) as i32;
        let y = (d.visor.y + d.visor.alto * 0.5) as i32;
        assert!(!entrada.mover(&mut editor, d, x, y));
        let antes = entrada.bloque(&editor, true)?;
        entrada.pulsar(&mut editor, d);
        assert!(!entrada.mover(&mut editor, d, x + 20, y + 10));
        assert_ne!(antes, entrada.bloque(&editor, true)?);
        entrada.soltar(&mut editor);
        let rect = d.control(0);
        let x = (rect.x + 20.0) as i32;
        let y = (rect.y + 40.0) as i32;
        assert!(!entrada.mover(&mut editor, d, x, y));
        entrada.pulsar(&mut editor, d);
        assert!(editor.arrastrando_control());
        assert!(entrada.mover(&mut editor, d, x + 30, y));
        entrada.soltar(&mut editor);
        assert!(!entrada.mover(&mut editor, d, x + 40, y));
        Ok(())
    }
}
