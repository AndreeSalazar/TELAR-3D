//! `unir`: junta varias geometrías en una. Ver `libreto.md`.

use alloc::string::String;
use alloc::vec::Vec;

use nudo::{Clase, Geometria, Nodo, Valor};

/// Junta sus entradas, en orden, en una sola geometría.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Unir;

impl Nodo for Unir {
    fn tipo(&self) -> &'static str {
        "unir"
    }

    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        Vec::new()
    }

    fn poner(&mut self, nombre: &str, _valor: Valor) -> Result<(), String> {
        Err(crate::parametros::desconocido(self.tipo(), nombre))
    }

    fn cocinar(&self, entradas: &[&Geometria]) -> Result<Geometria, String> {
        let mut g = Geometria::nueva();
        for entrada in entradas {
            juntar(&mut g, entrada)?;
        }
        Ok(g)
    }
}

/// Añade `otra` al final de `g`: sus puntos detrás de los de `g`, sus vértices
/// apuntando a los puntos ya desplazados, y sus primitivas detrás. Un atributo
/// que solo tiene una de las dos se rellena con ceros en la otra.
fn juntar(g: &mut Geometria, otra: &Geometria) -> Result<(), String> {
    let desplazar = |n: usize| u32::try_from(n).map_err(|_| String::from("demasiados elementos para u32"));
    let (puntos, vertices) = (desplazar(g.puntos.len())?, desplazar(g.vertices.len())?);
    let antes = [Clase::Punto, Clase::Vertice, Clase::Primitiva].map(|c| g.cantidad(c));
    let despues = [Clase::Punto, Clase::Vertice, Clase::Primitiva].map(|c| otra.cantidad(c));

    g.puntos.extend_from_slice(&otra.puntos);
    g.vertices.extend(otra.vertices.iter().map(|v| v + puntos));
    g.primitivas.extend(otra.primitivas.iter().map(|p| p.start + vertices..p.end + vertices));

    let indice = |c: Clase| match c {
        Clase::Punto => Some(0),
        Clase::Vertice => Some(1),
        Clase::Primitiva => Some(2),
        Clase::Detalle => None, // un solo valor: se queda el de la primera que lo tenga
    };
    let claves: Vec<_> = g.atributos.keys().chain(otra.atributos.keys()).cloned().collect();
    for clave in claves {
        let Some(i) = indice(clave.0) else {
            if let (None, Some(v)) = (g.atributos.get(&clave), otra.atributos.get(&clave)) {
                g.atributos.insert(clave, v.clone());
            }
            continue;
        };
        let nuevo = match (g.atributos.get(&clave), otra.atributos.get(&clave)) {
            (Some(a), Some(b)) => a
                .juntar(b)
                .ok_or_else(|| alloc::format!("el atributo `{}` tiene tipos distintos en las entradas", clave.1))?,
            (Some(a), None) => a.con_ceros(despues[i]),
            (None, Some(b)) => b.ceros(antes[i]).juntar(b).unwrap_or_else(|| b.clone()),
            (None, None) => continue,
        };
        g.atributos.insert(clave, nuevo);
    }
    g.comprobar()
}
