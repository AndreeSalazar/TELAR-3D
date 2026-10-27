//! Composición semántica; colores, cajas y tipografía salen de la hoja CSS.
use crate::{Distribucion, Estudio, Objetivo, Rectangulo, lienzo::Lienzo};
use alloc::{format, vec::Vec};
use telar_estilo::{Estado, Estilo};

impl Estudio {
    pub(crate) fn estilo(&self, clase: &str, hover: bool, seleccionado: bool) -> Estilo {
        let raiz = self.hoja.resolver("estudio", Estado::default(), None);
        self.hoja.resolver(clase, Estado::nuevo(hover, seleccionado), Some(&raiz))
    }

    /// Compone triángulos al cambiar estado o tamaño; no modifica la malla 3D.
    #[must_use]
    pub fn dibujar(&self, diseno: Distribucion, puntos: u32) -> Vec<[f32; 8]> {
        self.componer(diseno, puntos, None)
    }

    /// Compone con tipografía proporcional y cobertura del atlas.
    ///
    /// # Errors
    /// Si el atlas contiene dimensiones o métricas inválidas.
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

    #[allow(clippy::cast_precision_loss, reason = "dimensiones de ventana")]
    fn componer(&self, diseno: Distribucion, puntos: u32, fuente: Option<&crate::Fuente>) -> Vec<[f32; 8]> {
        let mut lienzo = Lienzo::nuevo(diseno.tamano, fuente);
        if !diseno.completa {
            return lienzo.vertices;
        }
        let base = self.estilo("estudio", false, false);
        let cabecera = self.estilo("cabecera", false, false);
        let pie = self.estilo("pie", false, false);
        let (ancho, alto) = (diseno.tamano.0 as f32, diseno.tamano.1 as f32);
        // Los fondos rodean el viewport: nunca cubren la escena ya dibujada en GPU.
        lienzo.rect(0.0, 0.0, ancho, diseno.cabecera + diseno.margen, base.fondo);
        lienzo.rect(0.0, diseno.cabecera, diseno.margen, alto - diseno.cabecera, base.fondo);
        lienzo.rect(
            diseno.escena.x + diseno.escena.ancho,
            diseno.cabecera,
            diseno.espacio,
            alto - diseno.cabecera,
            base.fondo,
        );
        lienzo.rect(ancho - diseno.margen, diseno.cabecera, diseno.margen, alto - diseno.cabecera, base.fondo);
        lienzo.rect(0.0, alto - diseno.pie - diseno.margen, ancho, diseno.pie + diseno.margen, base.fondo);
        lienzo.rect(diseno.red.x, diseno.ajustes.y + diseno.ajustes.alto, diseno.red.ancho, diseno.espacio, base.fondo);
        lienzo.caja(
            Rectangulo { x: 0.0, y: -cabecera.grosor, ancho, alto: diseno.cabecera + cabecera.grosor },
            &cabecera,
        );
        let acento = self.estilo("acento", false, false);
        nudo(&mut lienzo, 22.0, 20.0, acento.color);
        texto(&mut lienzo, [48.0, 13.0, 100.0, 28.0], "TELAR", &self.estilo("titulo", false, false));
        texto(&mut lienzo, [152.0, 17.0, 80.0, 22.0], "Estudio", &base);
        lienzo.rect(152.0, diseno.cabecera - 3.0, 46.0, 2.0, acento.color);
        texto(
            &mut lienzo,
            [252.0, 17.0, ancho - 440.0, 22.0],
            "Superficie sin título",
            &self.estilo("etiqueta", false, false),
        );
        texto(&mut lienzo, [ancho - 142.0, 17.0, 126.0, 22.0], "GPU conectada", &acento);
        marco(self, &mut lienzo, diseno.escena, "Escena", Some("Perspectiva · Sombreado"));
        inspector(self, &mut lienzo, diseno);
        grafo(self, &mut lienzo, diseno);
        lienzo.rect(0.0, alto - diseno.pie, ancho, diseno.pie, pie.fondo);
        texto(
            &mut lienzo,
            [20.0, alto - diseno.pie, 100.0, diseno.pie],
            if self.pausado { "Pausado" } else { "En vivo" },
            &acento,
        );
        texto(
            &mut lienzo,
            [120.0, alto - diseno.pie, diseno.escena.ancho - 120.0, diseno.pie],
            "Arrastrar: órbita    Rueda: zoom",
            &pie,
        );
        texto(
            &mut lienzo,
            [diseno.red.x, alto - diseno.pie, diseno.red.ancho, diseno.pie],
            &format!("{puntos} puntos · 4 etapas"),
            &pie,
        );
        lienzo.vertices
    }
}

