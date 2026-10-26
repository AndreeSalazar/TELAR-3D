//! El constructor de módulos SPIR-V: cada instrucción es una palabra de
//! cabecera (`palabras << 16 | código`) seguida de sus operandos. Las secciones
//! se guardan aparte y se juntan al ensamblar, en el orden que exige la
//! especificación (2.4, «Logical Layout of a Module»).

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::op::{self, Clase};

/// El identificador de algo del módulo (`<id>` en la especificación).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub u32);

/// Un módulo SPIR-V en construcción.
pub struct Modulo {
    siguiente: u32,
    capacidades: Vec<u32>,
    memoria: Vec<u32>,
    entradas: Vec<u32>,
    modos: Vec<u32>,
    nombres: Vec<u32>,
    anotaciones: Vec<u32>,
    /// Tipos, constantes y variables globales: comparten sección, en orden de declaración.
    globales: Vec<u32>,
    funciones: Vec<u32>,
    /// Tipos y constantes ya declarados: el mismo tipo, el mismo `Id`.
    unicos: BTreeMap<Vec<u32>, Id>,
}

/// Escribe una instrucción en `destino`.
fn instruccion(destino: &mut Vec<u32>, codigo: u16, operandos: &[u32]) {
    let palabras = u32::try_from(operandos.len() + 1).unwrap_or(u32::MAX);
    destino.push((palabras << 16) | u32::from(codigo));
    destino.extend_from_slice(operandos);
}

/// Un texto como literal SPIR-V: UTF-8, terminado en cero y relleno hasta 4 bytes.
#[must_use]
pub fn texto(s: &str) -> Vec<u32> {
    let mut bytes: Vec<u8> = s.bytes().collect();
    bytes.push(0);
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    bytes.as_chunks::<4>().0.iter().map(|&c| u32::from_le_bytes(c)).collect()
}

impl Default for Modulo {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl Modulo {
    /// Un módulo para shaders de Vulkan: `Capability Shader` y el modelo de
    /// memoria `Logical GLSL450`.
    #[must_use]
    pub fn nuevo() -> Self {
        let mut m = Self {
            siguiente: 1,
            capacidades: Vec::new(),
            memoria: Vec::new(),
            entradas: Vec::new(),
            modos: Vec::new(),
            nombres: Vec::new(),
            anotaciones: Vec::new(),
            globales: Vec::new(),
            funciones: Vec::new(),
            unicos: BTreeMap::new(),
        };
        instruccion(&mut m.capacidades, op::CAPABILITY, &[op::CAPACIDAD_SHADER]);
        instruccion(&mut m.memoria, op::MEMORY_MODEL, &[op::DIRECCIONES_LOGICAS, op::MEMORIA_GLSL450]);
        m
    }

    /// Un identificador nuevo.
    pub fn id(&mut self) -> Id {
        let id = Id(self.siguiente);
        self.siguiente += 1;
        id
    }

    /// Declara un tipo o una constante una sola vez: si ya existe, devuelve su `Id`.
    fn unico(&mut self, codigo: u16, operandos: &[u32]) -> Id {
        let mut clave = Vec::with_capacity(operandos.len() + 1);
        clave.push(u32::from(codigo));
        clave.extend_from_slice(operandos);
        if let Some(&id) = self.unicos.get(&clave) {
            return id;
        }
        let id = self.global(codigo, operandos);
        self.unicos.insert(clave, id);
        id
    }

    /// Declara algo global con un `Id` nuevo. Los tipos (`OpType*`) llevan el
    /// resultado primero; las constantes, primero su tipo y después el resultado
    /// (`OpConstant <tipo> <resultado> <valor>`), y aquí `operandos[0]` es ese tipo.
    fn global(&mut self, codigo: u16, operandos: &[u32]) -> Id {
        let id = self.id();
        let mut todo = Vec::with_capacity(operandos.len() + 1);
        if codigo == op::CONSTANT {
            todo.push(operandos[0]);
            todo.push(id.0);
            todo.extend_from_slice(&operandos[1..]);
        } else {
            todo.push(id.0);
            todo.extend_from_slice(operandos);
        }
        instruccion(&mut self.globales, codigo, &todo);
        id
    }

    /// `void`.
    pub fn tipo_void(&mut self) -> Id {
        self.unico(op::TYPE_VOID, &[])
    }

    /// `bool`.
    pub fn tipo_bool(&mut self) -> Id {
        self.unico(op::TYPE_BOOL, &[])
    }

    /// Entero de 32 bits sin signo.
    pub fn tipo_u32(&mut self) -> Id {
        self.unico(op::TYPE_INT, &[32, 0])
    }

    /// Vector de `n` elementos.
    pub fn tipo_vector(&mut self, elemento: Id, n: u32) -> Id {
        self.unico(op::TYPE_VECTOR, &[elemento.0, n])
    }

    /// Puntero a `tipo` en la clase `clase`.
    pub fn tipo_puntero(&mut self, clase: Clase, tipo: Id) -> Id {
        self.unico(op::TYPE_POINTER, &[clase.numero(), tipo.0])
    }

