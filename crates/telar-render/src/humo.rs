//! Visor de campos MAC 2D residentes; interpolación bilineal, sin descargas.
use crate::{
    tipos::{Tipos, variable},
    trazado::Matematica,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
/// Lee doce planos f32 en set 0/binding 1. Constantes [6]=exposición, [7]=vista,
/// [9]=codificar sRGB. Las demás constantes pertenecen al solver.
/// # Errors
/// Rechaza lados fuera de 8..256.
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
pub fn fragmentos(lado: u32) -> Result<Vec<u32>, &'static str> {
    emitir(lado, false)
}
/// Visor con franja de estado del control adaptativo en el plano 14.
/// # Errors
/// Rechaza lados fuera de 8..256; requiere estado de quince planos.
pub fn fragmentos_adaptativo(lado: u32) -> Result<Vec<u32>, &'static str> {
    emitir(lado, true)
}
#[allow(clippy::cast_precision_loss, reason = "lado <=256")]
fn emitir(lado: u32, adaptativo: bool) -> Result<Vec<u32>, &'static str> {
    if !(8..=256).contains(&lado) {
        return Err("visor de humo fuera de dominio");
    }
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let v2 = m.tipo_vector(t.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let array = m.tipo_array_dinamico(t.real, 4);
    let bloque = m.tipo_bloque(&[array], &[0]);
    let ptr = m.tipo_puntero(Clase::Almacen, bloque);
    let estado = m.variable(ptr, Clase::Almacen);
    m.decorar(estado, op::DECORACION_CONJUNTO, &[0]);
    m.decorar(estado, op::DECORACION_ENLACE, &[1]);
    m.decorar(estado, 24, &[]);
    let puntero = m.tipo_puntero(Clase::Almacen, t.real);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let uv = f.cargar(v2, entrada);
    let escala = f.modulo().constante_f32((lado - 1) as f32);
    let uno = f.modulo().constante_f32(1.0);
    let horizontal = f.extraer(t.real, uv, 0);
    let vertical = f.extraer(t.real, uv, 1);
    let vertical = f.binaria(op::F_SUB, t.real, uno, vertical);
    let coords = [horizontal, vertical].map(|v| f.binaria(op::F_MUL, t.real, v, escala));
    let campos: [_; 5] =
        core::array::from_fn(|i| leer(&mut f, &t, &mat, estado, puntero, lado, coords, [2, 3, 0, 1, 11][i]));
    let densidad = campos[0];
    let temperatura = campos[1];
    let rgb = f.construir(t.v3, &[densidad; 3]);
    let ajustes = t.cargar(&mut f, 1);
    let vista = f.extraer(t.real, ajustes, 3);
    let expo = f.extraer(t.real, ajustes, 2);
    let negro = f.modulo().constante_f32(0.0);
    let naranja = f.modulo().constante_f32(0.25);
    let calor_medio = f.binaria(op::F_MUL, t.real, temperatura, naranja);
    let termico = f.construir(t.v3, &[temperatura, calor_medio, negro]);
    let velocidad = campos[2..4].iter().map(|v| f.binaria(op::F_MUL, t.real, *v, *v)).collect::<Vec<_>>();
    let rapidez = f.binaria(op::F_ADD, t.real, velocidad[0], velocidad[1]);
    let rapidez = f.glsl(t.real, 31, &[rapidez]);
    let factor = f.modulo().constante_f32(4.0);
    let rapidez = f.binaria(op::F_MUL, t.real, rapidez, factor);
    let flujo = f.construir(t.v3, &[negro, rapidez, rapidez]);
    let div = campos[4];
    let opuesto = f.binaria(op::F_SUB, t.real, negro, div);
    let positivo = f.glsl(t.real, op::GLSL_MAXIMO, &[div, negro]);
    let negativo = f.glsl(t.real, op::GLSL_MAXIMO, &[opuesto, negro]);
    let residual = f.construir(t.v3, &[positivo, negro, negativo]);
    let mut rgb = rgb;
    for (indice, alternativa) in [(1.0, termico), (2.0, flujo), (3.0, residual)] {
        let indice = f.modulo().constante_f32(indice);
        let igual = f.binaria(180, mat.booleano, vista, indice);
        rgb = mat.elegir_vector(&mut f, igual, alternativa, rgb);
    }
    let opciones = t.cargar(&mut f, 2);
    let srgb = f.extraer(t.real, opciones, 1);
    let color = crate::superficie::salida(&mut f, &t, rgb, expo, srgb);
    let color =
        if adaptativo { indicador(&mut f, &t, &mat, estado, puntero, uv, 14 * lado * lado, color) } else { color };
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    Ok(m.ensamblar())
}
#[allow(clippy::too_many_arguments, reason = "ABI explícita del buffer y coordenadas del emisor")]
fn leer(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    estado: Id,
    puntero: Id,
    lado: u32,
    coords: [Id; 2],
    canal: u32,
) -> Id {
    let indices = coords.map(|v| f.unaria(op::F_A_U, mat.entero, v));
    let fracciones = core::array::from_fn::<_, 2, _>(|i| {
        let base = f.unaria(op::U_A_F, t.real, indices[i]);
        f.binaria(op::F_SUB, t.real, coords[i], base)
    });
    let cero = f.modulo().constante_u32(0);
    let max = f.modulo().constante_u32(lado - 1);
    let ancho = f.modulo().constante_u32(lado);
    let plano = f.modulo().constante_u32(canal * lado * lado);
    let valores = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| {
        let dx = f.modulo().constante_u32(dx);
        let dy = f.modulo().constante_u32(dy);
        let x = f.binaria(op::I_ADD, mat.entero, indices[0], dx);
        let y = f.binaria(op::I_ADD, mat.entero, indices[1], dy);
        let x = f.glsl(mat.entero, 38, &[x, max]);
        let y = f.glsl(mat.entero, 38, &[y, max]);
        let fila = f.binaria(op::I_MUL, mat.entero, y, ancho);
        let pixel = f.binaria(op::I_ADD, mat.entero, fila, x);
        let indice = f.binaria(op::I_ADD, mat.entero, pixel, plano);
        let ptr = f.acceso(puntero, estado, &[cero, indice]);
        f.cargar(t.real, ptr)
    });
    let mezclar = |f: &mut Funcion, a: Id, b: Id, peso: Id| {
        let delta = f.binaria(op::F_SUB, t.real, b, a);
        let delta = f.binaria(op::F_MUL, t.real, delta, peso);
        f.binaria(op::F_ADD, t.real, a, delta)
    };
    let inferior = mezclar(f, valores[0], valores[1], fracciones[0]);
    let superior = mezclar(f, valores[2], valores[3], fracciones[0]);
    mezclar(f, inferior, superior, fracciones[1])
}

