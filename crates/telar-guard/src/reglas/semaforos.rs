//! Cada carpeta del proyecto declara riesgo y consumo; los padres no ocultan hijos críticos.
use crate::{informe::Problema, repo::Repo};
use std::collections::{BTreeMap, BTreeSet};

const CAMPOS: [&str; 10] =
    ["Estado", "Motivo", "CPU", "GPU", "VRAM", "RAM", "E/S", "Frecuencia", "Limites", "Evidencia"];

fn archivo(carpeta: &str) -> String {
    if carpeta.is_empty() { "SEMAFORO.md".to_owned() } else { format!("{carpeta}/SEMAFORO.md") }
}

pub(crate) fn revisar(repo: &Repo) -> Vec<Problema> {
    let mut carpetas = BTreeSet::from([String::new()]);
    for ruta in &repo.archivos {
        let mut parte = ruta.as_str();
        while let Some((padre, _)) = parte.rsplit_once('/') {
            carpetas.insert(padre.to_owned());
            parte = padre;
        }
    }
    let mut problemas = Vec::new();
    let mut niveles = BTreeMap::new();
    for carpeta in &carpetas {
        let ruta = archivo(carpeta);
        let Some(texto) = repo.leer(&ruta) else {
            problemas.push(Problema::en(&ruta, 0, "falta SEMAFORO.md con riesgo y consumo"));
            continue;
        };
        let mut estado = None;
        for campo in CAMPOS {
            let prefijo = format!("{campo}:");
            let valores: Vec<_> = texto.lines().filter_map(|l| l.strip_prefix(&prefijo)).collect();
            if valores.len() != 1 || valores[0].trim().chars().count() < 4 {
                problemas.push(Problema::en(&ruta, 0, format!("campo {campo} ausente, duplicado o sin explicación")));
            } else if campo == "Estado" {
                estado = match valores[0].trim() {
                    "ROJO" => Some(2),
                    "AMARILLO" => Some(1),
                    "VERDE" => Some(0),
                    _ => None,
                };
                if estado.is_none() {
                    problemas.push(Problema::en(&ruta, 0, "Estado debe ser ROJO, AMARILLO o VERDE"));
                }
            }
        }
        if let Some(nivel) = estado {
            niveles.insert(carpeta.as_str(), nivel);
            if carpeta.split('/').any(|p| p.eq_ignore_ascii_case("ffi")) && nivel != 2 {
                problemas.push(Problema::en(&ruta, 0, "una frontera FFI debe declarar ROJO"));
            }
        }
    }
    for (carpeta, nivel) in &niveles {
        if carpeta.is_empty() {
            continue;
        }
        let padre = carpeta.rsplit_once('/').map_or("", |(p, _)| p);
        if niveles.get(padre).is_some_and(|p| p < nivel) {
            problemas.push(Problema::en(
                &archivo(padre),
                0,
                format!("su riesgo no puede ser menor que el de {carpeta}"),
            ));
        }
    }
    problemas
}
