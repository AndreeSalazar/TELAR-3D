//! Los shaders del visor, escritos con el constructor:
//!
//! ```glsl
//! layout(push_constant) uniform Constantes {   // 128 bytes: 8 vec4
//!     vec4 c0, c1, c2, c3;   // proyección × vista, por columnas
//!     vec4 luz;              // dirección hacia la luz (xyz, unitaria)
//!     vec4 color;            // color de la superficie (rgb, lineal)
//!     vec4 cielo, suelo;     // la luz ambiente de arriba y de abajo
//! } k;
//!
//! // vértices
//! layout(location = 0) in vec3 posicion;  layout(location = 1) in vec3 normal;
//! layout(location = 0) out vec3 n;
//! void main() { gl_Position = k.c0*posicion.x + k.c1*posicion.y + k.c2*posicion.z + k.c3; n = normal; }
//!
//! // fragmentos
//! layout(location = 0) in vec3 n;  layout(location = 0) out vec4 salida;
//! void main() {
//!     vec3 N = normalize(n);
//!     float difusa = max(dot(N, k.luz.xyz), 0.0);
//!     vec3 ambiente = mix(k.suelo.xyz, k.cielo.xyz, vec3(N.y * 0.5 + 0.5));
//!     salida = vec4(k.color.xyz * (ambiente + vec3(difusa * 0.85)), 1.0);
//! }
//! ```
//!
//! La luz ambiente «de cielo y suelo» (hemisférica) hace que las caras que miran
//! hacia arriba se vean más claras y las de abajo más oscuras, sin sombras.

use alloc::vec::Vec;

use crate::modulo::{Id, Modulo};
use crate::op::{self, Clase};

/// Bytes de las constantes de empuje: 8 `vec4` (el mínimo que garantiza Vulkan).
pub const TAMANO_CONSTANTES: u32 = 128;

/// Los tipos que comparten los dos shaders.
struct Tipos {
    f32_: Id,
    v3: Id,
    v4: Id,
    /// Puntero a un `vec4` de las constantes.
    p_constante: Id,
    /// La variable de las constantes.
    constantes: Id,
}

fn tipos(m: &mut Modulo) -> Tipos {
    let f32_ = m.tipo_f32();
    let v3 = m.tipo_vector(f32_, 3);
    let v4 = m.tipo_vector(f32_, 4);
    let bloque = m.tipo_bloque(&[v4; 8], &[0, 16, 32, 48, 64, 80, 96, 112]);
    let p_bloque = m.tipo_puntero(Clase::Constantes, bloque);
    let p_constante = m.tipo_puntero(Clase::Constantes, v4);
    let constantes = m.variable(p_bloque, Clase::Constantes);
    m.nombre(constantes, "k");
    Tipos { f32_, v3, v4, p_constante, constantes }
}

/// Una variable de entrada o salida en `ubicacion`.
fn variable(m: &mut Modulo, tipo: Id, clase: Clase, ubicacion: u32, nombre: &str) -> Id {
    let puntero = m.tipo_puntero(clase, tipo);
    let v = m.variable(puntero, clase);
    m.decorar(v, op::DECORACION_UBICACION, &[ubicacion]);
    m.nombre(v, nombre);
    v
}

