//! Estudio dividido: visor a la izquierda, ajustes arriba y grafo abajo.
//! Nudos y cables conservan identidad TELAR con una red de ejecución fija.
use crate::lienzo::Lienzo;
use crate::{Distribucion, Estudio, Objetivo, Rectangulo};
use alloc::{format, vec::Vec};

const PANEL: [f32; 3] = [0.063, 0.078, 0.098];
const BORDE: [f32; 3] = [0.14, 0.18, 0.22];
const TEXTO: [f32; 3] = [0.86, 0.90, 0.92];
const TENUE: [f32; 3] = [0.49, 0.59, 0.65];
const CIAN: [f32; 3] = [0.23, 0.81, 0.85];
const COBRE: [f32; 3] = [0.94, 0.49, 0.26];

impl Estudio {
    /// Compone la interfaz al cambiar estado o tamaño, sin modificar la malla 3D.
    #[must_use]
    pub fn dibujar(&self, diseno: Distribucion, puntos: u32) -> Vec<[f32; 8]> {
        self.componer(diseno, puntos, None)
    }

    /// Compone con tipografía proporcional y cobertura antialias del atlas.
    ///
    /// # Errors
    /// Si el atlas no contiene dimensiones y métricas válidas.
    pub fn dibujar_con_fuente(
        &self,
        diseno: Distribucion,
        puntos: u32,
        fuente: &crate::Fuente,
    ) -> Result<Vec<[f32; 8]>, &'static str> {
        if !fuente.valida() {
            return Err("atlas tipográfico inválido");
        }
        Ok(self.componer(diseno, puntos, Some(fuente)))
    }

    #[allow(clippy::cast_precision_loss, reason = "dimensiones de la ventana")]
    fn componer(&self, diseno: Distribucion, puntos: u32, fuente: Option<&crate::Fuente>) -> Vec<[f32; 8]> {
        let mut lienzo = Lienzo::nuevo(diseno.tamano, fuente);
        if !diseno.completa {
            return lienzo.vertices;
        }
        let (ancho, alto) = (diseno.tamano.0 as f32, diseno.tamano.1 as f32);
        lienzo.rect(0.0, 0.0, ancho, 52.0, PANEL);
        lienzo.rect(0.0, 51.0, ancho, 1.0, BORDE);
        nudo(&mut lienzo, 22.0, 17.0, CIAN);
        lienzo.texto(48.0, 19.0, "TELAR", 2.0, TEXTO);
        lienzo.texto(140.0, 24.0, "ESTUDIO / 01", 1.0, TENUE);
        lienzo.texto(ancho - 128.0, 24.0, "GPU / EN VIVO", 1.0, CIAN);
        lienzo.texto(24.0, 65.0, "ESCENA / SUPERFICIE", 1.0, TENUE);
        lienzo.rect(diseno.inspector, 52.0, 1.0, alto - 52.0, BORDE);
        lienzo.rect(diseno.inspector + 1.0, 52.0, ancho - diseno.inspector - 1.0, alto - 52.0, PANEL);
        inspector(self, &mut lienzo, diseno);
        grafo(self, &mut lienzo, diseno);
        lienzo.rect(0.0, alto - 44.0, diseno.inspector, 44.0, PANEL);
        lienzo.rect(0.0, alto - 44.0, diseno.inspector, 1.0, BORDE);
        lienzo.texto(24.0, alto - 26.0, if self.pausado { "PAUSADO" } else { "EN VIVO" }, 1.0, CIAN);
        lienzo.texto(108.0, alto - 26.0, "ARRASTRA: ORBITA   RUEDA: ZOOM", 1.0, TENUE);
        lienzo.texto(diseno.inspector + 24.0, alto - 25.0, &format!("CAP. {puntos} PUNTOS"), 1.0, TENUE);
        lienzo.vertices
    }
}

