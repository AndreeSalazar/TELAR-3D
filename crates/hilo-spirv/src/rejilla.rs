//! Tres kernels residentes: preparar argumentos, generar la rejilla y deformarla.
//! ABI: enlaces 0 parámetros u32[4], 1 vértices f32[], 2 índices u32[], 3 control
//! u32[16]. Control: deformación[0..3], draw[3..8], lado[8], puntos[9], error[10],
//! último lado generado[11], generación[12..15], generaciones acumuladas[15].
//! Los lados superiores a la capacidad producen trabajo cero y error 1.

use crate::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
use alloc::vec::Vec;

mod operaciones;
pub use operaciones::escalar_altura;

struct Entorno {
    modulo: Modulo,
    entero: Id,
    real: Id,
    booleano: Id,
    void: Id,
    funcion: Id,
    puntero: [Id; 4],
    buffers: [Id; 4],
    global: Id,
    entrada: Id,
}

impl Entorno {
    fn nuevo() -> Self {
        let mut modulo = Modulo::nuevo();
        let entero = modulo.tipo_u32();
        let real = modulo.tipo_f32();
        let booleano = modulo.tipo_bool();
        let void = modulo.tipo_void();
        let funcion = modulo.tipo_funcion(void, &[]);
        let vector = modulo.tipo_vector(entero, 3);
        let entrada = modulo.tipo_puntero(Clase::Entrada, entero);
        let pv = modulo.tipo_puntero(Clase::Entrada, vector);
        let global = modulo.variable(pv, Clase::Entrada);
        modulo.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
        let tipos = [entero, real, entero, entero];
        let puntero = tipos.map(|tipo| modulo.tipo_puntero(Clase::Almacen, tipo));
        let buffers = core::array::from_fn(|indice| {
            let array = modulo.tipo_array_dinamico(tipos[indice], 4);
            let bloque = modulo.tipo_bloque(&[array], &[0]);
            let pb = modulo.tipo_puntero(Clase::Almacen, bloque);
            let variable = modulo.variable(pb, Clase::Almacen);
            modulo.decorar(variable, op::DECORACION_CONJUNTO, &[0]);
            modulo.decorar(variable, op::DECORACION_ENLACE, &[u32::try_from(indice).unwrap_or(0)]);
            variable
        });
        Self { modulo, entero, real, booleano, void, funcion, puntero, buffers, global, entrada }
    }
}

fn leer(funcion: &mut Funcion, puntero: Id, buffer: Id, tipo: Id, indice: Id) -> Id {
    let cero = funcion.modulo().constante_u32(0);
    let direccion = funcion.acceso(puntero, buffer, &[cero, indice]);
    funcion.cargar(tipo, direccion)
}

fn escribir(funcion: &mut Funcion, puntero: Id, buffer: Id, indice: Id, valor: Id) {
    let cero = funcion.modulo().constante_u32(0);
    let direccion = funcion.acceso(puntero, buffer, &[cero, indice]);
    // GPU: el productor único, o el hilo dueño del índice, escribe; las pasadas
    // consumidoras quedan separadas por barreras Vulkan fuera del shader.
    funcion.guardar(direccion, valor);
}

