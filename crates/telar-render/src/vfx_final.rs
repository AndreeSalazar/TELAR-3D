//! Render final compute progresivo: media HDR, contador y RGB8 de revisión residentes.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
/// Resolución cuadrada de salida; independiente del viewport y su interfaz.
pub const LADO_IMAGEN: u32 = 256;
/// Cinco palabras por píxel: RGB lineal medio, muestras u32 y RGB8 empaquetado.
pub const PALABRAS_IMAGEN: u32 = LADO_IMAGEN * LADO_IMAGEN * 5;
/// Offset de salida tras campos/configuración/secundarios; contrato compartido con app.
#[must_use]
pub fn base(lado: u32) -> u32 {
    (20 + 3072u32.div_ceil(lado.saturating_pow(3).max(1))) * lado.saturating_pow(3)
}
/// Programa de una muestra por píxel o limpieza, sin avanzar simulación.
/// # Errors
/// Rechaza dominios fuera de 8..64.
pub fn programa(lado: u32, limpiar: bool) -> Result<Vec<u32>, &'static str> {
    if !(8..=64).contains(&lado) {
        return Err("render VFX requiere lado 8..64");
    }
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let mut v = Volumen::nuevo_calculo(&mut m, &t, lado);
    v.vfx = true;
    let v3 = m.tipo_vector(mat.entero, 3);
    let ptr = m.tipo_puntero(Clase::Entrada, v3);
    let global = m.variable(ptr, Clase::Entrada);
    m.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let indice = f.cargar(v3, global);
    let indice = f.extraer(mat.entero, indice, 0);
    let cero = Matematica::escalar(&mut f, 0.0);
    f.repetir((LADO_IMAGEN * LADO_IMAGEN).div_ceil(lado.pow(3)), t.real, cero, |f, numero, _| {
        let celdas = f.modulo().constante_u32(lado.pow(3));
        let offset = f.binaria(op::I_MUL, mat.entero, numero, celdas);
        let pixel = f.binaria(op::I_ADD, mat.entero, indice, offset);
        let max = f.modulo().constante_u32(LADO_IMAGEN * LADO_IMAGEN);
        let activo = f.binaria(op::U_LESS_THAN, mat.booleano, pixel, max);
        let longitud = f.longitud(mat.entero, v.buffer, 0);
        let minimo = f.modulo().constante_u32(base(lado) + PALABRAS_IMAGEN);
        let cabe = f.binaria(op::U_LESS_THAN_EQUAL, mat.booleano, minimo, longitud);
        let activo = f.binaria(op::LOGICAL_AND, mat.booleano, activo, cabe);
        let cuerpo = f.etiqueta();
        let fin = f.etiqueta();
        f.fusion(fin);
        f.saltar_si(activo, cuerpo, fin);
        f.bloque(cuerpo);
        if limpiar {
            for c in 0..5 {
                let ptr = direccion(f, &t, &mat, &v, pixel, c);
                f.guardar(ptr, cero);
            }
        } else {
            muestra(f, &t, &mat, &v, pixel);
        }
        f.saltar(fin);
        f.bloque(fin);
        cero
    });
    f.volver();
    let main = f.terminar();
    m.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    Ok(m.ensamblar())
}
fn direccion(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, pixel: Id, canal: u32) -> Id {
    let cinco = f.modulo().constante_u32(5);
    let indice = f.binaria(op::I_MUL, mat.entero, pixel, cinco);
    let off = f.modulo().constante_u32(base(v.lado) + canal);
    let indice = f.binaria(op::I_ADD, mat.entero, indice, off);
    let cero = f.modulo().constante_u32(0);
    let _ = t;
    f.acceso(v.puntero, v.buffer, &[cero, indice])
}
fn muestra(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, pixel: Id) {
    let contador_ptr = direccion(f, t, mat, v, pixel, 3);
    let contador = f.cargar(t.real, contador_ptr);
    let contador = f.unaria(op::BITCAST, mat.entero, contador);
    let ancho = f.modulo().constante_u32(LADO_IMAGEN);
    let columna = f.binaria(op::U_MOD, mat.entero, pixel, ancho);
    let fila = f.binaria(op::U_DIV, mat.entero, pixel, ancho);
    let mut clave = f.binaria(198, mat.entero, pixel, contador);
    let coords = [columna, fila].map(|c| {
        let multiplicador = f.modulo().constante_u32(1_664_525);
        let incremento = f.modulo().constante_u32(1_013_904_223);
        clave = f.binaria(op::I_MUL, mat.entero, clave, multiplicador);
        clave = f.binaria(op::I_ADD, mat.entero, clave, incremento);
        let bits = f.modulo().constante_u32(16);
        let alto = f.binaria(194, mat.entero, clave, bits);
        let alto = f.unaria(op::U_A_F, t.real, alto);
        let escala = Matematica::escalar(f, 1.0 / 65535.0);
        let jitter = f.binaria(op::F_MUL, t.real, alto, escala);
        let coord = f.unaria(op::U_A_F, t.real, c);
        let coord = f.binaria(op::F_ADD, t.real, coord, jitter);
        let factor = Matematica::escalar(f, 1.0 / 256.0);
        f.binaria(op::F_MUL, t.real, coord, factor)
    });
    let v2 = f.modulo().tipo_vector(t.real, 2);
    let uv = f.construir(v2, &coords);
    let direccion = crate::volumen::direccion(f, t, mat, uv);
    let ojo = t.cargar(f, 7);
    let ojo = t.xyz(f, ojo);
    let opciones = t.cargar(f, 1);
    let vista = f.extraer(t.real, opciones, 3);
    let lineal = crate::trazadores::sumar(f, t, mat, v, [ojo, direccion, vista], |f| {
        crate::vfx_volumen::integrar(f, t, mat, v, [ojo, direccion, vista], 64)
    });
    let cantidad = f.unaria(op::U_A_F, t.real, contador);
    let uno = Matematica::escalar(f, 1.0);
    let nueva = f.binaria(op::F_ADD, t.real, cantidad, uno);
    let canales = core::array::from_fn::<_, 3, _>(|i| {
        let ptr = direccion_palabra(f, t, mat, v, pixel, u32::try_from(i).unwrap_or(0));
        let antes = f.cargar(t.real, ptr);
        let antes = f.binaria(op::F_MUL, t.real, antes, cantidad);
        let valor = f.extraer(t.real, lineal, u32::try_from(i).unwrap_or(0));
        let suma = f.binaria(op::F_ADD, t.real, antes, valor);
        let media = f.binaria(op::F_DIV, t.real, suma, nueva);
        // GPU: cada invocación posee su píxel; la media/contador se publican con barreras del ejecutor.
        f.guardar(ptr, media);
        media
    });
    let rgb = f.construir(t.v3, &canales);
    let opciones = t.cargar(f, 1);
    let expo = f.extraer(t.real, opciones, 2);
    let empaquetado = crate::color_rayos::convertir_exposicion(f, mat, rgb, expo);
    let uno_u = f.modulo().constante_u32(1);
    let contador = f.binaria(op::I_ADD, mat.entero, contador, uno_u);
    let contador = f.unaria(op::BITCAST, t.real, contador);
    f.guardar(contador_ptr, contador);
    let color = f.unaria(op::BITCAST, t.real, empaquetado);
    let ptr = direccion_palabra(f, t, mat, v, pixel, 4);
    f.guardar(ptr, color);
}
fn direccion_palabra(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, pixel: Id, canal: u32) -> Id {
    direccion(f, t, mat, v, pixel, canal)
}
