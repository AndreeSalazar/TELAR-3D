//! Kernel acotado de imagen RGB HDR: una invocación posee un píxel completo.
use crate::{escena_rayos::Trabajo, trazado::Matematica};
use alloc::vec::Vec;
use hilo_spirv::{
    Modulo,
    op::{self, Clase},
};

pub(crate) fn emitir(trabajo: &Trabajo) -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let mat = Matematica::nueva(&mut m);
    let global_tipo = m.tipo_vector(mat.entero, 3);
    let entrada = m.tipo_puntero(Clase::Entrada, global_tipo);
    let global = m.variable(entrada, Clase::Entrada);
    m.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let array = m.tipo_array_dinamico(mat.entero, 4);
    let bloque = m.tipo_bloque(&[array], &[0]);
    let tipo = m.tipo_puntero(Clase::Almacen, bloque);
    let datos = m.variable(tipo, Clase::Almacen);
    m.decorar(datos, op::DECORACION_CONJUNTO, &[0]);
    m.decorar(datos, op::DECORACION_ENLACE, &[0]);
    let puntero = m.tipo_puntero(Clase::Almacen, mat.entero);
    let [cero, cinco, pixeles, muestras] =
        [0, 5, trabajo.tamano[0] * trabajo.tamano[1], trabajo.muestras].map(|v| m.constante_u32(v));
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let [entrada, cuerpo, fin] = core::array::from_fn(|_| f.etiqueta());
    f.bloque(entrada);
    let invocacion = f.cargar(global_tipo, global);
    let pixel = f.extraer(mat.entero, invocacion, 0);
    let longitud = f.longitud(mat.entero, datos, 0);
    let capacidad = f.binaria(op::U_DIV, mat.entero, longitud, cinco);
    let cabe = f.binaria(op::U_LESS_THAN, mat.booleano, pixel, capacidad);
    let activo = f.binaria(op::U_LESS_THAN, mat.booleano, pixel, pixeles);
    let activo = f.binaria(op::LOGICAL_AND, mat.booleano, activo, cabe);
    f.fusion(fin);
    f.saltar_si(activo, cuerpo, fin);
    f.bloque(cuerpo);
    let negro = mat.vector(&mut f, [0.0; 3]);
    let suma = f.repetir(trabajo.muestras, mat.vector, negro, |f, indice, acumulado| {
        let muestra = crate::muestra_rayos::muestra(f, &mat, trabajo, pixel, indice);
        f.binaria(op::F_ADD, mat.vector, acumulado, muestra)
    });
    #[allow(clippy::cast_precision_loss, reason = "muestras <=1024")]
    let peso = Matematica::escalar(&mut f, 1.0 / trabajo.muestras as f32);
    let media = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, suma, peso);
    let base = f.binaria(op::I_MUL, mat.entero, pixel, cinco);
    let revision = crate::color_rayos::convertir(&mut f, &mat, media, trabajo.exposicion);
    for canal in 0..5 {
        let valor = if canal == 3 {
            muestras
        } else if canal == 4 {
            revision
        } else {
            let real = f.extraer(mat.real, media, canal);
            f.unaria(op::BITCAST, mat.entero, real)
        };
        let canal = f.modulo().constante_u32(canal);
        let indice = f.binaria(op::I_ADD, mat.entero, base, canal);
        let destino = f.acceso(puntero, datos, &[cero, indice]);
        // GPU: propietario único por píxel; longitud/5 evita registros parciales.
        // La acumulación tiene orden fijo, sin atómicos ni memoria compartida.
        f.guardar(destino, valor);
    }
    f.saltar(fin);
    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    m.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    m.ensamblar()
}
