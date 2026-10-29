//! Perfiles GEO clásicos propios: muestras y partículas con atributos de vida.
use crate::{MAX_BYTES, MAX_PUNTOS, Punto, validar};
use alloc::{string::String, vec::Vec};
use core::fmt::Write;
const ATRIBUTOS: &str = "v 3 float 0 0 0\ndensity 1 float 0\nheat 1 float 0\nid 1 int 0\nsim_time 1 float 0\n";
const VIDA: &str = "age 1 float 0\nlife 1 float 1\nbirth_time 1 float 0\n";
/// Codifica el perfil de cinco atributos, sin primitivas ni grupos.
/// # Errors
/// Rechaza puntos inválidos y texto fuera de 16 MiB.
pub fn codificar_geo(puntos: &[Punto]) -> Result<String, &'static str> {
    validar(puntos)?;
    let mut texto = cabecera(puntos.len(), false)?;
    for p in puntos {
        escribir_punto(&mut texto, p)?;
        texto.push_str(" )\n");
        limite(&texto)?;
    }
    texto.push_str("beginExtra\nendExtra\n");
    Ok(texto)
}
/// Lee únicamente el perfil de muestras de cinco atributos; no un GEO arbitrario.
/// # Errors
/// Rechaza variantes, truncamiento, datos inválidos y exceso de tamaño.
pub fn decodificar_geo(texto: &str) -> Result<Vec<Punto>, &'static str> {
    let (mut tokens, cantidad) = iniciar(texto, false)?;
    let mut puntos = Vec::with_capacity(cantidad);
    for _ in 0..cantidad {
        let p = leer_punto(&mut tokens)?;
        tokens.exigir(")")?;
        puntos.push(p);
    }
    finalizar(&mut tokens)?;
    validar(&puntos)?;
    Ok(puntos)
}
pub(super) fn cabecera(cantidad: usize, vida: bool) -> Result<String, &'static str> {
    let mut t = String::new();
    let atributos = if vida { 8 } else { 5 };
    let extra = if vida { VIDA } else { "" };
    writeln!(t,"PGEOMETRY V5\nNPoints {cantidad} NPrims 0\nNPointGroups 0 NPrimGroups 0\nNPointAttrib {atributos} NVertexAttrib 0 NPrimAttrib 0 NAttrib 0\nPointAttrib\n{ATRIBUTOS}{extra}").map_err(|_|"cabecera GEO inválida")?;
    Ok(t)
}
pub(super) fn escribir_punto(texto: &mut String, p: &Punto) -> Result<(), &'static str> {
    let [x, y, z] = p.posicion;
    let [vx, vy, vz] = p.velocidad;
    write!(
        texto,
        "{x:.8e} {y:.8e} {z:.8e} 1 ( {vx:.8e} {vy:.8e} {vz:.8e} {:.8e} {:.8e} {} {:.8e}",
        p.densidad, p.calor, p.id, p.tiempo
    )
    .map_err(|_| "dato GEO inválido")
}
pub(super) fn limite(t: &str) -> Result<(), &'static str> {
    if t.len() > MAX_BYTES - 20 {
        return Err("GEO excede 16 MiB");
    }
    Ok(())
}
pub(super) fn iniciar(texto: &str, vida: bool) -> Result<(Tokens<'_>, usize), &'static str> {
    if texto.len() > MAX_BYTES || !texto.is_ascii() {
        return Err("GEO fuera de tamaño o codificación ASCII");
    }
    let mut t = Tokens(texto);
    t.exigir("PGEOMETRY")?;
    t.exigir("V5")?;
    t.exigir("NPoints")?;
    let cantidad = usize::try_from(t.entero()?).map_err(|_| "recuento GEO inválido")?;
    if cantidad > MAX_PUNTOS {
        return Err("exceso de puntos GEO");
    }
    for p in "NPrims 0 NPointGroups 0 NPrimGroups 0 NPointAttrib".split_ascii_whitespace() {
        t.exigir(p)?;
    }
    t.exigir(if vida { "8" } else { "5" })?;
    for p in "NVertexAttrib 0 NPrimAttrib 0 NAttrib 0 PointAttrib".split_ascii_whitespace() {
        t.exigir(p)?;
    }
    for p in ATRIBUTOS.split_ascii_whitespace() {
        t.exigir(p)?;
    }
    if vida {
        for p in VIDA.split_ascii_whitespace() {
            t.exigir(p)?;
        }
    }
    if t.0.len() < cantidad * 16 {
        return Err("GEO truncado antes de los puntos");
    }
    Ok((t, cantidad))
}
pub(super) fn leer_punto(t: &mut Tokens<'_>) -> Result<Punto, &'static str> {
    let posicion = [t.real()?, t.real()?, t.real()?];
    t.exigir("1")?;
    t.exigir("(")?;
    let velocidad = [t.real()?, t.real()?, t.real()?];
    let densidad = t.real()?;
    let calor = t.real()?;
    let id = t.entero()?;
    let tiempo = t.real()?;
    Ok(Punto { posicion, velocidad, densidad, calor, id, tiempo })
}
pub(super) fn finalizar(t: &mut Tokens<'_>) -> Result<(), &'static str> {
    t.exigir("beginExtra")?;
    t.exigir("endExtra")?;
    if !t.0.trim().is_empty() {
        return Err("contenido GEO adicional no soportado");
    }
    Ok(())
}
pub(super) struct Tokens<'a>(pub(super) &'a str);
impl<'a> Tokens<'a> {
    pub(super) fn siguiente(&mut self) -> Result<&'a str, &'static str> {
        self.0 = self.0.trim_start();
        let primero = *self.0.as_bytes().first().ok_or("GEO truncado")?;
        let cantidad = if matches!(primero, b'(' | b')') {
            1
        } else {
            self.0.find(|c: char| c.is_ascii_whitespace() || c == '(' || c == ')').unwrap_or(self.0.len())
        };
        if cantidad == 0 || cantidad > 64 {
            return Err("token GEO fuera de tamaño");
        }
        let (token, resto) = self.0.split_at(cantidad);
        self.0 = resto;
        Ok(token)
    }
    pub(super) fn exigir(&mut self, valor: &str) -> Result<(), &'static str> {
        if self.siguiente()? != valor {
            return Err("esquema GEO no soportado");
        }
        Ok(())
    }
    pub(super) fn real(&mut self) -> Result<f32, &'static str> {
        self.siguiente()?.parse().map_err(|_| "real GEO inválido")
    }
    pub(super) fn entero(&mut self) -> Result<u32, &'static str> {
        self.siguiente()?.parse().map_err(|_| "entero GEO inválido")
    }
}
