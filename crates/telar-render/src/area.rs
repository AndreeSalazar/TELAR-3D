//! Cuadratura fija de una luz de área; radiancia, coseno emisor y distancia al cuadrado.
use crate::{
    LuzArea,
    sombras::Rejilla,
    superficie::{Superficie, iluminar},
    tipos::Tipos,
};
use hilo_spirv::{Funcion, Id, op};

pub(super) fn iluminar_area(
    f: &mut Funcion,
    t: &Tipos,
    superficie: &Superficie,
    mundo: Id,
    rejilla: &Rejilla,
    luz: LuzArea,
) -> Id {
    let mut total = t.rgb(f, [0.0; 3]);
    let emisor = t.rgb(f, [0.0, 1.0, 0.0]); // dot con dirección hacia la fuente = coseno emisor hacia abajo.
    #[allow(clippy::cast_precision_loss, reason = "solo una o cuatro muestras")]
    let peso = Tipos::constante(f, 4.0 * luz.semilado * luz.semilado / luz.muestras as f32);
    let minimo = Tipos::constante(f, 0.0001);
    for muestra in 0..luz.muestras {
        let [u, v] = if luz.muestras == 1 {
            [0.0, 0.0]
        } else {
            [if muestra % 2 == 0 { -0.5 } else { 0.5 }, if muestra / 2 == 0 { -0.5 } else { 0.5 }]
        };
        let destino = t.rgb(f, [luz.centro[0] + u * luz.semilado, luz.centro[1], luz.centro[2] + v * luz.semilado]);
        let delta = f.binaria(op::F_SUB, t.v3, destino, mundo);
        let distancia2 = f.binaria(op::DOT, t.real, delta, delta);
        let distancia2 = f.glsl(t.real, op::GLSL_MAXIMO, &[distancia2, minimo]);
        let direccion = t.normalizar(f, delta);
        let coseno = f.binaria(op::DOT, t.real, emisor, direccion);
        let coseno = t.saturar(f, coseno);
        let factor = f.binaria(op::F_MUL, t.real, coseno, peso);
        let mut factor = f.binaria(op::F_DIV, t.real, factor, distancia2);
        if luz.sombras {
            let visibilidad = rejilla.visibilidad(f, t, mundo, destino, luz.pasos);
            factor = f.binaria(op::F_MUL, t.real, factor, visibilidad);
        }
        let brdf = iluminar(f, t, superficie, direccion);
        let muestra = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, brdf, factor);
        total = f.binaria(op::F_ADD, t.v3, total, muestra);
    }
    total
}
