//! Declaraciones tipadas; ningún nombre CSS llega al renderizador.
use crate::{Color, Estilo, Sombra};
use alloc::{format, string::String, vec::Vec};

#[derive(Default)]
pub(super) struct Declaraciones(Vec<Propiedad>);

enum Propiedad {
    Fondo(Color),
    Color(Color),
    Borde(Color),
    Grosor(f32),
    Radio(f32),
    Relleno(f32),
    Margen(f32),
    Espacio(f32),
    Ancho(Option<f32>),
    Alto(Option<f32>),
    Minimo(f32),
    Maximo(f32),
    Fuente(f32),
    Linea(f32),
    Sombra(Option<Sombra>),
}

impl Declaraciones {
    pub(super) fn agregar(&mut self, nombre: &str, valor: &str) -> Result<(), String> {
        let propiedad = match nombre {
            "background-color" => Propiedad::Fondo(color(valor)?),
            "color" => Propiedad::Color(color(valor)?),
            "border-color" => Propiedad::Borde(color(valor)?),
            "border-width" => Propiedad::Grosor(px(valor)?),
            "border-radius" => Propiedad::Radio(px(valor)?),
            "padding" => Propiedad::Relleno(px(valor)?),
            "margin" => Propiedad::Margen(px(valor)?),
            "gap" => Propiedad::Espacio(px(valor)?),
            "width" => Propiedad::Ancho(dimension(valor)?),
            "height" => Propiedad::Alto(dimension(valor)?),
            "min-width" => Propiedad::Minimo(px(valor)?),
            "max-width" => Propiedad::Maximo(px(valor)?),
            "font-size" => Propiedad::Fuente(positivo(valor)?),
            "line-height" => Propiedad::Linea(positivo(valor)?),
            "box-shadow" => Propiedad::Sombra(sombra(valor)?),
            _ => return Err(format!("propiedad CSS no implementada: {nombre}")),
        };
        self.0.push(propiedad);
        Ok(())
    }

    pub(super) fn aplicar(&self, estilo: &mut Estilo) {
        for propiedad in &self.0 {
            match *propiedad {
                Propiedad::Fondo(v) => estilo.fondo = v,
                Propiedad::Color(v) => estilo.color = v,
                Propiedad::Borde(v) => estilo.borde = v,
                Propiedad::Grosor(v) => estilo.grosor = v,
                Propiedad::Radio(v) => estilo.radio = v,
                Propiedad::Relleno(v) => estilo.relleno = v,
                Propiedad::Margen(v) => estilo.margen = v,
                Propiedad::Espacio(v) => estilo.espacio = v,
                Propiedad::Ancho(v) => estilo.ancho = v,
                Propiedad::Alto(v) => estilo.alto = v,
                Propiedad::Minimo(v) => estilo.minimo = v,
                Propiedad::Maximo(v) => estilo.maximo = v,
                Propiedad::Fuente(v) => estilo.fuente = v,
                Propiedad::Linea(v) => estilo.linea = v,
                Propiedad::Sombra(v) => estilo.sombra = v,
            }
        }
    }
}

fn dimension(valor: &str) -> Result<Option<f32>, String> {
    if valor == "auto" { Ok(None) } else { px(valor).map(Some) }
}

fn numero(valor: &str) -> Result<f32, String> {
    let texto = valor
        .strip_suffix("px")
        .or_else(|| (valor == "0").then_some(valor))
        .ok_or_else(|| format!("se esperaba longitud px: {valor}"))?;
    let numero = texto.parse::<f32>().map_err(|_| format!("longitud inválida: {valor}"))?;
    if numero.is_finite() && numero.abs() <= 8192.0 {
        Ok(numero)
    } else {
        Err(format!("longitud fuera de rango: {valor}"))
    }
}

fn px(valor: &str) -> Result<f32, String> {
    let numero = numero(valor)?;
    if numero >= 0.0 { Ok(numero) } else { Err(format!("longitud negativa: {valor}")) }
}

fn positivo(valor: &str) -> Result<f32, String> {
    let numero = px(valor)?;
    if numero > 0.0 { Ok(numero) } else { Err(format!("se esperaba longitud positiva: {valor}")) }
}

fn color(valor: &str) -> Result<Color, String> {
    let hex = valor
        .strip_prefix('#')
        .filter(|v| v.len() == 6 && v.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| format!("se esperaba color #rrggbb: {valor}"))?;
    let numero = u32::from_str_radix(hex, 16).map_err(|_| format!("color inválido: {valor}"))?;
    Ok([16, 8, 0].map(|bits| f32::from(((numero >> bits) & 255) as u8) / 255.0))
}

fn sombra(valor: &str) -> Result<Option<Sombra>, String> {
    if valor == "none" {
        return Ok(None);
    }
    let partes: Vec<_> = valor.split_whitespace().collect();
    if partes.len() != 4 || px(partes[2])? != 0.0 {
        return Err(format!("box-shadow requiere x y 0 color, sin desenfoque: {valor}"));
    }
    Ok(Some(Sombra { x: numero(partes[0])?, y: numero(partes[1])?, color: color(partes[3])? }))
}
