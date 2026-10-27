//! Estudio grafito, cian y cobre. La red representa las cuatro etapas reales.
use crate::lienzo::Lienzo;
use crate::{Distribucion, Estudio};
use alloc::{format, vec::Vec};

const PANEL: [f32; 3] = [0.063, 0.078, 0.098];
const BORDE: [f32; 3] = [0.14, 0.18, 0.22];
const TEXTO: [f32; 3] = [0.86, 0.90, 0.92];
const TENUE: [f32; 3] = [0.49, 0.59, 0.65];
const CIAN: [f32; 3] = [0.23, 0.81, 0.85];

impl Estudio {
    /// Compone la interfaz en triángulos (posición NDC y color sRGB).
    /// Se llama cuando cambia el estado o el tamaño, no por fotograma.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "dimensiones y cuatro nodos de interfaz")]
    pub fn dibujar(&self, diseno: Distribucion, puntos: u32) -> Vec<[f32; 6]> {
        let mut lienzo = Lienzo::nuevo(diseno.tamano);
        if !diseno.completa {
            return lienzo.vertices;
        }
        let (ancho, alto) = (diseno.tamano.0 as f32, diseno.tamano.1 as f32);
        lienzo.rect(0.0, 0.0, ancho, 58.0, PANEL);
        lienzo.rect(0.0, 57.0, ancho, 1.0, BORDE);
        lienzo.rect(20.0, 18.0, 4.0, 22.0, CIAN);
        lienzo.texto(36.0, 21.0, "TELAR", 2.0, TEXTO);
        lienzo.texto(120.0, 25.0, "/ ESTUDIO PROCEDURAL", 1.25, TENUE);
        lienzo.rect(ancho - 206.0, 18.0, 8.0, 8.0, CIAN);
        lienzo.texto(ancho - 187.0, 19.0, "GPU RESIDENTE", 1.25, CIAN);
        lienzo.rect(0.0, 58.0, 208.0, alto - 58.0, PANEL);
        lienzo.rect(207.0, 58.0, 1.0, alto - 58.0, BORDE);
        lienzo.texto(20.0, 84.0, "RED DE OPERACIONES", 1.25, TENUE);
        lienzo.texto(20.0, 108.0, "ESCENA / SUPERFICIE", 1.0, TENUE);
        for (indice, (titulo, detalle)) in
            [("REJILLA", "GENERACION"), ("ONDA", "DEFORMACION"), ("RELIEVE", "HILO / IR"), ("SALIDA", "VISOR")]
                .into_iter()
                .enumerate()
        {
            let y = 140.0 + indice as f32 * 92.0;
            if indice < 3 {
                lienzo.rect(108.0, y + 64.0, 2.0, 28.0, BORDE);
            }
            let activo = indice == self.seleccionado;
            lienzo.rect(20.0, y, 178.0, 64.0, if activo { [0.09, 0.17, 0.20] } else { [0.09, 0.11, 0.14] });
            lienzo.rect(20.0, y, if activo { 3.0 } else { 1.0 }, 64.0, if activo { CIAN } else { BORDE });
            lienzo.texto(34.0, y + 13.0, titulo, 1.5, TEXTO);
            lienzo.texto(34.0, y + 39.0, detalle, 1.0, TENUE);
        }
        lienzo.texto(20.0, alto - 46.0, "RED FIJA / V0", 1.0, TENUE);
        let inicio = diseno.inspector;
        lienzo.rect(inicio, 58.0, ancho - inicio, alto - 58.0, PANEL);
        lienzo.rect(inicio, 58.0, 1.0, alto - 58.0, BORDE);
        lienzo.texto(inicio + 24.0, 84.0, "INSPECTOR", 1.25, TENUE);
        let titulo = ["REJILLA", "ONDA", "RELIEVE", "SALIDA"][self.seleccionado.min(3)];
        lienzo.texto(inicio + 24.0, 120.0, titulo, 2.0, TEXTO);
        lienzo.texto(inicio + 24.0, 153.0, "CONTROLES DE ESCENA", 1.0, TENUE);
        lienzo.rect(inicio + 24.0, 185.0, 210.0, 1.0, BORDE);
        deslizador(&mut lienzo, inicio + 24.0, 248.0, "AMPLITUD", self.amplitud, 0.8);
        deslizador(&mut lienzo, inicio + 24.0, 324.0, "VELOCIDAD", self.velocidad, 2.0);
        lienzo.rect(inicio + 24.0, 373.0, 210.0, 36.0, [0.12, 0.19, 0.22]);
        lienzo.texto(inicio + 48.0, 386.0, if self.pausado { "REANUDAR" } else { "PAUSAR" }, 1.25, CIAN);
        lienzo.texto(inicio + 24.0, 433.0, "MATERIAL / PREVIA", 1.0, TENUE);
        for material in 0..3 {
            let estado = Estudio { material, ..Estudio::default() };
            let color = estado.color();
            let x = inicio + 24.0 + material as f32 * 72.0;
            lienzo.rect(x, 458.0, 62.0, 42.0, [color[0], color[1], color[2]]);
            if self.material == material {
                lienzo.rect(x, 505.0, 62.0, 2.0, CIAN);
            }
        }
        lienzo.texto(inicio + 24.0, alto - 54.0, &format!("CAP. {puntos} PUNTOS"), 1.0, TEXTO);
        lienzo.texto(inicio + 24.0, alto - 30.0, "VISTA PREVIA DIFUSA", 1.0, TENUE);
        let visor = diseno.visor;
        lienzo.texto(visor.x + 14.0, visor.y + 12.0, "PERSPECTIVA / SUPERFICIE", 1.0, TENUE);
        lienzo.rect(218.0, alto - 48.0, visor.ancho, 30.0, PANEL);
        lienzo.texto(232.0, alto - 37.0, "ARRASTRAR: ORBITA   RUEDA: ZOOM", 1.0, TENUE);
        lienzo.vertices
    }
}

fn deslizador(lienzo: &mut Lienzo, x: f32, y: f32, nombre: &str, valor: f32, maximo: f32) {
    lienzo.texto(x, y - 34.0, nombre, 1.25, TENUE);
    lienzo.texto(x + 160.0, y - 34.0, &format!("{valor:.2}"), 1.25, TEXTO);
    lienzo.rect(x, y, 210.0, 3.0, BORDE);
    lienzo.rect(x, y, 210.0 * valor / maximo, 3.0, CIAN);
    lienzo.rect(x + 210.0 * valor / maximo - 3.0, y - 4.0, 6.0, 11.0, TEXTO);
}
