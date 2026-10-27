//! Referencia CPU secuencial. Nunca se usa para cocinar la geometría del visor vivo.
use crate::{Error, Operacion, Programa};
use alloc::vec::Vec;

/// Datos de un único punto y parámetros de la ejecución.
#[derive(Clone, Copy, Debug)]
pub struct Entrada {
    /// Posición del punto antes de esta operación.
    pub posicion: [f32; 3],
    /// Normal antes de esta operación.
    pub normal: [f32; 3],
    /// Fase del parámetro vivo.
    pub fase: f32,
    /// Amplitud del parámetro vivo.
    pub amplitud: f32,
}

/// Valor evaluado de una instrucción.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Valor {
    /// Número f32.
    Escalar(f32),
    /// Vector de tres componentes.
    Vector([f32; 3]),
}

impl Valor {
    fn finito(self) -> bool {
        match self {
            Self::Escalar(v) => v.is_finite(),
            Self::Vector(v) => v.iter().all(|n| n.is_finite()),
        }
    }

    fn componente(self, indice: usize) -> f32 {
        match self {
            Self::Escalar(v) => v,
            Self::Vector(v) => v[indice],
        }
    }
}

/// Salida interpretada, manteniendo x/z de la entrada.
#[derive(Clone, Copy, Debug)]
pub struct Resultado {
    /// Posición final.
    pub posicion: [f32; 3],
    /// Normal final, según el programa.
    pub normal: [f32; 3],
}

impl Programa {
    /// Ejecuta la misma representación que consume el backend SPIR-V.
    ///
    /// # Errors
    /// Rechaza entradas no finitas o el primer resultado numérico no finito.
    /// El backend GPU no incluye todavía estos diagnósticos numéricos en vivo.
    pub fn interpretar(&self, entrada: Entrada) -> Result<Resultado, Error> {
        self.interpretar_con_parametros(entrada, [1.3, 0.0, 0.0, 0.0])
    }

    /// Evalúa con los mismos cuatro escalares propios enviados a GPU.
    ///
    /// # Errors
    /// Si una entrada o un resultado no es finito.
    pub fn interpretar_con_parametros(&self, entrada: Entrada, parametros: [f32; 4]) -> Result<Resultado, Error> {
        if !entrada
            .posicion
            .iter()
            .chain(&entrada.normal)
            .chain([&entrada.fase, &entrada.amplitud])
            .chain(&parametros)
            .all(|v| v.is_finite())
        {
            return Err(Error::nuevo(0, "entrada no finita"));
        }
        let mut valores = Vec::with_capacity(self.operaciones.len());
        for (indice, operacion) in self.operaciones.iter().enumerate() {
            let valor = evaluar(*operacion, entrada, parametros, &valores);
            if !valor.finito() {
                return Err(Error::nuevo(indice, "resultado numérico no finito"));
            }
            valores.push(valor);
        }
        let altura = valores[self.salida.altura.0].componente(0);
        let normal = valores[self.salida.normal.0];
        Ok(Resultado {
            posicion: [entrada.posicion[0], altura, entrada.posicion[2]],
            normal: core::array::from_fn(|i| normal.componente(i)),
        })
    }
}

fn evaluar(operacion: Operacion, entrada: Entrada, parametros: [f32; 4], valores: &[Valor]) -> Valor {
    let valor = |id: crate::Id| valores[id.0];
    match operacion {
        Operacion::Escalar(v) => Valor::Escalar(v),
        Operacion::Vector(v) => Valor::Vector(v),
        Operacion::Posicion => Valor::Vector(entrada.posicion),
        Operacion::Normal => Valor::Vector(entrada.normal),
        Operacion::Fase => Valor::Escalar(entrada.fase),
        Operacion::Amplitud => Valor::Escalar(entrada.amplitud),
        Operacion::Parametro(indice) => Valor::Escalar(parametros[indice as usize]),
        Operacion::Sumar(a, b) => binaria(valor(a), valor(b), |a, b| a + b),
        Operacion::Restar(a, b) => binaria(valor(a), valor(b), |a, b| a - b),
        Operacion::Multiplicar(a, b) => binaria(valor(a), valor(b), |a, b| a * b),
        Operacion::Extraer(id, componente) => Valor::Escalar(valor(id).componente(componente as usize)),
        Operacion::Componer(ids) => Valor::Vector(ids.map(|id| valor(id).componente(0))),
        Operacion::ProductoEscalar(a, b) => {
            Valor::Escalar((0..3).map(|i| valor(a).componente(i) * valor(b).componente(i)).sum())
        }
        Operacion::Normalizar(id) => {
            let vector = core::array::from_fn(|i| valor(id).componente(i));
            Valor::Vector(normalizar(vector))
        }
    }
}

fn binaria(a: Valor, b: Valor, operacion: impl Fn(f32, f32) -> f32) -> Valor {
    match (a, b) {
        (Valor::Escalar(a), Valor::Escalar(b)) => Valor::Escalar(operacion(a, b)),
        _ => Valor::Vector(core::array::from_fn(|i| operacion(a.componente(i), b.componente(i)))),
    }
}

fn normalizar(vector: [f32; 3]) -> [f32; 3] {
    let escala = vector.iter().map(|v| v.abs()).fold(0.0, f32::max);
    if escala == 0.0 {
        return [0.0, 1.0, 0.0];
    }
    let escalado = vector.map(|v| v / escala);
    let longitud = nudo::mate::raiz(escalado.iter().map(|v| v * v).sum());
    escalado.map(|v| v / longitud)
}
