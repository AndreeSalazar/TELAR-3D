//! GGX isotrópico con Fresnel Schlick y visibilidad Schlick-GGX aproximada.
//! Variantes de luces direccionales o área con autosombra de rejilla; ambiente aproximado.
use crate::tipos::{Tipos, variable};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

pub(super) struct Superficie {
    normal: Id,
    vista: Id,
    color: Id,
    rugosidad: Id,
    metal: Id,
}

fn multiplicar(f: &mut Funcion, t: &Tipos, a: Id, b: Id) -> Id {
    f.binaria(op::F_MUL, t.real, a, b)
}
fn cuadrado(f: &mut Funcion, t: &Tipos, a: Id) -> Id {
    multiplicar(f, t, a, a)
}
fn quinta(f: &mut Funcion, t: &Tipos, a: Id) -> Id {
    let dos = cuadrado(f, t, a);
    let cuatro = cuadrado(f, t, dos);
    multiplicar(f, t, cuatro, a)
}
fn coseno(f: &mut Funcion, t: &Tipos, a: Id, b: Id) -> Id {
    let valor = f.binaria(op::DOT, t.real, a, b);
    t.saturar(f, valor)
}
fn geometria(f: &mut Funcion, t: &Tipos, coseno: Id, k: Id) -> Id {
    let uno = Tipos::constante(f, 1.0);
    let inverso = f.binaria(op::F_SUB, t.real, uno, k);
    let producto = multiplicar(f, t, coseno, inverso);
    let divisor = f.binaria(op::F_ADD, t.real, producto, k);
    f.binaria(op::F_DIV, t.real, coseno, divisor)
}

pub(super) fn iluminar(f: &mut Funcion, t: &Tipos, s: &Superficie, luz: Id) -> Id {
    let uno = Tipos::constante(f, 1.0);
    let pi = Tipos::constante(f, core::f32::consts::PI);
    let suma = f.binaria(op::F_ADD, t.v3, s.vista, luz);
    let mitad = t.normalizar(f, suma);
    let nl = coseno(f, t, s.normal, luz);
    let nv = coseno(f, t, s.normal, s.vista);
    let nh = coseno(f, t, s.normal, mitad);
    let vh = coseno(f, t, s.vista, mitad);
    let alfa = cuadrado(f, t, s.rugosidad);
    let alfa2 = cuadrado(f, t, alfa);
    let nh2 = cuadrado(f, t, nh);
    let diferencia = f.binaria(op::F_SUB, t.real, alfa2, uno);
    let termino = multiplicar(f, t, nh2, diferencia);
    let termino = f.binaria(op::F_ADD, t.real, termino, uno);
    let termino = cuadrado(f, t, termino);
    let divisor = multiplicar(f, t, pi, termino);
    let distribucion = f.binaria(op::F_DIV, t.real, alfa2, divisor);
    let mas_uno = f.binaria(op::F_ADD, t.real, s.rugosidad, uno);
    let mas_uno2 = cuadrado(f, t, mas_uno);
    let octavo = Tipos::constante(f, 0.125);
    let k = multiplicar(f, t, mas_uno2, octavo);
    let gv = geometria(f, t, nv, k);
    let gl = geometria(f, t, nl, k);
    let visibilidad = multiplicar(f, t, gv, gl);
    let base = t.rgb(f, [0.04; 3]);
    let metal3 = f.construir(t.v3, &[s.metal; 3]);
    let f0 = f.glsl(t.v3, op::GLSL_MEZCLA, &[base, s.color, metal3]);
    let inverso = f.binaria(op::F_SUB, t.real, uno, vh);
    let potencia = quinta(f, t, inverso);
    let blanco = t.rgb(f, [1.0; 3]);
    let restante = f.binaria(op::F_SUB, t.v3, blanco, f0);
    let extra = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, restante, potencia);
    let fresnel = f.binaria(op::F_ADD, t.v3, f0, extra);
    let numerador = multiplicar(f, t, distribucion, visibilidad);
    let producto = multiplicar(f, t, nv, nl);
    let cuatro = Tipos::constante(f, 4.0);
    let divisor = multiplicar(f, t, producto, cuatro);
    let epsilon = Tipos::constante(f, 0.0001);
    let divisor = f.glsl(t.real, op::GLSL_MAXIMO, &[divisor, epsilon]);
    let factor = f.binaria(op::F_DIV, t.real, numerador, divisor);
    let especular = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, fresnel, factor);
    let energia = f.binaria(op::F_SUB, t.v3, blanco, fresnel);
    let difusa = f.binaria(op::F_MUL, t.v3, s.color, energia);
    let no_metal = f.binaria(op::F_SUB, t.real, uno, s.metal);
    let factor_difuso = f.binaria(op::F_DIV, t.real, no_metal, pi);
    let difusa = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, difusa, factor_difuso);
    let brdf = f.binaria(op::F_ADD, t.v3, difusa, especular);
    f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, brdf, nl)
}