/// Un solo hilo determina recuentos y argumentos, con capacidad y vacío explícitos.
#[must_use]
pub fn preparar() -> Vec<u32> {
    let mut entorno = Entorno::nuevo();
    let [cero, uno, dos, tres, seis, ocho, nueve, diez, sesenta_y_tres, sesenta_y_cuatro] =
        [0, 1, 2, 3, 6, 8, 9, 10, 63, 64].map(|n| entorno.modulo.constante_u32(n));
    let mut funcion = entorno.modulo.funcion(entorno.void, entorno.funcion);
    let bloque = funcion.etiqueta();
    funcion.bloque(bloque);
    let lado = leer(&mut funcion, entorno.puntero[0], entorno.buffers[0], entorno.entero, cero);
    // La capacidad exacta llega en el cuarto parámetro, verificada por el ejecutor.
    let capacidad = leer(&mut funcion, entorno.puntero[0], entorno.buffers[0], entorno.entero, tres);
    let excede = funcion.binaria(op::U_LESS_THAN, entorno.booleano, capacidad, lado);
    let suficiente = funcion.binaria(op::U_LESS_THAN, entorno.booleano, uno, lado);
    let falso = funcion.binaria(op::U_LESS_THAN, entorno.booleano, cero, cero);
    let permitido = funcion.elegir(entorno.booleano, excede, falso, suficiente);
    let lado_seguro = funcion.elegir(entorno.entero, permitido, lado, uno);
    let cuadrado = funcion.binaria(op::I_MUL, entorno.entero, lado_seguro, lado_seguro);
    let puntos = funcion.elegir(entorno.entero, permitido, cuadrado, cero);
    let borde = funcion.binaria(op::I_SUB, entorno.entero, lado_seguro, uno);
    let celdas = funcion.binaria(op::I_MUL, entorno.entero, borde, borde);
    let indices = funcion.binaria(op::I_MUL, entorno.entero, celdas, seis);
    let redondeo = funcion.binaria(op::I_ADD, entorno.entero, puntos, sesenta_y_tres);
    let grupos = funcion.binaria(op::U_DIV, entorno.entero, redondeo, sesenta_y_cuatro);
    cache_gpu(
        &mut funcion,
        entorno.entero,
        entorno.booleano,
        entorno.puntero[3],
        entorno.buffers[3],
        lado,
        permitido,
        grupos,
    );
    let error = funcion.elegir(entorno.entero, excede, uno, cero);
    let valores =
        [(cero, grupos), (uno, uno), (dos, uno), (tres, indices), (ocho, lado_seguro), (nueve, puntos), (diez, error)];
    for (indice, valor) in valores {
        escribir(&mut funcion, entorno.puntero[3], entorno.buffers[3], indice, valor);
    }
    for (indice, valor) in [(4, uno), (5, cero), (6, cero), (7, cero)] {
        let indice = funcion.modulo().constante_u32(indice);
        escribir(&mut funcion, entorno.puntero[3], entorno.buffers[3], indice, valor);
    }
    funcion.volver();
    let main = funcion.terminar();
    entorno.modulo.entrada_calculo(main, "main", &[], [1, 1, 1]);
    entorno.modulo.ensamblar()
}

/// El único hilo productor conserva la geometría hasta cambiar de dimensiones.
#[allow(clippy::too_many_arguments, reason = "ids SPIR-V del entorno compartido por el productor")]
fn cache_gpu(
    funcion: &mut Funcion,
    entero: Id,
    booleano: Id,
    puntero: Id,
    control: Id,
    lado: Id,
    permitido: Id,
    grupos: Id,
) {
    let [cero, uno, anterior, despacho, contador] = [0, 1, 11, 12, 15].map(|n| funcion.modulo().constante_u32(n));
    let previo = leer(funcion, puntero, control, entero, anterior);
    let total = leer(funcion, puntero, control, entero, contador);
    let cambio = funcion.binaria(op::I_NOT_EQUAL, booleano, lado, previo);
    let generar = funcion.binaria(op::LOGICAL_AND, booleano, permitido, cambio);
    let cantidad = funcion.elegir(entero, generar, grupos, cero);
    let incremento = funcion.elegir(entero, generar, uno, cero);
    let total = funcion.binaria(op::I_ADD, entero, total, incremento);
    let ultimo = funcion.elegir(entero, permitido, lado, previo);
    for (indice, valor) in [(anterior, ultimo), (despacho, cantidad), (contador, total)] {
        escribir(funcion, puntero, control, indice, valor);
    }
    for indice in [13, 14] {
        let indice = funcion.modulo().constante_u32(indice);
        escribir(funcion, puntero, control, indice, uno);
    }
}

