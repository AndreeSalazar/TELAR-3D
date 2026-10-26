//! El primer kernel de TELAR, escrito con el constructor (todavía sin HILO):
//!
//! ```glsl
//! layout(local_size_x = grupo) in;
//! layout(set = 0, binding = 0) buffer Datos { uint datos[]; };
//! void main() {
//!     uint i = gl_GlobalInvocationID.x;
//!     if (i < datos.length()) datos[i] = hash_pcg(i);   // nunca fuera del buffer
//! }
//! ```
//!
//! El hash es el PCG de Jarzynski y Olano, *Hash Functions for GPU Rendering*
//! (JCGT, 2020): el azar determinista de docs/MATEMATICAS.md. Solo usa
//! multiplicaciones, sumas, desplazamientos y XOR de enteros de 32 bits, así que
//! la GPU y la CPU dan **los mismos bits**.

use alloc::vec::Vec;

use crate::modulo::Modulo;
use crate::op::{self, Clase};

/// Los números del PCG (Jarzynski y Olano, 2020).
const MULTIPLICADOR: u32 = 747_796_405;
const INCREMENTO: u32 = 2_891_336_453;
const MEZCLA: u32 = 277_803_737;

/// La referencia en la CPU: lo que tiene que dar la GPU, bit a bit.
#[must_use]
pub const fn hash_pcg(v: u32) -> u32 {
    let estado = v.wrapping_mul(MULTIPLICADOR).wrapping_add(INCREMENTO);
    let palabra = ((estado >> ((estado >> 28) + 4)) ^ estado).wrapping_mul(MEZCLA);
    (palabra >> 22) ^ palabra
}

/// El kernel en SPIR-V: llena el buffer del conjunto 0, enlace 0, con `hash_pcg(i)`.
/// `grupo` es el tamaño del grupo de trabajo (`local_size_x`).
#[must_use]
pub fn llenar_con_hash(grupo: u32) -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let void = m.tipo_void();
    let tipo_main = m.tipo_funcion(void, &[]);
    let u32_ = m.tipo_u32();
    let bool_ = m.tipo_bool();
    let v3 = m.tipo_vector(u32_, 3);
    let p_entrada_v3 = m.tipo_puntero(Clase::Entrada, v3);
    let p_entrada_u32 = m.tipo_puntero(Clase::Entrada, u32_);
    let array = m.tipo_array_dinamico(u32_, 4);
    let bloque = m.tipo_bloque(&[array], &[0]);
    let p_bloque = m.tipo_puntero(Clase::Almacen, bloque);
    let p_dato = m.tipo_puntero(Clase::Almacen, u32_);

    let id_global = m.variable(p_entrada_v3, Clase::Entrada);
    m.decorar(id_global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    m.nombre(id_global, "gl_GlobalInvocationID");
    let datos = m.variable(p_bloque, Clase::Almacen);
    m.decorar(datos, op::DECORACION_CONJUNTO, &[0]);
    m.decorar(datos, op::DECORACION_ENLACE, &[0]);
    m.nombre(datos, "datos");

    let cero = m.constante_u32(0);
    let [multiplicador, incremento, mezcla, c28, c4, c22] =
        [MULTIPLICADOR, INCREMENTO, MEZCLA, 28, 4, 22].map(|v| m.constante_u32(v));

    let mut f = m.funcion(void, tipo_main);
    let (entrada, cuerpo, fin) = (f.etiqueta(), f.etiqueta(), f.etiqueta());
    f.bloque(entrada);
    let px = f.acceso(p_entrada_u32, id_global, &[cero]);
    let i = f.cargar(u32_, px);
    let n = f.longitud(u32_, datos, 0);
    let dentro = f.binaria(op::U_LESS_THAN, bool_, i, n);
    f.fusion(fin);
    f.saltar_si(dentro, cuerpo, fin);

    f.bloque(cuerpo);
    let mul = f.binaria(op::I_MUL, u32_, i, multiplicador);
    let estado = f.binaria(op::I_ADD, u32_, mul, incremento);
    let alto = f.binaria(op::SHIFT_RIGHT_LOGICAL, u32_, estado, c28);
    let giro = f.binaria(op::I_ADD, u32_, alto, c4);
    let desplazado = f.binaria(op::SHIFT_RIGHT_LOGICAL, u32_, estado, giro);
    let mezclado = f.binaria(op::BITWISE_XOR, u32_, desplazado, estado);
    let palabra = f.binaria(op::I_MUL, u32_, mezclado, mezcla);
    let final_ = f.binaria(op::SHIFT_RIGHT_LOGICAL, u32_, palabra, c22);
    let hash = f.binaria(op::BITWISE_XOR, u32_, final_, palabra);
    let destino = f.acceso(p_dato, datos, &[cero, i]);
    f.guardar(destino, hash);
    f.saltar(fin);

    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    m.nombre(main, "main");
    m.entrada_calculo(main, "main", &[id_global], [grupo, 1, 1]);
    m.ensamblar()
}
