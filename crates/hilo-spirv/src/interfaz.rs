//! Interfaz 2D con cobertura tipográfica y mezcla alfa. El atlas reside en un
//! SSBO inmutable: muestreo bilineal explícito y límites comprobados en shader.
use crate::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
use alloc::vec::Vec;

fn variable(modulo: &mut Modulo, tipo: Id, clase: Clase, ubicacion: u32) -> Id {
    let puntero = modulo.tipo_puntero(clase, tipo);
    let variable = modulo.variable(puntero, clase);
    modulo.decorar(variable, op::DECORACION_UBICACION, &[ubicacion]);
    variable
}

/// Posición 2D, coordenada de atlas y color RGBA, con 32 bytes por vértice.
#[must_use]
pub fn vertices() -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let real = modulo.tipo_f32();
    let vector2 = modulo.tipo_vector(real, 2);
    let vector4 = modulo.tipo_vector(real, 4);
    let posicion = variable(&mut modulo, vector2, Clase::Entrada, 0);
    let uv = variable(&mut modulo, vector2, Clase::Entrada, 1);
    let color = variable(&mut modulo, vector4, Clase::Entrada, 2);
    let salida = variable(&mut modulo, vector4, Clase::Salida, 0);
    let salida_uv = variable(&mut modulo, vector2, Clase::Salida, 1);
    let puntero = modulo.tipo_puntero(Clase::Salida, vector4);
    let clip = modulo.variable(puntero, Clase::Salida);
    modulo.decorar(clip, op::DECORACION_INTEGRADA, &[op::INTEGRADA_POSICION]);
    let cero = modulo.constante_f32(0.0);
    let uno = modulo.constante_f32(1.0);
    let void = modulo.tipo_void();
    let tipo = modulo.tipo_funcion(void, &[]);
    let mut funcion = modulo.funcion(void, tipo);
    let bloque = funcion.etiqueta();
    funcion.bloque(bloque);
    let valor_posicion = funcion.cargar(vector2, posicion);
    let x = funcion.extraer(real, valor_posicion, 0);
    let y = funcion.extraer(real, valor_posicion, 1);
    let valor = funcion.construir(vector4, &[x, y, cero, uno]);
    funcion.guardar(clip, valor);
    let valor = funcion.cargar(vector4, color);
    funcion.guardar(salida, valor);
    let valor = funcion.cargar(vector2, uv);
    funcion.guardar(salida_uv, valor);
    funcion.volver();
    let main = funcion.terminar();
    modulo.entrada(op::MODELO_VERTICES, main, "main", &[posicion, uv, color, salida, salida_uv, clip]);
    modulo.ensamblar()
}

struct Atlas {
    entero: Id,
    real: Id,
    buffer: Id,
    puntero: Id,
}

impl Atlas {
    fn leer(&self, funcion: &mut Funcion, indice: Id) -> Id {
        let cero = funcion.modulo().constante_u32(0);
        let direccion = funcion.acceso(self.puntero, self.buffer, &[cero, indice]);
        funcion.cargar(self.entero, direccion)
    }

    fn cobertura(&self, funcion: &mut Funcion, ancho: Id, x: Id, y: Id) -> Id {
        let dos = funcion.modulo().constante_u32(2);
        let fila = funcion.binaria(op::I_MUL, self.entero, y, ancho);
        let indice = funcion.binaria(op::I_ADD, self.entero, fila, x);
        let indice = funcion.binaria(op::I_ADD, self.entero, indice, dos);
        // GPU: x/y se limitan a las dimensiones; el atlas tiene 2+ancho*alto u32,
        // se sube antes del dibujo y no se modifica durante la lectura.
        let valor = self.leer(funcion, indice);
        let valor = funcion.unaria(op::U_A_F, self.real, valor);
        let escala = funcion.modulo().constante_f32(1.0 / 255.0);
        funcion.binaria(op::F_MUL, self.real, valor, escala)
    }

