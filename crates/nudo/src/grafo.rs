//! El grafo que cocina: cada nodo produce una geometría a partir de las de sus
//! entradas, y **solo se recalcula lo que cambió**.
//!
//! - Un nodo solo puede tener como entradas nodos que ya existen, así que el
//!   grafo no puede tener ciclos y el orden de los números es un orden válido.
//! - Cambiar un parámetro marca **sucio** ese nodo y todo lo que depende de él;
//!   lo demás conserva su resultado.
//! - `cocinar` recorre hacia atrás desde el nodo pedido y solo recalcula los sucios.
//! - Lleva la cuenta de cuántos nodos se han cocinado, para comprobarlo.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::geometria::Geometria;

/// El valor de un parámetro.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Valor {
    /// Un número real.
    Real(f32),
    /// Un entero.
    Entero(i32),
    /// Tres reales (una posición, un tamaño, unos ángulos).
    Vector([f32; 3]),
}

/// Un nodo: un nombre, unos parámetros y una forma de cocinar.
pub trait Nodo {
    /// El tipo de nodo («caja», «transformar»…).
    fn tipo(&self) -> &'static str;

    /// Los parámetros con su valor actual, en orden.
    fn parametros(&self) -> Vec<(&'static str, Valor)>;

    /// Cambia un parámetro.
    ///
    /// # Errors
    ///
    /// Si no existe o el valor no es del tipo que espera.
    fn poner(&mut self, nombre: &str, valor: Valor) -> Result<(), String>;

    /// Produce la geometría a partir de las de sus entradas.
    ///
    /// # Errors
    ///
    /// Si las entradas no son las que espera, o un parámetro no tiene sentido.
    fn cocinar(&self, entradas: &[&Geometria]) -> Result<Geometria, String>;
}

/// El número de un nodo dentro del grafo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdNodo(pub usize);

struct Ranura {
    nodo: Box<dyn Nodo>,
    entradas: Vec<IdNodo>,
    resultado: Option<Geometria>,
    sucio: bool,
}

/// Un grafo de nodos.
#[derive(Default)]
pub struct Grafo {
    ranuras: Vec<Ranura>,
    cocinados: u64,
}

impl Grafo {
    /// Un grafo vacío.
    #[must_use]
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Añade un nodo con sus entradas (nodos que ya existen) y devuelve su número.
    ///
    /// # Errors
    ///
    /// Si alguna entrada no existe todavía.
    pub fn anadir(&mut self, nodo: impl Nodo + 'static, entradas: &[IdNodo]) -> Result<IdNodo, String> {
        let id = IdNodo(self.ranuras.len());
        if let Some(e) = entradas.iter().find(|e| e.0 >= id.0) {
            return Err(alloc::format!("la entrada {} no existe todavía (el nodo nuevo sería el {})", e.0, id.0));
        }
        self.ranuras.push(Ranura { nodo: Box::new(nodo), entradas: entradas.to_vec(), resultado: None, sucio: true });
        Ok(id)
    }

    /// Cambia un parámetro y marca sucio ese nodo y todo lo que depende de él.
    /// Si el valor es el mismo que ya tenía, no ensucia nada.
    ///
    /// # Errors
    ///
    /// Si el nodo no existe o rechaza el parámetro.
    pub fn poner(&mut self, id: IdNodo, nombre: &str, valor: Valor) -> Result<(), String> {
        let ranura = self.ranuras.get_mut(id.0).ok_or("ese nodo no existe")?;
        if ranura.nodo.parametros().iter().any(|&(n, v)| n == nombre && v == valor) {
            return Ok(());
        }
        ranura.nodo.poner(nombre, valor)?;
        self.ensuciar(id);
        Ok(())
    }

    /// Marca sucio `id` y, en orden, cada nodo que tenga una entrada sucia.
    fn ensuciar(&mut self, id: IdNodo) {
        let mut sucios = alloc::vec![false; self.ranuras.len()];
        sucios[id.0] = true;
        for i in id.0..self.ranuras.len() {
            if i == id.0 || self.ranuras[i].entradas.iter().any(|e| sucios[e.0]) {
                sucios[i] = true;
                self.ranuras[i].sucio = true;
            }
        }
    }

    /// El resultado de `id`, cocinando antes lo que haga falta (y solo eso).
    ///
    /// # Errors
    ///
    /// Si el nodo no existe o algún nodo del camino falla al cocinar.
    pub fn cocinar(&mut self, id: IdNodo) -> Result<&Geometria, String> {
        let entradas = self.ranuras.get(id.0).ok_or("ese nodo no existe")?.entradas.clone();
        for e in &entradas {
            self.cocinar(*e)?;
        }
        if self.ranuras[id.0].sucio || self.ranuras[id.0].resultado.is_none() {
            let resultado = {
                let geos: Vec<&Geometria> =
                    entradas.iter().filter_map(|e| self.ranuras[e.0].resultado.as_ref()).collect();
                self.ranuras[id.0]
                    .nodo
                    .cocinar(&geos)
                    .map_err(|e| alloc::format!("{} (nodo {}): {e}", self.ranuras[id.0].nodo.tipo(), id.0))?
            };
            let ranura = &mut self.ranuras[id.0];
            (ranura.resultado, ranura.sucio) = (Some(resultado), false);
            self.cocinados += 1;
        }
        self.ranuras[id.0].resultado.as_ref().ok_or_else(|| String::from("el nodo no tiene resultado"))
    }

    /// Cuántos nodos se han cocinado desde que se creó el grafo.
    #[must_use]
    pub fn cocinados(&self) -> u64 {
        self.cocinados
    }

    /// ¿Está sucio ese nodo?
    #[must_use]
    pub fn sucio(&self, id: IdNodo) -> bool {
        self.ranuras.get(id.0).is_some_and(|r| r.sucio)
    }

    /// El nodo, para leer sus parámetros.
    #[must_use]
    pub fn nodo(&self, id: IdNodo) -> Option<&dyn Nodo> {
        self.ranuras.get(id.0).map(|r| &*r.nodo)
    }
}
