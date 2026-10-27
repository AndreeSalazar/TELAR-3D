//! Emisión del IR validado mediante hilo-spirv; no conoce Vulkan ni hace FFI.
use crate::{Operacion, Programa, Tipo};
use alloc::vec::Vec;
use hilo_spirv::{Funcion, Id as IdSpirv, op};

mod interfaz;
use interfaz::Interfaz;

struct Emisor {
    real: IdSpirv,
    vector: IdSpirv,
    booleano: IdSpirv,
    entradas: [IdSpirv; 4],
}

impl Programa {
    /// Emite un kernel por punto con grupo de 64 hilos y el ABI de rejilla.
    /// Solo escribe altura y normal; conserva x/z, índices y control.
    #[must_use]
    pub fn compilar(&self) -> Vec<u32> {
        let mut interfaz = Interfaz::nueva();
        let [cero, uno, dos, tres, seis, nueve] = [0, 1, 2, 3, 6, 9].map(|n| interfaz.modulo.constante_u32(n));
        // Separar el módulo permite usar los metadatos de la interfaz durante la emisión.
        let mut modulo = core::mem::take(&mut interfaz.modulo);
        let mut funcion = modulo.funcion(interfaz.void, interfaz.tipo_main);
        let (entrada, cuerpo, fin) = (funcion.etiqueta(), funcion.etiqueta(), funcion.etiqueta());
        funcion.bloque(entrada);
        let direccion = funcion.acceso(interfaz.entrada, interfaz.global, &[cero]);
        let indice = funcion.cargar(interfaz.entero, direccion);
        let puntos = interfaz.leer(&mut funcion, 3, nueve);
        let dentro = funcion.binaria(op::U_LESS_THAN, interfaz.booleano, indice, puntos);
        funcion.fusion(fin);
        funcion.saltar_si(dentro, cuerpo, fin);
        funcion.bloque(cuerpo);
        let base = funcion.binaria(op::I_MUL, interfaz.entero, indice, seis);
        let posicion = cargar_vector(&interfaz, &mut funcion, base);
        let normal_base = funcion.binaria(op::I_ADD, interfaz.entero, base, tres);
        let normal = cargar_vector(&interfaz, &mut funcion, normal_base);
        let fase = interfaz.leer(&mut funcion, 0, uno);
        let fase = funcion.unaria(op::BITCAST, interfaz.real, fase);
        let amplitud = interfaz.leer(&mut funcion, 0, dos);
        let amplitud = funcion.unaria(op::BITCAST, interfaz.real, amplitud);
        let emisor = Emisor {
            real: interfaz.real,
            vector: interfaz.vector,
            booleano: interfaz.booleano,
            entradas: [posicion, normal, fase, amplitud],
        };
        let mut valores = Vec::with_capacity(self.operaciones.len());
        for (indice, operacion) in self.operaciones.iter().enumerate() {
            valores.push(emisor.emitir(&mut funcion, *operacion, &valores, &self.tipos, self.tipos[indice]));
        }
        let altura = funcion.binaria(op::I_ADD, interfaz.entero, base, uno);
        guardar(&interfaz, &mut funcion, altura, valores[self.salida.altura.0]);
        for componente in 0..3 {
            let indice = funcion.modulo().constante_u32(componente);
            let direccion = funcion.binaria(op::I_ADD, interfaz.entero, normal_base, indice);
            let valor = funcion.extraer(interfaz.real, valores[self.salida.normal.0], componente);
            guardar(&interfaz, &mut funcion, direccion, valor);
        }
        funcion.saltar(fin);
        funcion.bloque(fin);
        funcion.volver();
        let main = funcion.terminar();
        modulo.entrada_calculo(main, "main", &[interfaz.global], [64, 1, 1]);
        modulo.ensamblar()
    }
}

fn cargar_vector(interfaz: &Interfaz, funcion: &mut Funcion, base: IdSpirv) -> IdSpirv {
    let valores = [0, 1, 2].map(|componente| {
        let constante = funcion.modulo().constante_u32(componente);
        let indice = funcion.binaria(op::I_ADD, interfaz.entero, base, constante);
        interfaz.leer(funcion, 1, indice)
    });
    funcion.construir(interfaz.vector, &valores)
}

