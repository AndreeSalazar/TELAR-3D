//! Una función SPIR-V en construcción: sus bloques y sus instrucciones. Los
//! tipos y constantes que necesite se declaran en el módulo ([`Funcion::modulo`]).

use alloc::vec::Vec;

use crate::modulo::{Id, Modulo, instruccion};
use crate::op;

/// Una función en construcción. Sus instrucciones van a su propio cuerpo; los
/// tipos y constantes que necesite se declaran en el módulo ([`Funcion::modulo`]).
pub struct Funcion<'a> {
    pub(crate) modulo: &'a mut Modulo,
    pub(crate) id: Id,
    pub(crate) cuerpo: Vec<u32>,
}

impl Funcion<'_> {
    /// El `Id` de la función (para el punto de entrada).
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// El módulo, para declarar tipos y constantes.
    pub fn modulo(&mut self) -> &mut Modulo {
        self.modulo
    }

    /// Una instrucción con resultado: `tipo`, el `Id` nuevo y los operandos.
    fn con_resultado(&mut self, codigo: u16, tipo: Id, operandos: &[u32]) -> Id {
        let id = self.modulo.id();
        let mut todo = alloc::vec![tipo.0, id.0];
        todo.extend_from_slice(operandos);
        instruccion(&mut self.cuerpo, codigo, &todo);
        id
    }

    /// Reserva el `Id` de un bloque que se abrirá más tarde (para saltar a él).
    pub fn etiqueta(&mut self) -> Id {
        self.modulo.id()
    }

    /// Abre el bloque `etiqueta` (`OpLabel`).
    pub fn bloque(&mut self, etiqueta: Id) {
        instruccion(&mut self.cuerpo, op::LABEL, &[etiqueta.0]);
    }

    /// `OpLoad`.
    pub fn cargar(&mut self, tipo: Id, puntero: Id) -> Id {
        self.con_resultado(op::LOAD, tipo, &[puntero.0])
    }

    /// `OpStore`.
    pub fn guardar(&mut self, puntero: Id, valor: Id) {
        instruccion(&mut self.cuerpo, op::STORE, &[puntero.0, valor.0]);
    }

    /// `OpAccessChain`: un puntero a un elemento de `base`.
    pub fn acceso(&mut self, tipo_puntero: Id, base: Id, indices: &[Id]) -> Id {
        let mut operandos = alloc::vec![base.0];
        operandos.extend(indices.iter().map(|i| i.0));
        self.con_resultado(op::ACCESS_CHAIN, tipo_puntero, &operandos)
    }

    /// `OpArrayLength`: cuántos elementos tiene el array dinámico del miembro `miembro`.
    pub fn longitud(&mut self, tipo_u32: Id, estructura: Id, miembro: u32) -> Id {
        self.con_resultado(op::ARRAY_LENGTH, tipo_u32, &[estructura.0, miembro])
    }

    /// Una operación de dos operandos (`OpIAdd`, `OpBitwiseXor`, `OpULessThan`…).
    pub fn binaria(&mut self, codigo: u16, tipo: Id, a: Id, b: Id) -> Id {
        self.con_resultado(codigo, tipo, &[a.0, b.0])
    }

    /// Conversión o reinterpretación de un valor (`OpConvertUToF`, `OpBitcast`…).
    pub fn unaria(&mut self, codigo: u16, tipo: Id, valor: Id) -> Id {
        self.con_resultado(codigo, tipo, &[valor.0])
    }

    /// Elige un valor sin bifurcar (`OpSelect`).
    pub fn elegir(&mut self, tipo: Id, condicion: Id, si: Id, no: Id) -> Id {
        self.con_resultado(op::SELECT, tipo, &[condicion.0, si.0, no.0])
    }
    /// `OpPhi`: valores y etiquetas predecesoras. Debe ser la primera instrucción
    /// de un bloque de fusión, antes de operaciones que no sean otros Phi.
    pub fn phi(&mut self, tipo: Id, entradas: &[(Id, Id)]) -> Id {
        let operandos: Vec<_> = entradas.iter().flat_map(|(valor, bloque)| [valor.0, bloque.0]).collect();
        self.con_resultado(245, tipo, &operandos)
    }

    /// `OpSelectionMerge`: dónde se juntan las dos ramas del `if` que sigue.
    pub fn fusion(&mut self, fusion: Id) {
        instruccion(&mut self.cuerpo, op::SELECTION_MERGE, &[fusion.0, op::CONTROL_NINGUNO]);
    }

    /// `OpBranchConditional`.
    pub fn saltar_si(&mut self, condicion: Id, si: Id, no: Id) {
        instruccion(&mut self.cuerpo, op::BRANCH_CONDITIONAL, &[condicion.0, si.0, no.0]);
    }

    /// `OpBranch`.
    pub fn saltar(&mut self, destino: Id) {
        instruccion(&mut self.cuerpo, op::BRANCH, &[destino.0]);
    }

    /// `OpReturn`.
    pub fn volver(&mut self) {
        instruccion(&mut self.cuerpo, op::RETURN, &[]);
    }

    /// Cierra la función (`OpFunctionEnd`) y la añade al módulo. Devuelve su `Id`.
    #[must_use = "el Id de la función hace falta para su punto de entrada"]
    pub fn terminar(mut self) -> Id {
        instruccion(&mut self.cuerpo, op::FUNCTION_END, &[]);
        self.modulo.funciones.append(&mut self.cuerpo);
        self.id
    }
}

impl Funcion<'_> {
    /// `OpCompositeConstruct`: un vector (o estructura) a partir de sus partes.
    pub fn construir(&mut self, tipo: Id, partes: &[Id]) -> Id {
        let operandos: Vec<u32> = partes.iter().map(|p| p.0).collect();
        self.con_resultado(op::COMPOSITE_CONSTRUCT, tipo, &operandos)
    }

    /// `OpCompositeExtract`: la componente `indice` de un vector.
    pub fn extraer(&mut self, tipo: Id, compuesto: Id, indice: u32) -> Id {
        self.con_resultado(op::COMPOSITE_EXTRACT, tipo, &[compuesto.0, indice])
    }

    /// `OpVectorShuffle`: un vector nuevo con las componentes elegidas de `a` y `b`
    /// (las de `b` cuentan a partir del final de `a`). Por ejemplo, `.xyz`.
    pub fn barajar(&mut self, tipo: Id, a: Id, b: Id, componentes: &[u32]) -> Id {
        let mut operandos = alloc::vec![a.0, b.0];
        operandos.extend_from_slice(componentes);
        self.con_resultado(op::VECTOR_SHUFFLE, tipo, &operandos)
    }

    /// `OpExtInst`: una instrucción de `GLSL.std.450` (`normalize`, `max`, `mix`…).
    pub fn glsl(&mut self, tipo: Id, instruccion_glsl: u32, argumentos: &[Id]) -> Id {
        let biblioteca = self.modulo.glsl();
        let mut operandos = alloc::vec![biblioteca.0, instruccion_glsl];
        operandos.extend(argumentos.iter().map(|a| a.0));
        self.con_resultado(op::EXT_INST, tipo, &operandos)
    }
}
