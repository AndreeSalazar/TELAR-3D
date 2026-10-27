//! Analizador estricto y acotado de reglas de clase y variables en :root.
use crate::{Hoja, Regla};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

pub(super) fn analizar(css: &str) -> Result<Hoja, String> {
    if css.len() > 1_048_576 {
        return Err("hoja CSS mayor de 1 MiB".to_string());
    }
    let limpio = comentarios(css)?;
    let mut pendiente = limpio.trim();
    let mut bloques = Vec::new();
    while !pendiente.is_empty() {
        let (selector, resto) = pendiente.split_once('{').ok_or("falta { en regla CSS")?;
        let (cuerpo, resto) = resto.split_once('}').ok_or("falta } en regla CSS")?;
        if cuerpo.contains('{') || selector.contains('}') {
            return Err("regla CSS anidada o mal cerrada".to_string());
        }
        bloques.push((selector.trim(), declaraciones(cuerpo)?));
        if bloques.len() > 512 {
            return Err("demasiadas reglas CSS".to_string());
        }
        pendiente = resto.trim();
    }
    let mut variables: Vec<(&str, &str)> = Vec::new();
    for (selector, declaraciones) in &bloques {
        if *selector == ":root" {
            for &(nombre, valor) in declaraciones {
                if !nombre.starts_with("--") || !identificador(&nombre[2..]) {
                    return Err("solo variables --nombre en :root".to_string());
                }
                variables.push((nombre, valor));
            }
        }
    }
    // También se validan variables que aún no usa ninguna regla.
    for &(nombre, _) in &variables {
        resolver(&format!("var({nombre})"), &variables, 0)?;
    }
    let mut hoja = Hoja::default();
    for (selector, declaraciones) in bloques {
        if selector == ":root" {
            continue;
        }
        let (clase, estado) = selector_clase(selector)?;
        let mut regla = Regla { clase: clase.to_string(), estado, ..Regla::default() };
        for (nombre, valor) in declaraciones {
            regla.estilo.agregar(nombre, &resolver(valor, &variables, 0)?)?;
        }
        hoja.reglas.push(regla);
    }
    Ok(hoja)
}

fn comentarios(css: &str) -> Result<String, String> {
    let mut limpio = String::new();
    let mut resto = css;
    while let Some((antes, despues)) = resto.split_once("/*") {
        limpio.push_str(antes);
        limpio.push(' ');
        resto = despues.split_once("*/").ok_or("comentario CSS sin cerrar")?.1;
    }
    limpio.push_str(resto);
    Ok(limpio)
}

fn declaraciones(cuerpo: &str) -> Result<Vec<(&str, &str)>, String> {
    cuerpo
        .split(';')
        .filter(|d| !d.trim().is_empty())
        .map(|d| {
            let (nombre, valor) = d.split_once(':').ok_or("declaración CSS sin :")?;
            if nombre.trim().is_empty() || valor.trim().is_empty() {
                return Err("declaración CSS vacía".to_string());
            }
            Ok((nombre.trim(), valor.trim()))
        })
        .collect()
}

fn identificador(texto: &str) -> bool {
    !texto.is_empty() && texto.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

fn selector_clase(selector: &str) -> Result<(&str, u8), String> {
    let mut partes = selector.strip_prefix('.').ok_or("se esperaba selector .clase")?.split(':');
    let clase = partes.next().ok_or("falta clase CSS")?;
    if !identificador(clase) {
        return Err(format!("selector no implementado: {selector}"));
    }
    let mut estado = 0;
    for parte in partes {
        estado |= match parte {
            "hover" => 1,
            "checked" => 2,
            "focus" => 4,
            "active" => 8,
            _ => return Err(format!("estado CSS no implementado: {parte}")),
        };
    }
    Ok((clase, estado))
}

fn resolver(valor: &str, variables: &[(&str, &str)], nivel: usize) -> Result<String, String> {
    if nivel > 16 {
        return Err("variables CSS cíclicas o demasiado profundas".to_string());
    }
    let Some((antes, resto)) = valor.split_once("var(") else {
        return Ok(valor.to_string());
    };
    let (nombre, despues) = resto.split_once(')').ok_or("var() sin cerrar")?;
    let reemplazo = variables
        .iter()
        .rev()
        .find(|(n, _)| *n == nombre.trim())
        .ok_or_else(|| format!("variable CSS ausente: {nombre}"))?
        .1;
    let reemplazo = resolver(reemplazo, variables, nivel + 1)?;
    let despues = resolver(despues, variables, nivel + 1)?;
    if antes.len() + reemplazo.len() + despues.len() > 65_536 {
        return Err("expansión de variable CSS demasiado grande".to_string());
    }
    Ok(format!("{antes}{reemplazo}{despues}"))
}
