//! Emisión y dispersión simple con Beer-Lambert, fase HG y sombra por integración secundaria.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use hilo_spirv::{Funcion, Id, op};
pub(super) fn ajuste(f: &mut Funcion, t: &Tipos, v: &Volumen, i: u32) -> Id {
    let cero = f.modulo().constante_u32(0);
    let indice = f.modulo().constante_u32(19 * v.lado.pow(3) + i);
    let ptr = f.acceso(v.puntero, v.buffer, &[cero, indice]);
    f.cargar(t.real, ptr)
}
#[allow(clippy::cast_precision_loss, reason = "pasos <=128")]
pub(super) fn integrar(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, camara: [Id; 3], pasos: u32) -> Id {
    let [ojo, direccion, vista] = camara;
    let cero = Matematica::escalar(f, 0.0);
    let uno = Matematica::escalar(f, 1.0);
    // Segmento conservador que cubre el cubo; las muestras exteriores valen cero.
    let centro = mat.vector(f, [0.5; 3]);
    let rel = f.binaria(op::F_SUB, t.v3, centro, ojo);
    let distancia = f.binaria(op::DOT, t.real, rel, direccion);
    let radio = Matematica::escalar(f, 0.9);
    let inicio = f.binaria(op::F_SUB, t.real, distancia, radio);
    let inicio = f.glsl(t.real, op::GLSL_MAXIMO, &[inicio, cero]);
    let largo = Matematica::escalar(f, 1.8 / pasos as f32);
    let inicial = f.construir(t.v4, &[cero, cero, cero, uno]);
    let resultado = f.repetir(pasos, t.v4, inicial, |f, indice, antes| {
        let indice = f.unaria(op::U_A_F, t.real, indice);
        let medio = Matematica::escalar(f, 0.5);
        let indice = f.binaria(op::F_ADD, t.real, indice, medio);
        let delta = f.binaria(op::F_MUL, t.real, indice, largo);
        let delta = f.binaria(op::F_ADD, t.real, inicio, delta);
        let delta = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, direccion, delta);
        let posicion = f.binaria(op::F_ADD, t.v3, ojo, delta);
        let density = muestra(f, t, mat, v, posicion, 3);
        let density = f.glsl(t.real, op::GLSL_MAXIMO, &[density, cero]);
        let ext = ajuste(f, t, v, 16);
        let sigma = f.binaria(op::F_MUL, t.real, density, ext);
        let espesor = f.binaria(op::F_MUL, t.real, sigma, largo);
        let negativo = Matematica::escalar(f, -1.0);
        let espesor = f.binaria(op::F_MUL, t.real, espesor, negativo);
        let transmision = f.glsl(t.real, 27, &[espesor]);
        let heat = muestra(f, t, mat, v, posicion, 4);
        let color = fuente(f, t, mat, v, posicion, direccion, sigma, heat);
        let color = diagnostico(f, t, mat, v, posicion, vista, color, heat);
        let resta = f.binaria(op::F_SUB, t.real, uno, transmision);
        let minimo = Matematica::escalar(f, 1e-6);
        let divisor = f.glsl(t.real, op::GLSL_MAXIMO, &[sigma, minimo]);
        let integral = f.binaria(op::F_DIV, t.real, resta, divisor);
        let vacio = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, minimo, sigma);
        let integral = f.elegir(t.real, vacio, largo, integral);
        let transparencia = f.extraer(t.real, antes, 3);
        let peso = f.binaria(op::F_MUL, t.real, transparencia, integral);
        let aporte = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, peso);
        let anterior = t.xyz(f, antes);
        let rgb = f.binaria(op::F_ADD, t.v3, anterior, aporte);
        let nueva = f.binaria(op::F_MUL, t.real, transparencia, transmision);
        let canales_rgb = core::array::from_fn::<_, 3, _>(|i| f.extraer(t.real, rgb, u32::try_from(i).unwrap_or(0)));
        f.construir(t.v4, &[canales_rgb[0], canales_rgb[1], canales_rgb[2], nueva])
    });
    let rgb = t.xyz(f, resultado);
    let fondo = mat.vector(f, [0.008, 0.012, 0.018]);
    let transparencia = f.extraer(t.real, resultado, 3);
    let fondo = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, fondo, transparencia);
    f.binaria(op::F_ADD, t.v3, rgb, fondo)
}
fn muestra(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, posicion: Id, canal: u32) -> Id {
    let canal = f.modulo().constante_u32(canal);
    v.muestra(f, t, mat, posicion, canal)
}
#[allow(clippy::too_many_arguments, reason = "factores explícitos del término fuente volumétrico")]
fn fuente(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    v: &Volumen,
    posicion: Id,
    direccion: Id,
    sigma: Id,
    heat: Id,
) -> Id {
    let canales = core::array::from_fn::<_, 3, _>(|i| ajuste(f, t, v, 12 + u32::try_from(i).unwrap_or(0)));
    let luz = f.construir(t.v3, &canales);
    let hacia = f.binaria(op::F_SUB, t.v3, luz, posicion);
    let r2 = f.binaria(op::DOT, t.real, hacia, hacia);
    let distancia_luz = f.glsl(t.real, 31, &[r2]);
    let minimo_dir = Matematica::escalar(f, 1e-8);
    let den = f.glsl(t.real, op::GLSL_MAXIMO, &[r2, minimo_dir]);
    let inv = f.glsl(t.real, 32, &[den]);
    let dir = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, hacia, inv);
    let shadow = sombra(f, t, mat, v, posicion, hacia, distancia_luz);
    let radiancia = ajuste(f, t, v, 15);
    let minimo = Matematica::escalar(f, 0.01);
    let r2 = f.glsl(t.real, op::GLSL_MAXIMO, &[r2, minimo]);
    let radiancia = f.binaria(op::F_DIV, t.real, radiancia, r2);
    let radiancia = f.binaria(op::F_MUL, t.real, radiancia, shadow);
    let cos = f.binaria(op::DOT, t.real, dir, direccion);
    let anisotropia = ajuste(f, t, v, 19);
    let gg = f.binaria(op::F_MUL, t.real, anisotropia, anisotropia);
    let uno = Matematica::escalar(f, 1.0);
    let arriba = f.binaria(op::F_SUB, t.real, uno, gg);
    let base = f.binaria(op::F_ADD, t.real, uno, gg);
    let dos = Matematica::escalar(f, 2.0);
    let gc = f.binaria(op::F_MUL, t.real, anisotropia, cos);
    let gc = f.binaria(op::F_MUL, t.real, gc, dos);
    let base = f.binaria(op::F_SUB, t.real, base, gc);
    let raiz = f.glsl(t.real, 31, &[base]);
    let den = f.binaria(op::F_MUL, t.real, base, raiz);
    let pi = Matematica::escalar(f, 12.566_371);
    let den = f.binaria(op::F_MUL, t.real, den, pi);
    let fase = f.binaria(op::F_DIV, t.real, arriba, den);
    let albedo = ajuste(f, t, v, 17);
    let directo = f.binaria(op::F_MUL, t.real, radiancia, fase);
    let directo = f.binaria(op::F_MUL, t.real, directo, albedo);
    let directo = f.binaria(op::F_MUL, t.real, directo, sigma);
    let gris = mat.vector(f, [0.8, 0.85, 0.95]);
    let gris = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, gris, directo);
    let intensidad = ajuste(f, t, v, 18);
    let intensidad = f.binaria(op::F_MUL, t.real, intensidad, heat);
    let verde_factor = Matematica::escalar(f, 0.2);
    let verde = f.binaria(op::F_MUL, t.real, heat, verde_factor);
    let verde_base = Matematica::escalar(f, 0.1);
    let verde = f.binaria(op::F_ADD, t.real, verde, verde_base);
    let azul_factor = Matematica::escalar(f, 0.12);
    let azul = f.binaria(op::F_MUL, t.real, heat, azul_factor);
    let cero = Matematica::escalar(f, 0.0);
    let verde = f.glsl(t.real, 43, &[verde, cero, uno]);
    let azul = f.glsl(t.real, 43, &[azul, cero, uno]);
    let emision = f.construir(t.v3, &[uno, verde, azul]);
    let emision = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, emision, intensidad);
    f.binaria(op::F_ADD, t.v3, gris, emision)
}
fn sombra(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, posicion: Id, hacia: Id, largo: Id) -> Id {
    let cero = Matematica::escalar(f, 0.0);
    let octavo = Matematica::escalar(f, 0.125);
    let paso = f.binaria(op::F_MUL, t.real, largo, octavo);
    let espesor = f.repetir(8, t.real, cero, |f, i, anterior| {
        let i = f.unaria(op::U_A_F, t.real, i);
        let medio = Matematica::escalar(f, 0.5);
        let i = f.binaria(op::F_ADD, t.real, i, medio);
        let i = f.binaria(op::F_MUL, t.real, i, octavo);
        let delta = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, hacia, i);
        let punto = f.binaria(op::F_ADD, t.v3, posicion, delta);
        let density = muestra(f, t, mat, v, punto, 3);
        let ext = ajuste(f, t, v, 16);
        let density = f.binaria(op::F_MUL, t.real, density, ext);
        let solido = v.solido(f, t, mat, punto);
        let opaco = Matematica::escalar(f, 40.0);
        let density = f.elegir(t.real, solido, opaco, density);
        let delta = f.binaria(op::F_MUL, t.real, density, paso);
        f.binaria(op::F_ADD, t.real, anterior, delta)
    });
    let menos = Matematica::escalar(f, -1.0);
    let espesor = f.binaria(op::F_MUL, t.real, espesor, menos);
    f.glsl(t.real, 27, &[espesor])
}
#[allow(clippy::too_many_arguments, reason = "estado explícito del diagnóstico volumétrico")]
fn diagnostico(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    v: &Volumen,
    posicion: Id,
    vista: Id,
    mut color: Id,
    heat: Id,
) -> Id {
    for codigo in [1u32, 2] {
        let valor = if codigo == 1 { heat } else { muestra(f, t, mat, v, posicion, 17) };
        let factor = Matematica::escalar(f, 6.0);
        let valor = f.binaria(op::F_MUL, t.real, valor, factor);
        let rgb = mat.vector(f, if codigo == 1 { [1.0, 0.25, 0.03] } else { [0.05, 0.8, 0.3] });
        let rgb = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, rgb, valor);
        let canales_rgb = Matematica::escalar(f, if codigo == 1 { 1.0 } else { 2.0 });
        let coincide = f.binaria(180, mat.booleano, vista, canales_rgb);
        color = mat.elegir_vector(f, coincide, rgb, color);
    }
    color
}