    /// Tipo de función: devuelve `retorno` y recibe `parametros`.
    pub fn tipo_funcion(&mut self, retorno: Id, parametros: &[Id]) -> Id {
        let mut operandos = alloc::vec![retorno.0];
        operandos.extend(parametros.iter().map(|p| p.0));
        self.unico(op::TYPE_FUNCTION, &operandos)
    }

    /// Array de tamaño dinámico (el de un buffer), con `paso` bytes entre elementos.
    /// No se comparte: cada uno lleva su propia decoración.
    pub fn tipo_array_dinamico(&mut self, elemento: Id, paso: u32) -> Id {
        let id = self.global(op::TYPE_RUNTIME_ARRAY, &[elemento.0]);
        self.decorar(id, op::DECORACION_PASO, &[paso]);
        id
    }

    /// La estructura de un buffer (`Block`), con el desplazamiento en bytes de cada miembro.
    pub fn tipo_bloque(&mut self, miembros: &[Id], desplazamientos: &[u32]) -> Id {
        let operandos: Vec<u32> = miembros.iter().map(|m| m.0).collect();
        let id = self.global(op::TYPE_STRUCT, &operandos);
        self.decorar(id, op::DECORACION_BLOQUE, &[]);
        for (indice, &desplazamiento) in (0u32..).zip(desplazamientos) {
            instruccion(
                &mut self.anotaciones,
                op::MEMBER_DECORATE,
                &[id.0, indice, op::DECORACION_DESPLAZAMIENTO, desplazamiento],
            );
        }
        id
    }

    /// Una constante `u32`.
    pub fn constante_u32(&mut self, valor: u32) -> Id {
        let tipo = self.tipo_u32();
        self.unico(op::CONSTANT, &[tipo.0, valor])
    }

    /// Una variable global de tipo `tipo_puntero`, en `clase`.
    pub fn variable(&mut self, tipo_puntero: Id, clase: Clase) -> Id {
        let id = self.id();
        instruccion(&mut self.globales, op::VARIABLE, &[tipo_puntero.0, id.0, clase.numero()]);
        id
    }

    /// `OpDecorate`: una decoración con sus literales.
    pub fn decorar(&mut self, id: Id, decoracion: u32, literales: &[u32]) {
        let mut operandos = alloc::vec![id.0, decoracion];
        operandos.extend_from_slice(literales);
        instruccion(&mut self.anotaciones, op::DECORATE, &operandos);
    }

    /// `OpName`: un nombre para depurar (lo enseña `spirv-dis`).
    pub fn nombre(&mut self, id: Id, nombre: &str) {
        let mut operandos = alloc::vec![id.0];
        operandos.extend(texto(nombre));
        instruccion(&mut self.nombres, op::NAME, &operandos);
    }

    /// Punto de entrada de cálculo (`GLCompute`) con su tamaño de grupo. En
    /// SPIR-V 1.3, la interfaz lista solo las variables de entrada y salida.
    pub fn entrada_calculo(&mut self, funcion: Id, nombre: &str, interfaz: &[Id], grupo: [u32; 3]) {
        let mut operandos = alloc::vec![op::MODELO_CALCULO, funcion.0];
        operandos.extend(texto(nombre));
        operandos.extend(interfaz.iter().map(|i| i.0));
        instruccion(&mut self.entradas, op::ENTRY_POINT, &operandos);
        instruccion(
            &mut self.modos,
            op::EXECUTION_MODE,
            &[funcion.0, op::MODO_TAMANO_GRUPO, grupo[0], grupo[1], grupo[2]],
        );
    }

    /// Empieza una función que devuelve `retorno` y es de tipo `tipo`.
    pub fn funcion(&mut self, retorno: Id, tipo: Id) -> Funcion<'_> {
        let id = self.id();
        let mut cuerpo = Vec::new();
        instruccion(&mut cuerpo, op::FUNCTION, &[retorno.0, id.0, op::CONTROL_NINGUNO, tipo.0]);
        Funcion { modulo: self, id, cuerpo }
    }

    /// El módulo terminado: cabecera y secciones, en el orden de la especificación.
    #[must_use]
    pub fn ensamblar(&self) -> Vec<u32> {
        let mut out = alloc::vec![op::MAGIA, op::VERSION_1_3, 0, self.siguiente, 0];
        for seccion in [
            &self.capacidades,
            &self.memoria,
            &self.entradas,
            &self.modos,
            &self.nombres,
            &self.anotaciones,
            &self.globales,
            &self.funciones,
        ] {
            out.extend_from_slice(seccion);
        }
        out
    }
}

/// Una función en construcción. Sus instrucciones van a su propio cuerpo; los
/// tipos y constantes que necesite se declaran en el módulo ([`Funcion::modulo`]).
pub struct Funcion<'a> {
    modulo: &'a mut Modulo,
    id: Id,
    cuerpo: Vec<u32>,
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