/// Genera puntos, normales planas e índices con una escritura exclusiva por hilo.
#[must_use]
pub fn generar() -> Vec<u32> {
    kernel(false)
}

/// Deforma la altura y calcula la normal analítica de una onda por vértice.
#[must_use]
pub fn deformar() -> Vec<u32> {
    kernel(true)
}

fn kernel(deformar: bool) -> Vec<u32> {
    let mut entorno = Entorno::nuevo();
    let [cero, uno, seis, ocho, nueve] = [0, 1, 6, 8, 9].map(|n| entorno.modulo.constante_u32(n));
    let mut funcion = entorno.modulo.funcion(entorno.void, entorno.funcion);
    let (entrada, cuerpo, fin) = (funcion.etiqueta(), funcion.etiqueta(), funcion.etiqueta());
    funcion.bloque(entrada);
    let direccion = funcion.acceso(entorno.entrada, entorno.global, &[cero]);
    let indice = funcion.cargar(entorno.entero, direccion);
    let puntos = leer(&mut funcion, entorno.puntero[3], entorno.buffers[3], entorno.entero, nueve);
    let dentro = funcion.binaria(op::U_LESS_THAN, entorno.booleano, indice, puntos);
    funcion.fusion(fin);
    funcion.saltar_si(dentro, cuerpo, fin);
    funcion.bloque(cuerpo);
    let lado = leer(&mut funcion, entorno.puntero[3], entorno.buffers[3], entorno.entero, ocho);
    let base = funcion.binaria(op::I_MUL, entorno.entero, indice, seis);
    if deformar {
        onda(&mut funcion, &entorno.puntero, &entorno.buffers, entorno.entero, entorno.real, base);
    } else {
        let columna = funcion.binaria(op::U_MOD, entorno.entero, indice, lado);
        let fila = funcion.binaria(op::U_DIV, entorno.entero, indice, lado);
        let borde = funcion.binaria(op::I_SUB, entorno.entero, lado, uno);
        let divisor = funcion.unaria(op::U_A_F, entorno.real, borde);
        let coordenadas = [columna, fila].map(|valor| {
            let valor = funcion.unaria(op::U_A_F, entorno.real, valor);
            let fraccion = funcion.binaria(op::F_DIV, entorno.real, valor, divisor);
            let cuatro = funcion.modulo().constante_f32(4.0);
            let dos = funcion.modulo().constante_f32(2.0);
            let ancho = funcion.binaria(op::F_MUL, entorno.real, fraccion, cuatro);
            funcion.binaria(op::F_SUB, entorno.real, ancho, dos)
        });
        let nulo = funcion.modulo().constante_f32(0.0);
        let arriba = funcion.modulo().constante_f32(1.0);
        guardar_vertice(
            &mut funcion,
            entorno.puntero[1],
            entorno.buffers[1],
            entorno.entero,
            base,
            [coordenadas[0], nulo, coordenadas[1], nulo, arriba, nulo],
        );
        let cx = funcion.binaria(op::U_LESS_THAN, entorno.booleano, columna, borde);
        let cz = funcion.binaria(op::U_LESS_THAN, entorno.booleano, fila, borde);
        let celda = funcion.binaria(op::LOGICAL_AND, entorno.booleano, cx, cz);
        let (indices, despues) = (funcion.etiqueta(), funcion.etiqueta());
        funcion.fusion(despues);
        funcion.saltar_si(celda, indices, despues);
        funcion.bloque(indices);
        let fila_celda = funcion.binaria(op::I_MUL, entorno.entero, fila, borde);
        let celda_id = funcion.binaria(op::I_ADD, entorno.entero, fila_celda, columna);
        let inicio = funcion.binaria(op::I_MUL, entorno.entero, celda_id, seis);
        let derecha = funcion.binaria(op::I_ADD, entorno.entero, indice, uno);
        let abajo = funcion.binaria(op::I_ADD, entorno.entero, indice, lado);
        let diagonal = funcion.binaria(op::I_ADD, entorno.entero, abajo, uno);
        for (desplazamiento, valor) in (0..6).zip([indice, abajo, derecha, derecha, abajo, diagonal]) {
            let constante = funcion.modulo().constante_u32(desplazamiento);
            let destino = funcion.binaria(op::I_ADD, entorno.entero, inicio, constante);
            escribir(&mut funcion, entorno.puntero[2], entorno.buffers[2], destino, valor);
        }
        funcion.saltar(despues);
        funcion.bloque(despues);
    }
    funcion.saltar(fin);
    funcion.bloque(fin);
    funcion.volver();
    let main = funcion.terminar();
    entorno.modulo.entrada_calculo(main, "main", &[entorno.global], [64, 1, 1]);
    entorno.modulo.ensamblar()
}