    fn eje(&self, funcion: &mut Funcion, uv: Id, dimension: Id) -> (Id, Id, Id) {
        let uno = funcion.modulo().constante_u32(1);
        let cero = funcion.modulo().constante_f32(0.0);
        let medio = funcion.modulo().constante_f32(0.5);
        let maximo = funcion.binaria(op::I_SUB, self.entero, dimension, uno);
        let limite = funcion.unaria(op::U_A_F, self.real, maximo);
        let uv = funcion.binaria(op::F_SUB, self.real, uv, medio);
        // GLSL FClamp=43; el atlas se valida con dimensiones positivas antes de subir.
        let uv = funcion.glsl(self.real, 43, &[uv, cero, limite]);
        let origen = funcion.unaria(op::F_A_U, self.entero, uv);
        let origen_real = funcion.unaria(op::U_A_F, self.real, origen);
        let fraccion = funcion.binaria(op::F_SUB, self.real, uv, origen_real);
        let siguiente = funcion.binaria(op::I_ADD, self.entero, origen, uno);
        // GLSL UMin=38, ambos límites son enteros sin signo.
        let siguiente = funcion.glsl(self.entero, 38, &[siguiente, maximo]);
        (origen, siguiente, fraccion)
    }
}

/// Color y cobertura bilineal del atlas; Vulkan mezcla el resultado con el fondo.
#[must_use]
pub fn fragmentos() -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let real = modulo.tipo_f32();
    let entero = modulo.tipo_u32();
    let vector2 = modulo.tipo_vector(real, 2);
    let vector4 = modulo.tipo_vector(real, 4);
    let entrada = variable(&mut modulo, vector4, Clase::Entrada, 0);
    let uv_entrada = variable(&mut modulo, vector2, Clase::Entrada, 1);
    let salida = variable(&mut modulo, vector4, Clase::Salida, 0);
    let array = modulo.tipo_array_dinamico(entero, 4);
    let bloque = modulo.tipo_bloque(&[array], &[0]);
    let puntero_bloque = modulo.tipo_puntero(Clase::Almacen, bloque);
    let buffer = modulo.variable(puntero_bloque, Clase::Almacen);
    modulo.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
    modulo.decorar(buffer, op::DECORACION_ENLACE, &[0]);
    // NonWritable=24: la etapa de fragmentos solo lee la máscara tipográfica.
    modulo.decorar(buffer, 24, &[]);
    let puntero = modulo.tipo_puntero(Clase::Almacen, entero);
    let atlas = Atlas { entero, real, buffer, puntero };
    let [cero, uno] = [0, 1].map(|n| modulo.constante_u32(n));
    let void = modulo.tipo_void();
    let tipo = modulo.tipo_funcion(void, &[]);
    let mut funcion = modulo.funcion(void, tipo);
    let bloque = funcion.etiqueta();
    funcion.bloque(bloque);
    let ancho = atlas.leer(&mut funcion, cero);
    let alto = atlas.leer(&mut funcion, uno);
    let uv = funcion.cargar(vector2, uv_entrada);
    let horizontal = funcion.extraer(real, uv, 0);
    let vertical = funcion.extraer(real, uv, 1);
    let (x0, x1, fx) = atlas.eje(&mut funcion, horizontal, ancho);
    let (y0, y1, fy) = atlas.eje(&mut funcion, vertical, alto);
    let arriba_izquierda = atlas.cobertura(&mut funcion, ancho, x0, y0);
    let arriba_derecha = atlas.cobertura(&mut funcion, ancho, x1, y0);
    let abajo_izquierda = atlas.cobertura(&mut funcion, ancho, x0, y1);
    let abajo_derecha = atlas.cobertura(&mut funcion, ancho, x1, y1);
    let superior = funcion.glsl(real, op::GLSL_MEZCLA, &[arriba_izquierda, arriba_derecha, fx]);
    let inferior = funcion.glsl(real, op::GLSL_MEZCLA, &[abajo_izquierda, abajo_derecha, fx]);
    let cobertura = funcion.glsl(real, op::GLSL_MEZCLA, &[superior, inferior, fy]);
    let color = funcion.cargar(vector4, entrada);
    let componentes = [0, 1, 2, 3].map(|i| funcion.extraer(real, color, i));
    let alfa = funcion.binaria(op::F_MUL, real, componentes[3], cobertura);
    let color = funcion.construir(vector4, &[componentes[0], componentes[1], componentes[2], alfa]);
    funcion.guardar(salida, color);
    funcion.volver();
    let main = funcion.terminar();
    modulo.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, uv_entrada, salida]);
    modulo.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    modulo.ensamblar()
}
