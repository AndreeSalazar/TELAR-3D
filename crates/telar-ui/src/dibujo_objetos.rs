//! Presentación CSS del editor de objetos; no accede a recursos de render.
use crate::{DisenoObjetos, EditorObjetos, Fuente, Rectangulo, lienzo::Lienzo};
use alloc::{format, vec::Vec};
use telar_estilo::{Estado, Estilo};

impl EditorObjetos {
    fn estilo(&self, clase: &str, activo: bool) -> Estilo {
        let raiz = self.hoja.resolver("estudio", Estado::default(), None);
        self.hoja.resolver(clase, Estado::nuevo(false, activo), Some(&raiz))
    }
    /// Triángulos UI solo cuando cambian datos o tamaño. El visor queda descubierto.
    /// # Errors
    /// Rechaza un atlas de fuente inválido.
    #[allow(clippy::cast_precision_loss, reason = "ventana y seis controles acotados")]
    pub fn dibujar(&self, diseno: DisenoObjetos, fuente: &Fuente) -> Result<Vec<[f32; 8]>, &'static str> {
        if !fuente.valida() {
            return Err("fuente del editor inválida");
        }
        let mut lienzo = Lienzo::nuevo(diseno.tamano, Some(fuente));
        let base = self.estilo("estudio", false);
        let etiqueta = self.estilo("etiqueta", false);
        let titulo = self.estilo("titulo", false);
        let panel = self.estilo("panel", false);
        let (ancho, alto) = (diseno.tamano.0 as f32, diseno.tamano.1 as f32);
        if !diseno.completo {
            lienzo.rect(0.0, 0.0, ancho, alto, base.fondo);
            texto(&mut lienzo, [16.0, 20.0, ancho - 32.0, 32.0], "Amplía la ventana: mínimo 760 × 650", &titulo);
            return Ok(lienzo.vertices);
        }
        let visor = diseno.visor;
        for rect in [
            Rectangulo { x: 0.0, y: 0.0, ancho, alto: visor.y },
            Rectangulo { x: 0.0, y: visor.y, ancho: visor.x, alto: visor.alto },
            Rectangulo { x: visor.x + visor.ancho, y: visor.y, ancho: ancho - visor.x - visor.ancho, alto: visor.alto },
            Rectangulo { x: 0.0, y: visor.y + visor.alto, ancho, alto: alto - visor.y - visor.alto },
        ] {
            lienzo.rect(rect.x, rect.y, rect.ancho, rect.alto, base.fondo);
        }
        texto(&mut lienzo, [20.0, 18.0, 145.0, 32.0], "TELAR · Escena", &titulo);
        for (i, nombre) in ["Guardar · F5", "Abrir · F9", "Render · F6"].iter().enumerate() {
            let boton = self.estilo("boton", false);
            let rect = DisenoObjetos::accion(i);
            lienzo.caja(rect, &boton);
            lienzo.etiqueta(rect.interior(6.0), nombre, &boton);
        }
        texto(
            &mut lienzo,
            [548.0, 22.0, (ancho - 568.0).max(1.0), 30.0],
            "Vista GPU · 1 spp · órbita / rueda",
            &etiqueta,
        );
        lienzo.caja(diseno.panel, &panel);
        texto(&mut lienzo, [diseno.panel.x + 16.0, 80.0, 288.0, 24.0], "Elementos de escena", &titulo);
        for (i, nombre) in ["Caja naranja", "Caja azul", "Luz de área", "Salida"].iter().enumerate() {
            let boton = self.estilo("boton", self.escena.seleccionado == i);
            let rect = diseno.entidad(i);
            lienzo.caja(rect, &boton);
            lienzo.etiqueta(rect.interior(6.0), nombre, &boton);
        }
        let paginas = match self.escena.seleccionado {
            0 | 1 => ["Transformación", "Color / visibilidad"],
            2 => ["Luz / sombras", "Color de luz"],
            _ => ["Render final", "Cámara"],
        };
        for (i, nombre) in paginas.iter().enumerate() {
            let estilo = self.estilo("boton", self.escena.pagina == i);
            let rect = diseno.pagina(i);
            lienzo.caja(rect, &estilo);
            lienzo.etiqueta(rect.interior(4.0), nombre, &estilo);
        }
        for i in 0..6 {
            let Some(control) = self.escena.control(i) else { continue };
            let rect = diseno.control(i);
            let valor_rect = diseno.valor(i);
            texto(&mut lienzo, [rect.x, rect.y, rect.ancho - 96.0, 26.0], control.nombre, &etiqueta);
            let edicion = self.edicion.as_ref().filter(|(indice, _)| *indice == i);
            let estilo = self.estilo("valor", edicion.is_some());
            lienzo.caja(valor_rect, &estilo);
            let valor = edicion.map_or_else(|| format!("{}", control.valor), |(_, v)| format!("{v}|"));
            lienzo.etiqueta(valor_rect.interior(4.0), &valor, &estilo);
            let fraccion = ((control.valor - control.minimo) / (control.maximo - control.minimo)).clamp(0.0, 1.0);
            let barra = self.estilo("control", false);
            let y = rect.y + 40.0;
            lienzo.rect(rect.x, y, rect.ancho, 3.0, barra.fondo);
            lienzo.rect(rect.x, y, rect.ancho * fraccion, 3.0, barra.color);
            lienzo.redondo(
                Rectangulo { x: rect.x + rect.ancho * fraccion - 4.0, y: y - 3.0, ancho: 9.0, alto: 9.0 },
                4.5,
                barra.color,
            );
        }
        let estado = if self.edicion.is_some() {
            format!("{} · Enter: aplicar · Esc: cancelar", self.mensaje)
        } else {
            self.mensaje.clone()
        };
        texto(&mut lienzo, [16.0, alto - 25.0, ancho - 32.0, 22.0], &estado, &etiqueta);
        Ok(lienzo.vertices)
    }
}
fn texto(lienzo: &mut Lienzo, rect: [f32; 4], texto: &str, estilo: &Estilo) {
    lienzo.etiqueta(Rectangulo { x: rect[0], y: rect[1], ancho: rect[2], alto: rect[3] }, texto, estilo);
}