fn texto(lienzo: &mut Lienzo, rect: [f32; 4], texto: &str, estilo: &Estilo) {
    lienzo.etiqueta(Rectangulo { x: rect[0], y: rect[1], ancho: rect[2], alto: rect[3] }, texto, estilo);
}

fn marco(estudio: &Estudio, lienzo: &mut Lienzo, rect: Rectangulo, titulo: &str, detalle: Option<&str>) {
    let mut panel = estudio.estilo(if detalle.is_some() { "visor" } else { "panel" }, false, false);
    if detalle.is_some() {
        panel.grosor = panel.grosor.clamp(0.0, 4.0);
    }
    let barra = estudio.estilo("barra", false, false);
    let alto = barra.alto.unwrap_or(36.0).clamp(24.0, 40.0);
    // Borde sin relleno: el panel de escena contiene píxeles de Vulkan.
    let interior = rect.interior(panel.grosor);
    lienzo.redondo(Rectangulo { alto: alto - panel.grosor, ..interior }, panel.radio, barra.fondo);
    lienzo.rect(
        interior.x,
        rect.y + panel.radio + panel.grosor,
        interior.ancho,
        (alto - panel.radio - panel.grosor).max(0.0),
        barra.fondo,
    );
    if detalle.is_some() {
        lienzo.rect(rect.x, rect.y, rect.ancho, panel.grosor, panel.borde);
        lienzo.rect(rect.x, rect.y + rect.alto - panel.grosor, rect.ancho, panel.grosor, panel.borde);
        lienzo.rect(rect.x, rect.y, panel.grosor, rect.alto, panel.borde);
        lienzo.rect(rect.x + rect.ancho - panel.grosor, rect.y, panel.grosor, rect.alto, panel.borde);
    }
    lienzo.rect(rect.x, rect.y + alto - 1.0, rect.ancho, 1.0, panel.borde);
    texto(lienzo, [rect.x + barra.relleno, rect.y, rect.ancho - barra.relleno * 2.0, alto], titulo, &barra);
    if let Some(detalle) = detalle {
        texto(lienzo, [rect.x + rect.ancho - 190.0, rect.y, 178.0, alto], detalle, &barra);
    }
}

fn inspector(estudio: &Estudio, lienzo: &mut Lienzo, diseno: Distribucion) {
    let panel = estudio.estilo("panel", false, false);
    let etiqueta = estudio.estilo("etiqueta", false, false);
    lienzo.caja(diseno.ajustes, &panel);
    marco(estudio, lienzo, diseno.ajustes, "Propiedades", None);
    let contenido = diseno.ajustes.contenido(&panel);
    let titulo = ["Rejilla", "Onda", "Relieve", "Salida"][estudio.seleccionado.min(3)];
    texto(
        lienzo,
        [contenido.x, diseno.ajustes.y + 44.0, contenido.ancho, 28.0],
        titulo,
        &estudio.estilo("titulo", false, false),
    );
    texto(
        lienzo,
        [contenido.x, diseno.ajustes.y + 74.0, contenido.ancho, 18.0],
        "Parámetros globales de superficie",
        &etiqueta,
    );
    for (indice, (nombre, valor, maximo)) in
        [("Amplitud", estudio.amplitud, 0.8), ("Velocidad", estudio.velocidad, 2.0)].into_iter().enumerate()
    {
        let rect = diseno.control(indice);
        let control = estudio.estilo("control", estudio.encima == Some(Objetivo::Control(indice)), false);
        let campo = estudio.estilo("valor", false, false);
        let ancho_valor = campo.ancho.unwrap_or(64.0).min(rect.ancho * 0.3);
        let alto_valor = campo.alto.unwrap_or(26.0).min(28.0);
        let valor_rect =
            Rectangulo { x: rect.x + rect.ancho - ancho_valor, y: rect.y - 28.0, ancho: ancho_valor, alto: alto_valor };
        texto(lienzo, [rect.x, rect.y - 26.0, rect.ancho - ancho_valor - 8.0, 22.0], nombre, &etiqueta);
        lienzo.caja(valor_rect, &campo);
        lienzo.etiqueta(valor_rect.interior(8.0), &format!("{valor:.2}"), &campo);
        let y = rect.y + rect.alto * 0.5;
        let barra = Rectangulo { x: rect.x, y, ancho: rect.ancho, alto: 3.0 };
        lienzo.redondo(barra, control.radio, control.fondo);
        lienzo.redondo(Rectangulo { ancho: rect.ancho * valor / maximo, ..barra }, control.radio, control.color);
        lienzo.redondo(
            Rectangulo { x: rect.x + rect.ancho * valor / maximo - 4.0, y: y - 3.0, ancho: 9.0, alto: 9.0 },
            4.5,
            control.color,
        );
    }
    let boton = diseno.pausa();
    let estilo = estudio.estilo("boton", estudio.encima == Some(Objetivo::Pausa), estudio.pausado);
    lienzo.caja(boton, &estilo);
    lienzo.etiqueta(boton.contenido(&estilo), if estudio.pausado { "Continuar" } else { "Pausar" }, &estilo);
    let primera = diseno.muestra(0);
    texto(
        lienzo,
        [primera.x, primera.y - 23.0, diseno.ajustes.x + diseno.ajustes.ancho - primera.x - panel.relleno, 20.0],
        "Color de vista previa",
        &etiqueta,
    );
    for material in 0..3 {
        let mut estilo = estudio.estilo(
            "muestra",
            estudio.encima == Some(Objetivo::Material(material)),
            estudio.material == material,
        );
        let color = crate::color_material(material);
        estilo.fondo = [color[0], color[1], color[2]];
        lienzo.caja(diseno.muestra(material), &estilo);
    }
}

