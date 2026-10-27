//! Bucle estructurado acotado con acumulador SSA; no escribe memoria compartida.
use crate::{Funcion, Id, modulo::instruccion, op};

impl Funcion<'_> {
    /// Repite un cuerpo un número fijo de veces, pasando índice y acumulador.
    /// El cuerpo debe terminar en su bloque actual, sin return ni saltos pendientes.
    pub fn repetir(
        &mut self,
        cantidad: u32,
        tipo: Id,
        inicial: Id,
        mut cuerpo: impl FnMut(&mut Self, Id, Id) -> Id,
    ) -> Id {
        let entero = self.modulo.tipo_u32();
        let booleano = self.modulo.tipo_bool();
        let cero = self.modulo.constante_u32(0);
        let uno = self.modulo.constante_u32(1);
        let limite = self.modulo.constante_u32(cantidad);
        let [previo, cabecera, trabajo, continuar, fin] = core::array::from_fn(|_| self.etiqueta());
        let [indice, valor, siguiente, acumulado] = core::array::from_fn(|_| self.modulo.id());
        self.saltar(previo);
        self.bloque(previo);
        self.saltar(cabecera);
        self.bloque(cabecera);
        // OpPhi: los valores del bloque de continuación se definen más abajo.
        instruccion(&mut self.cuerpo, 245, &[entero.0, indice.0, cero.0, previo.0, siguiente.0, continuar.0]);
        instruccion(&mut self.cuerpo, 245, &[tipo.0, valor.0, inicial.0, previo.0, acumulado.0, continuar.0]);
        let dentro = self.binaria(op::U_LESS_THAN, booleano, indice, limite);
        instruccion(&mut self.cuerpo, 246, &[fin.0, continuar.0, 0]); // OpLoopMerge.
        self.saltar_si(dentro, trabajo, fin);
        self.bloque(trabajo);
        let resultado = cuerpo(self, indice, valor);
        self.saltar(continuar);
        self.bloque(continuar);
        instruccion(&mut self.cuerpo, op::I_ADD, &[entero.0, siguiente.0, indice.0, uno.0]);
        instruccion(&mut self.cuerpo, 83, &[tipo.0, acumulado.0, resultado.0]); // OpCopyObject.
        self.saltar(cabecera);
        self.bloque(fin);
        valor
    }
}