fn guardar(interfaz: &Interfaz, funcion: &mut Funcion, indice: IdSpirv, valor: IdSpirv) {
    let cero = funcion.modulo().constante_u32(0);
    let direccion = funcion.acceso(interfaz.punteros[1], interfaz.buffers[1], &[cero, indice]);
    // GPU: la envoltura comprueba indice_global < puntos. Cada hilo posee sus
    // seis componentes; solo escribe y/normal, sin accesos a otros puntos ni atómicos.
    funcion.guardar(direccion, valor);
}

impl Emisor {
    fn emitir(
        &self,
        funcion: &mut Funcion,
        operacion: Operacion,
        valores: &[IdSpirv],
        tipos: &[Tipo],
        resultado: Tipo,
    ) -> IdSpirv {
        let valor = |id: crate::Id| valores[id.0];
        let tipo = if resultado == Tipo::Escalar { self.real } else { self.vector };
        match operacion {
            Operacion::Escalar(v) => funcion.modulo().constante_f32(v),
            Operacion::Vector(v) => {
                let componentes = v.map(|v| funcion.modulo().constante_f32(v));
                funcion.construir(self.vector, &componentes)
            }
            Operacion::Posicion => self.entradas[0],
            Operacion::Normal => self.entradas[1],
            Operacion::Fase => self.entradas[2],
            Operacion::Amplitud => self.entradas[3],
            Operacion::Sumar(a, b) => funcion.binaria(op::F_ADD, tipo, valor(a), valor(b)),
            Operacion::Restar(a, b) => funcion.binaria(op::F_SUB, tipo, valor(a), valor(b)),
            Operacion::Multiplicar(a, b) => match (tipos[a.0], tipos[b.0]) {
                (Tipo::Vector, Tipo::Escalar) => funcion.binaria(op::VECTOR_TIMES_SCALAR, tipo, valor(a), valor(b)),
                (Tipo::Escalar, Tipo::Vector) => funcion.binaria(op::VECTOR_TIMES_SCALAR, tipo, valor(b), valor(a)),
                _ => funcion.binaria(op::F_MUL, tipo, valor(a), valor(b)),
            },
            Operacion::Extraer(id, componente) => funcion.extraer(self.real, valor(id), componente),
            Operacion::Componer(ids) => funcion.construir(self.vector, &ids.map(valor)),
            Operacion::ProductoEscalar(a, b) => funcion.binaria(op::DOT, self.real, valor(a), valor(b)),
            Operacion::Normalizar(id) => self.normalizar(funcion, valor(id)),
        }
    }

    fn normalizar(&self, funcion: &mut Funcion, vector: IdSpirv) -> IdSpirv {
        let componentes = [0, 1, 2].map(|i| funcion.extraer(self.real, vector, i));
        let absolutos = componentes.map(|v| funcion.glsl(self.real, op::GLSL_ABSOLUTO, &[v]));
        let escala = funcion.glsl(self.real, op::GLSL_MAXIMO, &absolutos[..2]);
        let escala = funcion.glsl(self.real, op::GLSL_MAXIMO, &[escala, absolutos[2]]);
        let cero = funcion.modulo().constante_f32(0.0);
        let uno = funcion.modulo().constante_f32(1.0);
        let no_nulo = funcion.binaria(op::F_ORD_GREATER_THAN, self.booleano, escala, cero);
        let divisor = funcion.elegir(self.real, no_nulo, escala, uno);
        let divisor = funcion.construir(self.vector, &[divisor; 3]);
        let escalado = funcion.binaria(op::F_DIV, self.vector, vector, divisor);
        let arriba = funcion.construir(self.vector, &[cero, uno, cero]);
        let vector_bool = funcion.modulo().tipo_vector(self.booleano, 3);
        let mascara = funcion.construir(vector_bool, &[no_nulo; 3]);
        let seguro = funcion.elegir(self.vector, mascara, escalado, arriba);
        funcion.glsl(self.vector, op::GLSL_NORMALIZAR, &[seguro])
    }
}