pub(super) fn salida(f: &mut Funcion, t: &Tipos, radiancia: Id, exposicion: Id, codificar: Id) -> Id {
    let expuesta = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, radiancia, exposicion);
    let blanco = t.rgb(f, [1.0; 3]);
    let divisor = f.binaria(op::F_ADD, t.v3, expuesta, blanco);
    let lineal = f.binaria(op::F_DIV, t.v3, expuesta, divisor); // Reinhard.
    let booleano = f.modulo().tipo_bool();
    let medio = Tipos::constante(f, 0.5);
    let convertir = f.binaria(op::F_ORD_GREATER_THAN, booleano, codificar, medio);
    let componentes = [0, 1, 2].map(|i| {
        let valor = f.extraer(t.real, lineal, i);
        let exponente = Tipos::constante(f, 1.0 / 2.4);
        let potencia = f.glsl(t.real, 26, &[valor, exponente]);
        let a = Tipos::constante(f, 1.055);
        let b = Tipos::constante(f, 0.055);
        let alta = multiplicar(f, t, potencia, a);
        let alta = f.binaria(op::F_SUB, t.real, alta, b);
        let pendiente = Tipos::constante(f, 12.92);
        let baja = multiplicar(f, t, valor, pendiente);
        let umbral = Tipos::constante(f, 0.003_130_8);
        let pequena = f.binaria(op::F_ORD_GREATER_THAN, booleano, umbral, valor);
        let srgb = f.elegir(t.real, pequena, baja, alta);
        f.elegir(t.real, convertir, srgb, valor)
    });
    let alfa = Tipos::constante(f, 1.0);
    f.construir(t.v4, &[componentes[0], componentes[1], componentes[2], alfa])
}

/// Shader GGX de vista previa. No implementa trazado de caminos ni sombras.
#[must_use]
pub fn fragmentos() -> Vec<u32> {
    emitir(None)
}

pub(super) fn emitir(area: Option<crate::LuzArea>) -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let rejilla = area.map(|_| crate::sombras::Rejilla::nueva(&mut m, &t));
    let normal_entrada = variable(&mut m, t.v3, Clase::Entrada, 0);
    let posicion_entrada = variable(&mut m, t.v3, Clase::Entrada, 1);
    let color_salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let vacio = m.tipo_void();
    let firma = m.tipo_funcion(vacio, &[]);
    let mut f = m.funcion(vacio, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let ojo4 = t.cargar(&mut f, 4);
    let luz4 = t.cargar(&mut f, 5);
    let material4 = t.cargar(&mut f, 6);
    let opciones = t.cargar(&mut f, 7);
    let ojo = t.xyz(&mut f, ojo4);
    let mundo = f.cargar(t.v3, posicion_entrada);
    let vista = f.binaria(op::F_SUB, t.v3, ojo, mundo);
    let vista = t.normalizar(&mut f, vista);
    let cruda = f.cargar(t.v3, normal_entrada);
    let normal = t.normalizar(&mut f, cruda);
    let rugosidad = f.extraer(t.real, material4, 3);
    let minimo = Tipos::constante(&mut f, 0.08);
    let rugosidad = f.glsl(t.real, op::GLSL_MAXIMO, &[rugosidad, minimo]);
    let color = t.xyz(&mut f, material4);
    let metal = f.extraer(t.real, opciones, 0);
    let superficie = Superficie { normal, vista, color, rugosidad, metal };
    let luz = t.xyz(&mut f, luz4);
    let luz = t.normalizar(&mut f, luz);
    let principal = match (area, rejilla.as_ref()) {
        (Some(area), Some(rejilla)) => crate::area::iluminar_area(&mut f, &t, &superficie, mundo, rejilla, area),
        _ => iluminar(&mut f, &t, &superficie, luz),
    };
    let intensidad = f.extraer(t.real, luz4, 3);
    let principal = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, principal, intensidad);
    let calida = t.rgb(&mut f, [1.0, 0.90, 0.78]);
    let principal = f.binaria(op::F_MUL, t.v3, principal, calida);
    let relleno = t.rgb(&mut f, [-0.6, 0.3, -0.8]);
    let relleno = t.normalizar(&mut f, relleno);
    let relleno = iluminar(&mut f, &t, &superficie, relleno);
    let fria = t.rgb(&mut f, [0.65, 0.85, 1.25]);
    let relleno = f.binaria(op::F_MUL, t.v3, relleno, fria);
    let relleno = if area.is_some() {
        let factor = Tipos::constante(&mut f, 0.12);
        f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, relleno, factor)
    } else {
        relleno
    };
    let altura = f.extraer(t.real, normal, 1);
    let medio = Tipos::constante(&mut f, 0.5);
    let altura = multiplicar(&mut f, &t, altura, medio);
    let altura = f.binaria(op::F_ADD, t.real, altura, medio);
    let abajo = f.extraer(t.real, opciones, 1);
    let arriba = f.extraer(t.real, opciones, 2);
    let ambiente = f.glsl(t.real, op::GLSL_MEZCLA, &[abajo, arriba, altura]);
    let ambiente = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, ambiente);
    let luz_total = f.binaria(op::F_ADD, t.v3, principal, relleno);
    let radiancia = f.binaria(op::F_ADD, t.v3, luz_total, ambiente);
    let exposicion = f.extraer(t.real, ojo4, 3);
    let codificar = f.extraer(t.real, opciones, 3);
    let resultado = salida(&mut f, &t, radiancia, exposicion, codificar);
    f.guardar(color_salida, resultado);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[normal_entrada, posicion_entrada, color_salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    m.ensamblar()
}