#[allow(clippy::too_many_arguments, reason = "ABI explícita del buffer y color del fragmento")]
pub(super) fn indicador(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    estado: Id,
    puntero: Id,
    uv: Id,
    base: u32,
    color: Id,
) -> Id {
    let cero = f.modulo().constante_u32(0);
    let indice = f.modulo().constante_u32(base + 5);
    let ptr = f.acceso(puntero, estado, &[cero, indice]);
    let estado = f.cargar(t.real, ptr);
    let mut franja = t.rgb(f, [0.08, 0.2, 0.45]);
    for (codigo, rgba) in [(1.0, [0.03, 0.6, 0.15]), (2.0, [0.9, 0.55, 0.02]), (3.0, [0.8, 0.03, 0.02])] {
        let codigo = f.modulo().constante_f32(codigo);
        let igual = f.binaria(180, mat.booleano, estado, codigo);
        let alternativa = t.rgb(f, rgba);
        franja = mat.elegir_vector(f, igual, alternativa, franja);
    }
    let arriba = f.extraer(t.real, uv, 1);
    let ancho = f.modulo().constante_f32(0.012);
    let banda = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, ancho, arriba);
    let rgb = t.xyz(f, color);
    let rgb = mat.elegir_vector(f, banda, franja, rgb);
    let canales = core::array::from_fn::<_, 3, _>(|i| f.extraer(t.real, rgb, u32::try_from(i).unwrap_or(0)));
    let uno = f.modulo().constante_f32(1.0);
    f.construir(t.v4, &[canales[0], canales[1], canales[2], uno])
}
