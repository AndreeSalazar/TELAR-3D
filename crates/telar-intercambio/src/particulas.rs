//! Perfil GEO de partículas vivas; conserva edad/vida/nacimiento además de la muestra.
use crate::{Punto, geo, validar};
use alloc::{string::String, vec::Vec};
use core::fmt::Write;
/// Partícula viva exportada; no contiene todo el estado del emisor reanudable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particula {
    /// Posición, velocidad, identificador e instante de la exportación.
    pub punto: Punto,
    /// Tiempo integrado desde el nacimiento, en segundos.
    pub edad: f32,
    /// Duración efectiva asignada a este nacimiento, en segundos.
    pub vida: f32,
    /// Reloj GPU en que nació; independiente de redondeos acumulados en edad.
    pub nacimiento: f32,
}
/// Codifica el perfil fijo de ocho atributos de punto de partículas vivas.
/// # Errors
/// Rechaza datos/identidades inválidos, partículas muertas y texto fuera de límite.
pub fn codificar_particulas_geo(p: &[Particula]) -> Result<String, &'static str> {
    validar_particulas(p)?;
    let mut texto = geo::cabecera(p.len(), true)?;
    for p in p {
        geo::escribir_punto(&mut texto, &p.punto)?;
        writeln!(texto, " {:.8e} {:.8e} {:.8e} )", p.edad, p.vida, p.nacimiento).map_err(|_| "vida GEO inválida")?;
        geo::limite(&texto)?;
    }
    texto.push_str("beginExtra\nendExtra\n");
    Ok(texto)
}
/// Lee únicamente el perfil de ocho atributos; no convierte el perfil de muestras.
/// # Errors
/// Rechaza esquema/valores inválidos y contenido no soportado.
pub fn decodificar_particulas_geo(texto: &str) -> Result<Vec<Particula>, &'static str> {
    let (mut t, cantidad) = geo::iniciar(texto, true)?;
    let mut puntos = Vec::with_capacity(cantidad);
    for _ in 0..cantidad {
        let punto = geo::leer_punto(&mut t)?;
        let edad = t.real()?;
        let vida = t.real()?;
        let nacimiento = t.real()?;
        t.exigir(")")?;
        puntos.push(Particula { punto, edad, vida, nacimiento });
    }
    geo::finalizar(&mut t)?;
    validar_particulas(&puntos)?;
    Ok(puntos)
}
/// # Errors
/// Rechaza vida no positiva, edad fuera de [0,vida), nacimiento futuro/negativo y datos inválidos.
pub fn validar_particulas(p: &[Particula]) -> Result<(), &'static str> {
    if p.len() > crate::MAX_PUNTOS {
        return Err("exceso de partículas GEO");
    }
    for v in p {
        if [v.edad, v.vida, v.nacimiento].iter().any(|f| !f.is_finite() || f.abs() > 1e20)
            || v.edad < 0.0
            || v.vida <= 0.0
            || v.edad >= v.vida
            || v.nacimiento < 0.0
            || v.nacimiento > v.punto.tiempo
        {
            return Err("atributo de vida fuera de dominio");
        }
    }
    let base: Vec<_> = p.iter().map(|v| v.punto).collect();
    validar(&base)
}
