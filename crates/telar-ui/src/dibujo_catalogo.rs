//! Presentación del centro de herramientas con CSS nativo y capacidades explícitas.
use crate::{CATEGORIAS, Catalogo, DisenoCatalogo, Fuente, Rectangulo, lienzo::Lienzo};
use alloc::{format, vec::Vec};
use telar_estilo::{Estado, Estilo};
impl Catalogo {
    fn estilo(&self, clase: &str, seleccionado: bool) -> Estilo {
        let base = self.hoja.resolver("estudio", Estado::default(), None);
        self.hoja.resolver(clase, Estado::nuevo(false, seleccionado), Some(&base))
    }
    /// Composición de tarjetas y detalles; no dibuja ni simula las herramientas.
    /// # Errors
    /// Rechaza fuente inválida antes de emitir coordenadas de atlas.
    #[allow(clippy::cast_precision_loss, reason = "dimensiones físicas y ocho categorías")]
    pub fn dibujar(&self, d: DisenoCatalogo, fuente: &Fuente) -> Result<Vec<[f32; 8]>, &'static str> {
        if !fuente.valida() {
            return Err("fuente del centro inválida");
        }
        let mut l = Lienzo::nuevo(d.tamano, Some(fuente));
        let base = self.estilo("estudio", false);
        let titulo = self.estilo("titulo", false);
        let texto = self.estilo("etiqueta", false);
        let panel = self.estilo("panel", false);
        let (w, h) = (d.tamano.0 as f32, d.tamano.1 as f32);
        l.rect(0.0, 0.0, w, h, base.fondo);
        etiqueta(&mut l, [24.0, 18.0, 160.0, 30.0], "TELAR / Estudio", &titulo);
        if !d.completo {
            etiqueta(&mut l, [24.0, 80.0, (w - 48.0).max(1.0), 42.0], "Amplía la ventana: mínimo 1080 × 700", &texto);
            return Ok(l.vertices);
        }
        etiqueta(&mut l, [240.0, 22.0, w - 264.0, 28.0], "Crear · simular · renderizar", &texto);
        let borde = self.estilo("control", false);
        l.rect(16.0, 70.0, w - 32.0, 1.0, borde.fondo);
        l.caja(d.lateral, &panel);
        l.caja(d.detalle, &panel);
        etiqueta(&mut l, [32.0, 102.0, 176.0, 28.0], "ÁREAS DE TRABAJO", &texto);
        for (i, categoria) in CATEGORIAS.iter().enumerate() {
            let estilo = self.estilo("categoria", i == self.indice_categoria());
            let rect = d.categoria(i);
            l.caja(rect, &estilo);
            l.etiqueta(rect.interior(10.0), &format!("{}   {}", i + 1, categoria.nombre), &estilo);
        }
        let contexto = self.categoria();
        etiqueta(&mut l, [d.centro.x, 86.0, d.centro.ancho, 32.0], contexto.nombre, &titulo);
        etiqueta(&mut l, [d.centro.x, 120.0, d.centro.ancho, 24.0], contexto.resumen, &texto);
        let mut nombre = self.estilo("estudio", false);
        nombre.fuente = 16.0;
        for (i, herramienta) in contexto.herramientas.iter().enumerate() {
            let rect = d.herramienta(i);
            let estilo = self.estilo("herramienta", i == self.indice_herramienta());
            l.caja(rect, &estilo);
            let estado = self.estilo(if herramienta.espacio.is_some() { "disponible" } else { "pendiente" }, false);
            etiqueta(&mut l, [rect.x + 16.0, rect.y + 12.0, rect.ancho - 32.0, 22.0], herramienta.estado, &estado);
            etiqueta(&mut l, [rect.x + 16.0, rect.y + 40.0, rect.ancho - 32.0, 26.0], herramienta.nombre, &nombre);
            l.parrafo(
                Rectangulo { x: rect.x + 16.0, y: rect.y + 78.0, ancho: rect.ancho - 32.0, alto: 40.0 },
                herramienta.resumen,
                &texto,
            );
        }
        self.detalle(&mut l, d, &titulo, &texto);
        etiqueta(&mut l, [24.0, h - 30.0, w - 48.0, 24.0], &self.mensaje, &texto);
        Ok(l.vertices)
    }
    fn detalle(&self, l: &mut Lienzo, d: DisenoCatalogo, titulo: &Estilo, texto: &Estilo) {
        let herramienta = self.herramienta();
        let x = d.detalle.x + 16.0;
        etiqueta(l, [x, 102.0, 264.0, 28.0], "HERRAMIENTA", texto);
        l.parrafo(Rectangulo { x, y: 142.0, ancho: 264.0, alto: 64.0 }, herramienta.nombre, titulo);
        let estado = self.estilo(if herramienta.espacio.is_some() { "disponible" } else { "pendiente" }, false);
        etiqueta(l, [x, 214.0, 264.0, 28.0], herramienta.estado, &estado);
        l.parrafo(Rectangulo { x, y: 260.0, ancho: 264.0, alto: 230.0 }, herramienta.detalle, texto);
        let boton = self.estilo(if herramienta.espacio.is_some() { "abrir" } else { "desactivado" }, false);
        let rect = d.abrir();
        l.caja(rect, &boton);
        l.etiqueta(
            rect.interior(10.0),
            if herramienta.espacio.is_some() { "Abrir en otra ventana" } else { "Todavía no disponible" },
            &boton,
        );
        l.parrafo(
            Rectangulo { x, y: rect.y + 54.0, ancho: 264.0, alto: 58.0 },
            "1-8: categoría · flechas: herramienta · Enter: abrir",
            texto,
        );
    }
}
fn etiqueta(l: &mut Lienzo, r: [f32; 4], texto: &str, estilo: &Estilo) {
    l.etiqueta(Rectangulo { x: r[0], y: r[1], ancho: r[2], alto: r[3] }, texto, estilo);
}