/// El shader de vértices.
#[must_use]
pub fn vertices() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = tipos(&mut m);
    let void = m.tipo_void();
    let tipo_main = m.tipo_funcion(void, &[]);
    let posicion = variable(&mut m, t.v3, Clase::Entrada, 0, "posicion");
    let normal = variable(&mut m, t.v3, Clase::Entrada, 1, "normal");
    let n_salida = variable(&mut m, t.v3, Clase::Salida, 0, "n");
    let p_salida_v4 = m.tipo_puntero(Clase::Salida, t.v4);
    let gl_position = m.variable(p_salida_v4, Clase::Salida);
    m.decorar(gl_position, op::DECORACION_INTEGRADA, &[op::INTEGRADA_POSICION]);
    m.nombre(gl_position, "gl_Position");
    let indices: [Id; 4] = core::array::from_fn(|i| m.constante_u32(u32::try_from(i).unwrap_or(0)));

    let mut f = m.funcion(void, tipo_main);
    let entrada = f.etiqueta();
    f.bloque(entrada);
    let leida = f.cargar(t.v3, posicion);
    let componentes: [Id; 3] = core::array::from_fn(|i| f.extraer(t.f32_, leida, u32::try_from(i).unwrap_or(0)));
    let columnas: [Id; 4] = core::array::from_fn(|i| {
        let puntero = f.acceso(t.p_constante, t.constantes, &[indices[i]]);
        f.cargar(t.v4, puntero)
    });
    // c0·x + c1·y + c2·z + c3
    let mut suma = columnas[3];
    for (columna, componente) in columnas[..3].iter().zip(componentes) {
        let termino = f.binaria(op::VECTOR_TIMES_SCALAR, t.v4, *columna, componente);
        suma = f.binaria(op::F_ADD, t.v4, suma, termino);
    }
    f.guardar(gl_position, suma);
    let valor_normal = f.cargar(t.v3, normal);
    f.guardar(n_salida, valor_normal);
    f.volver();
    let main = f.terminar();
    m.nombre(main, "main");
    m.entrada(op::MODELO_VERTICES, main, "main", &[posicion, normal, n_salida, gl_position]);
    m.ensamblar()
}

/// El shader de fragmentos.
#[must_use]
pub fn fragmentos() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = tipos(&mut m);
    let void = m.tipo_void();
    let tipo_main = m.tipo_funcion(void, &[]);
    let n_entrada = variable(&mut m, t.v3, Clase::Entrada, 0, "n");
    let salida = variable(&mut m, t.v4, Clase::Salida, 0, "salida");
    let [i_luz, i_color, i_cielo, i_suelo] = [4, 5, 6, 7].map(|i| m.constante_u32(i));
    let [cero, medio, difusa_max, uno] = [0.0, 0.5, 0.85, 1.0].map(|v| m.constante_f32(v));

    let mut f = m.funcion(void, tipo_main);
    let entrada = f.etiqueta();
    f.bloque(entrada);
    // Un vec3 de las constantes: carga el vec4 y se queda con .xyz.
    let xyz = |f: &mut crate::Funcion<'_>, indice: Id| {
        let puntero = f.acceso(t.p_constante, t.constantes, &[indice]);
        let vector = f.cargar(t.v4, puntero);
        f.barajar(t.v3, vector, vector, &[0, 1, 2])
    };
    let crudo = f.cargar(t.v3, n_entrada);
    let normal = f.glsl(t.v3, op::GLSL_NORMALIZAR, &[crudo]);
    let luz = xyz(&mut f, i_luz);
    let coseno = f.binaria(op::DOT, t.f32_, normal, luz);
    let difusa = f.glsl(t.f32_, op::GLSL_MAXIMO, &[coseno, cero]);
    let altura = f.extraer(t.f32_, normal, 1);
    let mitad = f.binaria(op::F_MUL, t.f32_, altura, medio);
    let mezcla = f.binaria(op::F_ADD, t.f32_, mitad, medio);
    let mezcla3 = f.construir(t.v3, &[mezcla, mezcla, mezcla]);
    let (cielo, suelo) = (xyz(&mut f, i_cielo), xyz(&mut f, i_suelo));
    let ambiente = f.glsl(t.v3, op::GLSL_MEZCLA, &[suelo, cielo, mezcla3]);
    let directa = f.binaria(op::F_MUL, t.f32_, difusa, difusa_max);
    let directa3 = f.construir(t.v3, &[directa, directa, directa]);
    let luz_total = f.binaria(op::F_ADD, t.v3, ambiente, directa3);
    let color = xyz(&mut f, i_color);
    let rgb = f.binaria(op::F_MUL, t.v3, color, luz_total);
    let final_ = f.construir(t.v4, &[rgb, uno]);
    f.guardar(salida, final_);
    f.volver();
    let main = f.terminar();
    m.nombre(main, "main");
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[n_entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    m.ensamblar()
}