fn inspector(estudio: &Estudio, lienzo: &mut Lienzo, diseno: Distribucion) {
    let x = diseno.inspector + 24.0;
    lienzo.texto(x, 70.0, "AJUSTES / ESCENA", 1.25, TENUE);
    let titulo = ["Rejilla", "Onda", "Relieve", "Salida"][estudio.seleccionado.min(3)];
    lienzo.texto(x, 96.0, titulo, 2.0, TEXTO);
    lienzo.texto(x, 124.0, "PARAMETROS EN VIVO", 1.0, TENUE);
    for (indice, (nombre, valor, maximo)) in
        [("AMPLITUD", estudio.amplitud, 0.8), ("VELOCIDAD", estudio.velocidad, 2.0)].into_iter().enumerate()
    {
        let control = diseno.control(indice);
        let y = control.y + 12.0;
        let encima = estudio.encima == Some(Objetivo::Control(indice));
        lienzo.texto(x, y - 28.0, nombre, 1.25, if encima { CIAN } else { TENUE });
        lienzo.texto(x + control.ancho - 36.0, y - 28.0, &format!("{valor:.2}"), 1.25, TEXTO);
        lienzo.rect(x, y, control.ancho, 2.0, BORDE);
        lienzo.rect(x, y, control.ancho * valor / maximo, 2.0, CIAN);
        lienzo.redondo(
            Rectangulo { x: x + control.ancho * valor / maximo - 4.0, y: y - 3.0, ancho: 8.0, alto: 8.0 },
            4.0,
            if encima { CIAN } else { TEXTO },
        );
    }
    let boton = diseno.pausa();
    let encima = estudio.encima == Some(Objetivo::Pausa);
    lienzo.redondo(boton, 5.0, if encima { [0.15, 0.27, 0.29] } else { [0.11, 0.18, 0.20] });
    lienzo.texto(boton.x + 12.0, boton.y + 10.0, if estudio.pausado { "CONTINUAR" } else { "PAUSAR" }, 1.0, CIAN);
    lienzo.texto(x + 116.0, 266.0, "COLOR / PREVIA", 1.0, TENUE);
    for material in 0..3 {
        let estado = Estudio { material, ..Estudio::default() };
        let color = estado.color();
        let rect = diseno.muestra(material);
        lienzo.redondo(rect, 5.0, [color[0], color[1], color[2]]);
        if estudio.material == material || estudio.encima == Some(Objetivo::Material(material)) {
            lienzo.rect(rect.x + 4.0, rect.y + rect.alto + 5.0, rect.ancho - 8.0, 2.0, CIAN);
        }
    }
}

fn grafo(estudio: &Estudio, lienzo: &mut Lienzo, diseno: Distribucion) {
    let area = diseno.grafo;
    lienzo.rect(diseno.inspector, area.y - 38.0, area.ancho + 32.0, 1.0, BORDE);
    lienzo.texto(area.x + 8.0, area.y - 23.0, "GRAFO / SUPERFICIE", 1.25, TENUE);
    let mut y = area.y + 8.0;
    while y < area.y + area.alto {
        let mut x = area.x + 8.0;
        while x < area.x + area.ancho {
            lienzo.rect(x, y, 1.0, 1.0, [0.12, 0.15, 0.18]);
            x += 24.0;
        }
        y += 24.0;
    }
    for indice in 0..3 {
        let desde = estudio.nodo(diseno, indice);
        let hasta = estudio.nodo(diseno, indice + 1);
        lienzo.hilo(
            [desde.x + desde.ancho * 0.5, desde.y + desde.alto],
            [hasta.x + hasta.ancho * 0.5, hasta.y],
            [0.22, 0.40, 0.43],
        );
    }
    let seleccionado = estudio.seleccionado.min(3);
    for indice in (0..4).filter(|i| *i != seleccionado).chain(core::iter::once(seleccionado)) {
        let (titulo, detalle) =
            [("Rejilla", "Origen"), ("Onda", "Movimiento"), ("Relieve", "HILO / IR"), ("Salida", "Vista")][indice];
        let rect = estudio.nodo(diseno, indice);
        let activo = indice == estudio.seleccionado;
        let encima = estudio.encima == Some(Objetivo::Nodo(indice));
        lienzo.redondo(rect, 9.0, if activo || encima { CIAN } else { BORDE });
        let interior = Rectangulo { x: rect.x + 1.0, y: rect.y + 1.0, ancho: rect.ancho - 2.0, alto: rect.alto - 2.0 };
        lienzo.redondo(interior, 8.0, if activo || encima { [0.09, 0.17, 0.19] } else { [0.085, 0.10, 0.125] });
        nudo(lienzo, rect.x + 12.0, rect.y + 16.0, if activo { CIAN } else { TENUE });
        lienzo.texto(rect.x + 34.0, rect.y + 10.0, titulo, 1.5, TEXTO);
        lienzo.texto(rect.x + 34.0, rect.y + 29.0, detalle, 1.0, TENUE);
        for puerto_y in [rect.y - 3.0, rect.y + rect.alto - 3.0] {
            lienzo.redondo(
                Rectangulo { x: rect.x + rect.ancho * 0.5 - 3.0, y: puerto_y, ancho: 6.0, alto: 6.0 },
                3.0,
                if indice == 3 { COBRE } else { CIAN },
            );
        }
    }
}

fn nudo(lienzo: &mut Lienzo, x: f32, y: f32, color: [f32; 3]) {
    lienzo.linea([x, y + 4.0], [x + 8.0, y + 12.0], 2.0, color);
    lienzo.linea([x + 4.0, y], [x + 12.0, y + 8.0], 2.0, color);
    lienzo.linea([x, y + 8.0], [x + 8.0, y], 2.0, color);
    lienzo.linea([x + 4.0, y + 12.0], [x + 12.0, y + 4.0], 2.0, color);
}