fn grafo(estudio: &Estudio, lienzo: &mut Lienzo, diseno: Distribucion) {
    let panel = estudio.estilo("panel", false, false);
    let puntos = estudio.estilo("puntos", false, false);
    lienzo.caja(diseno.red, &panel);
    marco(estudio, lienzo, diseno.red, "Red procedural", None);
    let area = diseno.grafo;
    let mut y = area.y;
    while y < area.y + area.alto {
        let mut x = area.x;
        while x < area.x + area.ancho {
            lienzo.rect(x, y, 1.0, 1.0, puntos.color);
            x += puntos.espacio.max(8.0);
        }
        y += puntos.espacio.max(8.0);
    }
    let hilo = estudio.estilo("hilo", false, false);
    for indice in 0..3 {
        let desde = estudio.nodo(diseno, indice);
        let hasta = estudio.nodo(diseno, indice + 1);
        lienzo.hilo(
            [desde.x + desde.ancho * 0.5, desde.y + desde.alto],
            [hasta.x + hasta.ancho * 0.5, hasta.y],
            hilo.color,
            hilo.grosor,
        );
    }
    let seleccionado = estudio.seleccionado.min(3);
    for indice in (0..4).filter(|i| *i != seleccionado).chain(core::iter::once(seleccionado)) {
        nodo(estudio, lienzo, diseno, indice);
    }
}

fn nodo(estudio: &Estudio, lienzo: &mut Lienzo, diseno: Distribucion, indice: usize) {
    let (titulo, detalle) =
        [("Rejilla", "Geometría"), ("Onda", "Deformación"), ("Relieve", "HILO / IR"), ("Salida", "Superficie")][indice];
    let rect = estudio.nodo(diseno, indice);
    let estilo = estudio.estilo("nodo", estudio.encima == Some(Objetivo::Nodo(indice)), estudio.seleccionado == indice);
    lienzo.caja(rect, &estilo);
    let contenido = rect.contenido(&estilo);
    let acento = estudio.estilo("acento", false, false);
    nudo(lienzo, contenido.x, contenido.y + 2.0, acento.color);
    texto(lienzo, [contenido.x + 23.0, rect.y + 7.0, contenido.ancho - 23.0, estilo.linea], titulo, &estilo);
    texto(
        lienzo,
        [contenido.x + 23.0, rect.y + 7.0 + estilo.linea + estilo.espacio, contenido.ancho - 23.0, 16.0],
        detalle,
        &estudio.estilo("etiqueta", false, false),
    );
    let mut puerto = estudio.estilo("puerto", false, false);
    if indice == 3 {
        puerto.fondo = estudio.estilo("salida", false, false).fondo;
    }
    let lado = puerto.ancho.unwrap_or(7.0).clamp(4.0, 10.0);
    for (entrada, altura) in [(true, rect.y), (false, rect.y + rect.alto)] {
        // Rejilla solo produce y salida solo consume.
        if (indice == 0 && entrada) || (indice == 3 && !entrada) {
            continue;
        }
        lienzo.caja(
            Rectangulo { x: rect.x + rect.ancho * 0.5 - lado * 0.5, y: altura - lado * 0.5, ancho: lado, alto: lado },
            &puerto,
        );
    }
}

fn nudo(lienzo: &mut Lienzo, x: f32, y: f32, color: [f32; 3]) {
    lienzo.linea([x, y + 4.0], [x + 8.0, y + 12.0], 2.0, color);
    lienzo.linea([x + 4.0, y], [x + 12.0, y + 8.0], 2.0, color);
    lienzo.linea([x, y + 8.0], [x + 8.0, y], 2.0, color);
    lienzo.linea([x + 4.0, y + 12.0], [x + 12.0, y + 4.0], 2.0, color);
}
