//! Subconjunto textual PGEOMETRY V5, implementación propia según documentación pública.
use crate::{MAX_BYTES, MAX_PUNTOS, Punto, validar};
use alloc::{string::String, vec::Vec};
use core::fmt::Write;
const ESQUEMA: &str = "NPrims 0\nNPointGroups 0 NPrimGroups 0\nNPointAttrib 5 NVertexAttrib 0 NPrimAttrib 0 NAttrib 0\nPointAttrib\nv 3 float 0 0 0\ndensity 1 float 0\nheat 1 float 0\nid 1 int 0\nsim_time 1 float 0\n";
/// Codifica puntos sin primitivas ni grupos, con cinco atributos de punto fijos.
/// # Errors
/// Rechaza datos inválidos o tamaño superior a 16 MiB.
pub fn codificar_geo(puntos: &[Punto]) -> Result<String, &'static str> {
    validar(puntos)?;
    let mut texto = String::new();
    writeln!(texto, "PGEOMETRY V5\nNPoints {} {ESQUEMA}", puntos.len()).map_err(|_| "formato GEO inválido")?;
    for p in puntos {
        let [x, y, z] = p.posicion;
        let [vx, vy, vz] = p.velocidad;
        writeln!(
            texto,
            "{x:.8e} {y:.8e} {z:.8e} 1 ( {vx:.8e} {vy:.8e} {vz:.8e} {:.8e} {:.8e} {} {:.8e} )",
            p.densidad, p.calor, p.id, p.tiempo
        )
        .map_err(|_| "formato GEO inválido")?;
        if texto.len() > MAX_BYTES - 20 {
            return Err("GEO excede 16 MiB");
        }
    }
    texto.push_str("beginExtra\nendExtra\n");
    Ok(texto)
}
/// Lee únicamente el contrato TELAR GEO V5; conserva orden y precisión f32.
/// # Errors
/// Rechaza esquema/variantes distintos, truncamiento, datos inválidos, texto final y exceso de tamaño.
pub fn decodificar_geo(texto: &str) -> Result<Vec<Punto>, &'static str> {
    if texto.len() > MAX_BYTES || !texto.is_ascii() {
        return Err("GEO fuera de tamaño o codificación ASCII");
    }
    let mut tokens = Tokens(texto);
    tokens.exigir("PGEOMETRY")?;
    tokens.exigir("V5")?;
    tokens.exigir("NPoints")?;
    let cantidad = tokens.entero()?;
    let cantidad = usize::try_from(cantidad).map_err(|_| "cantidad GEO inválida")?;
    if cantidad > MAX_PUNTOS {
        return Err("exceso de puntos GEO");
    }
    for palabra in ESQUEMA.split_ascii_whitespace() {
        tokens.exigir(palabra)?;
    }
    // Comprobar longitud mínima antes de reservar a partir de una cabecera no fiable.
    if tokens.0.len() < cantidad * 16 {
        return Err("GEO truncado antes de los puntos");
    }
    let mut puntos = Vec::with_capacity(cantidad);
    for _ in 0..cantidad {
        let posicion = [tokens.real()?, tokens.real()?, tokens.real()?];
        tokens.exigir("1")?;
        tokens.exigir("(")?;
        let velocidad = [tokens.real()?, tokens.real()?, tokens.real()?];
        let densidad = tokens.real()?;
        let calor = tokens.real()?;
        let id = tokens.entero()?;
        let tiempo = tokens.real()?;
        tokens.exigir(")")?;
        puntos.push(Punto { posicion, velocidad, densidad, calor, id, tiempo });
    }
    tokens.exigir("beginExtra")?;
    tokens.exigir("endExtra")?;
    if !tokens.0.trim().is_empty() {
        return Err("contenido GEO adicional no soportado");
    }
    validar(&puntos)?;
    Ok(puntos)
}
struct Tokens<'a>(&'a str);
impl<'a> Tokens<'a> {
    fn siguiente(&mut self) -> Result<&'a str, &'static str> {
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
    fn exigir(&mut self, valor: &str) -> Result<(), &'static str> {
        if self.siguiente()? != valor {
            return Err("esquema GEO no soportado");
        }
        Ok(())
    }
    fn real(&mut self) -> Result<f32, &'static str> {
        self.siguiente()?.parse().map_err(|_| "real GEO inválido")
    }
    fn entero(&mut self) -> Result<u32, &'static str> {
        self.siguiente()?.parse().map_err(|_| "entero GEO inválido")
    }
}
