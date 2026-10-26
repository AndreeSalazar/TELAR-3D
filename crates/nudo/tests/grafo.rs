//! El grafo cocina solo lo que cambió: un contador lo demuestra.
//!
//! ```text
//!   A (fuente) ──► C (juntar) ──► D (juntar)
//!   B (fuente) ─────────────────────┘
//! ```

#![allow(clippy::unwrap_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use nudo::{Geometria, Grafo, IdNodo, Nodo, Valor};

/// Un nodo sin entradas: un punto en `[x, 0, 0]`.
struct Fuente(f32);

impl Nodo for Fuente {
    fn tipo(&self) -> &'static str {
        "fuente"
    }
    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        vec![("x", Valor::Real(self.0))]
    }
    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String> {
        match (nombre, valor) {
            ("x", Valor::Real(x)) => {
                self.0 = x;
                Ok(())
            }
            _ => Err(format!("no sé poner {nombre}")),
        }
    }
    fn cocinar(&self, _: &[&Geometria]) -> Result<Geometria, String> {
        if self.0 < 0.0 {
            return Err("x negativo".to_owned());
        }
        let mut g = Geometria::nueva();
        g.punto([self.0, 0.0, 0.0]);
        Ok(g)
    }
}

/// Junta los puntos de sus entradas.
struct Juntar;

impl Nodo for Juntar {
    fn tipo(&self) -> &'static str {
        "juntar"
    }
    fn parametros(&self) -> Vec<(&'static str, Valor)> {
        Vec::new()
    }
    fn poner(&mut self, nombre: &str, _: Valor) -> Result<(), String> {
        Err(format!("no tengo {nombre}"))
    }
    fn cocinar(&self, entradas: &[&Geometria]) -> Result<Geometria, String> {
        let mut g = Geometria::nueva();
        for e in entradas {
            g.puntos.extend_from_slice(&e.puntos);
        }
        Ok(g)
    }
}

fn escena() -> (Grafo, [IdNodo; 4]) {
    let mut grafo = Grafo::nuevo();
    let a = grafo.anadir(Fuente(1.0), &[]).unwrap();
    let b = grafo.anadir(Fuente(2.0), &[]).unwrap();
    let c = grafo.anadir(Juntar, &[a]).unwrap();
    let d = grafo.anadir(Juntar, &[c, b]).unwrap();
    (grafo, [a, b, c, d])
}

#[test]
fn solo_se_cocina_lo_que_cambio() {
    let (mut grafo, [a, b, c, d]) = escena();
    assert_eq!(grafo.cocinar(d).unwrap().puntos, [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
    assert_eq!(grafo.cocinados(), 4, "la primera vez, los cuatro");

    grafo.cocinar(d).unwrap();
    assert_eq!(grafo.cocinados(), 4, "sin cambios, ninguno");

    // Cambiar A ensucia A, C y D; B se queda como estaba.
    grafo.poner(a, "x", Valor::Real(5.0)).unwrap();
    assert!(grafo.sucio(a) && grafo.sucio(c) && grafo.sucio(d) && !grafo.sucio(b));
    assert_eq!(grafo.cocinar(d).unwrap().puntos[0], [5.0, 0.0, 0.0]);
    assert_eq!(grafo.cocinados(), 4 + 3);

    // El mismo valor que ya tenía: no ensucia nada.
    grafo.poner(b, "x", Valor::Real(2.0)).unwrap();
    grafo.cocinar(d).unwrap();
    assert_eq!(grafo.cocinados(), 7);

    // Cambiar B: B y D, nada más.
    grafo.poner(b, "x", Valor::Real(3.0)).unwrap();
    grafo.cocinar(d).unwrap();
    assert_eq!(grafo.cocinados(), 7 + 2);

    // Pedir solo C no cocina D.
    grafo.poner(a, "x", Valor::Real(6.0)).unwrap();
    grafo.cocinar(c).unwrap();
    assert_eq!(grafo.cocinados(), 9 + 2);
    assert!(grafo.sucio(d));
}

#[test]
fn las_entradas_tienen_que_existir_y_no_hay_ciclos() {
    let mut grafo = Grafo::nuevo();
    // Un nodo no puede ser su propia entrada ni la de uno anterior: no puede haber ciclos.
    assert!(grafo.anadir(Juntar, &[IdNodo(0)]).is_err());
    let a = grafo.anadir(Fuente(1.0), &[]).unwrap();
    assert!(grafo.anadir(Juntar, &[a, IdNodo(7)]).is_err());
}

#[test]
fn un_error_dice_que_nodo_fallo_y_no_rompe_nada() {
    let (mut grafo, [a, _, _, d]) = escena();
    grafo.poner(a, "x", Valor::Real(-1.0)).unwrap();
    let error = grafo.cocinar(d).unwrap_err();
    assert_eq!(error, "fuente (nodo 0): x negativo");
    // Arreglado el parámetro, todo vuelve a cocinar.
    grafo.poner(a, "x", Valor::Real(4.0)).unwrap();
    assert_eq!(grafo.cocinar(d).unwrap().puntos.len(), 2);
    assert!(grafo.poner(a, "y", Valor::Real(1.0)).is_err());
}