fn guardar_vertice(funcion: &mut Funcion, puntero: Id, buffer: Id, entero: Id, base: Id, valores: [Id; 6]) {
    for (indice, valor) in (0..6).zip(valores) {
        let constante = funcion.modulo().constante_u32(indice);
        let destino = funcion.binaria(op::I_ADD, entero, base, constante);
        escribir(funcion, puntero, buffer, destino, valor);
    }
}

fn onda(funcion: &mut Funcion, punteros: &[Id; 4], buffers: &[Id; 4], entero: Id, real: Id, base: Id) {
    let [uno, dos] = [1, 2].map(|n| funcion.modulo().constante_u32(n));
    let fase = leer(funcion, punteros[0], buffers[0], entero, uno);
    let amplitud = leer(funcion, punteros[0], buffers[0], entero, dos);
    let fase = funcion.unaria(op::BITCAST, real, fase);
    let amplitud = funcion.unaria(op::BITCAST, real, amplitud);
    let horizontal = leer(funcion, punteros[1], buffers[1], real, base);
    let direccion_z = funcion.binaria(op::I_ADD, entero, base, dos);
    let profundidad = leer(funcion, punteros[1], buffers[1], real, direccion_z);
    let doble = funcion.modulo().constante_f32(2.0);
    let eje_x = funcion.binaria(op::F_MUL, real, horizontal, doble);
    let eje_x = funcion.binaria(op::F_ADD, real, eje_x, fase);
    let eje_z = funcion.binaria(op::F_MUL, real, profundidad, doble);
    let seno_x = funcion.glsl(real, 13, &[eje_x]);
    let coseno_x = funcion.glsl(real, 14, &[eje_x]);
    let seno_z = funcion.glsl(real, 13, &[eje_z]);
    let coseno_z = funcion.glsl(real, 14, &[eje_z]);
    let altura = funcion.binaria(op::F_MUL, real, seno_x, coseno_z);
    let altura = funcion.binaria(op::F_MUL, real, altura, amplitud);
    let escala = funcion.binaria(op::F_MUL, real, doble, amplitud);
    let nx = funcion.binaria(op::F_MUL, real, coseno_x, coseno_z);
    let nx = funcion.binaria(op::F_MUL, real, nx, escala);
    let nulo = funcion.modulo().constante_f32(0.0);
    let nx = funcion.binaria(op::F_SUB, real, nulo, nx);
    let nz = funcion.binaria(op::F_MUL, real, seno_x, seno_z);
    let nz = funcion.binaria(op::F_MUL, real, nz, escala);
    let arriba = funcion.modulo().constante_f32(1.0);
    let vector = funcion.modulo().tipo_vector(real, 3);
    let normal = funcion.construir(vector, &[nx, arriba, nz]);
    let normal = funcion.glsl(vector, op::GLSL_NORMALIZAR, &[normal]);
    let normales = [0, 1, 2].map(|indice| funcion.extraer(real, normal, indice));
    guardar_vertice(
        funcion,
        punteros[1],
        buffers[1],
        entero,
        base,
        [horizontal, altura, profundidad, normales[0], normales[1], normales[2]],
    );
}
