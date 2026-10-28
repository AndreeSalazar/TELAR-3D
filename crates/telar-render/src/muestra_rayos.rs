//! Una muestra de cámara y luz de área; iluminación directa Lambert, sin rebotes.
use crate::{escena_rayos::Trabajo, rayos::Rayo, trazado::Matematica};
use hilo_spirv::{Funcion, Id, op};

pub(crate) fn muestra(f: &mut Funcion, mat: &Matematica, trabajo: &Trabajo, pixel: Id, indice: Id) -> Id {
    let semilla = f.modulo().constante_u32(trabajo.semilla);
    let mezcla = f.modulo().constante_u32(104_729);
    let semilla = f.binaria(op::I_ADD, mat.entero, semilla, pixel);
    let semilla = f.binaria(op::I_MUL, mat.entero, semilla, mezcla);
    let semilla = f.binaria(op::I_ADD, mat.entero, semilla, indice);
    let direccion = direccion(f, mat, trabajo, pixel, semilla);
    let origen = mat.vector(f, trabajo.camara.origen);
    let minimo = Matematica::escalar(f, 0.001);
    let maximo = Matematica::escalar(f, 1000.0);
    let rayo = Rayo { origen, direccion, minimo, maximo };
    let impacto = mat.cercano(f, trabajo, &rayo);
    let avance = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, direccion, impacto.t);
    let mundo = f.binaria(op::F_ADD, mat.vector, origen, avance);
    let cero = Matematica::escalar(f, 0.0);
    let contra = f.binaria(op::DOT, mat.real, impacto.normal, direccion);
    let invertir = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, contra, cero);
    let menos_uno = Matematica::escalar(f, -1.0);
    let inversa = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, impacto.normal, menos_uno);
    let normal = mat.elegir_vector(f, invertir, inversa, impacto.normal);
    let sesgo = Matematica::escalar(f, 0.001);
    let sesgo = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, normal, sesgo);
    let origen_sombra = f.binaria(op::F_ADD, mat.vector, mundo, sesgo);
    let destino = destino_luz(f, mat, trabajo, semilla);
    let delta = f.binaria(op::F_SUB, mat.vector, destino, origen_sombra);
    let distancia2 = f.binaria(op::DOT, mat.real, delta, delta);
    let minimo_distancia = Matematica::escalar(f, 0.0001);
    let distancia2 = f.glsl(mat.real, op::GLSL_MAXIMO, &[distancia2, minimo_distancia]);
    let luz = mat.normalizar(f, delta);
    let coseno = f.binaria(op::DOT, mat.real, normal, luz);
    let coseno = f.glsl(mat.real, op::GLSL_MAXIMO, &[coseno, cero]);
    let coseno_emisor = f.extraer(mat.real, luz, 1);
    let coseno_emisor = f.glsl(mat.real, op::GLSL_MAXIMO, &[coseno_emisor, cero]);
    let minimo = Matematica::escalar(f, 0.00001);
    let maximo = Matematica::escalar(f, 0.99999);
    let sombra = Rayo { origen: origen_sombra, direccion: delta, minimo, maximo };
    let visible = mat.visible(f, trabajo, &sombra);
    let peso =
        Matematica::escalar(f, trabajo.energia * 4.0 * trabajo.radio_luz * trabajo.radio_luz / core::f32::consts::PI);
    let factor = f.binaria(op::F_MUL, mat.real, coseno, coseno_emisor);
    let factor = f.binaria(op::F_MUL, mat.real, factor, peso);
    let factor = f.binaria(op::F_MUL, mat.real, factor, visible);
    let factor = f.binaria(op::F_DIV, mat.real, factor, distancia2);
    let iluminado = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, impacto.color, factor);
    let fondo = mat.vector(f, [0.015, 0.02, 0.03]);
    mat.elegir_vector(f, impacto.existe, iluminado, fondo)
}

#[allow(clippy::cast_precision_loss, reason = "dimensiones <=512")]
fn direccion(f: &mut Funcion, mat: &Matematica, trabajo: &Trabajo, pixel: Id, semilla: Id) -> Id {
    let ancho = f.modulo().constante_u32(trabajo.tamano[0]);
    let columna = f.binaria(op::U_MOD, mat.entero, pixel, ancho);
    let fila = f.binaria(op::U_DIV, mat.entero, pixel, ancho);
    let mut direccion = mat.vector(f, trabajo.camara.frente);
    for (eje, dimension, base, sal, invertir) in [
        (columna, trabajo.tamano[0], trabajo.camara.derecha, 0, false),
        (fila, trabajo.tamano[1], trabajo.camara.arriba, 1, true),
    ] {
        let posicion = f.unaria(op::U_A_F, mat.real, eje);
        let subpixel = if trabajo.muestras == 1 { Matematica::escalar(f, 0.5) } else { mat.azar(f, semilla, sal) };
        let posicion = f.binaria(op::F_ADD, mat.real, posicion, subpixel);
        let escala = Matematica::escalar(f, 2.0 / dimension as f32);
        let posicion = f.binaria(op::F_MUL, mat.real, posicion, escala);
        let uno = Matematica::escalar(f, 1.0);
        let coordenada = if invertir {
            f.binaria(op::F_SUB, mat.real, uno, posicion)
        } else {
            f.binaria(op::F_SUB, mat.real, posicion, uno)
        };
        let base = mat.vector(f, base);
        let avance = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, base, coordenada);
        direccion = f.binaria(op::F_ADD, mat.vector, direccion, avance);
    }
    mat.normalizar(f, direccion)
}

fn destino_luz(f: &mut Funcion, mat: &Matematica, trabajo: &Trabajo, semilla: Id) -> Id {
    let mut centro = trabajo.luz.map(|v| Matematica::escalar(f, v));
    for (eje, sal) in [(0, 2), (2, 3)] {
        let muestra = mat.azar(f, semilla, sal);
        let dos = Matematica::escalar(f, 2.0 * trabajo.radio_luz);
        let radio = Matematica::escalar(f, trabajo.radio_luz);
        let desplazamiento = f.binaria(op::F_MUL, mat.real, muestra, dos);
        let desplazamiento = f.binaria(op::F_SUB, mat.real, desplazamiento, radio);
        centro[eje] = f.binaria(op::F_ADD, mat.real, centro[eje], desplazamiento);
    }
    f.construir(mat.vector, &centro)
}
