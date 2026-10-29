//! Estelas gaussianas de cuatro segmentos; contribución emisiva diagnóstica sin oclusión por humo.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use hilo_spirv::{Funcion, Id, op};
pub(super) fn sumar(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, id: Id, datos: [Id; 4]) -> Id {
    let [ojo, direccion, mut inicio, mut rgb] = datos;
    let intensidad = crate::vfx_volumen::ajuste(f, t, v, 27);
    for muestra in 0..4 {
        let xyz = core::array::from_fn::<_, 3, _>(|eje| {
            let entero = f.modulo().tipo_u32();
            let base = f
                .modulo()
                .constante_u32(20 * v.lado.pow(3) + 1536 + (muestra * 3 + u32::try_from(eje).unwrap_or(0)) * 128);
            let indice = f.binaria(op::I_ADD, entero, base, id);
            let cero = f.modulo().constante_u32(0);
            let ptr = f.acceso(v.puntero, v.buffer, &[cero, indice]);
            f.cargar(t.real, ptr)
        });
        let fin = f.construir(t.v3, &xyz);
        let peso = segmento(f, t, mat, [ojo, direccion, inicio, fin]);
        let factor = Matematica::escalar(f, [0.7, 0.5, 0.3, 0.15][muestra as usize]);
        let peso = f.binaria(op::F_MUL, t.real, peso, factor);
        let peso = f.binaria(op::F_MUL, t.real, peso, intensidad);
        let color = mat.vector(f, [1.0, 0.3, 0.02]);
        let aporte = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, peso);
        rgb = f.binaria(op::F_ADD, t.v3, rgb, aporte);
        inicio = fin;
    }
    rgb
}
fn segmento(f: &mut Funcion, t: &Tipos, mat: &Matematica, c: [Id; 4]) -> Id {
    let [ojo, dir, inicio, fin] = c;
    let rel_a = f.binaria(op::F_SUB, t.v3, inicio, ojo);
    let rel_b = f.binaria(op::F_SUB, t.v3, fin, ojo);
    let za = f.binaria(op::DOT, t.real, rel_a, dir);
    let zb = f.binaria(op::DOT, t.real, rel_b, dir);
    let pa = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, dir, za);
    let pa = f.binaria(op::F_SUB, t.v3, rel_a, pa);
    let pb = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, dir, zb);
    let pb = f.binaria(op::F_SUB, t.v3, rel_b, pb);
    let delta = f.binaria(op::F_SUB, t.v3, pb, pa);
    let largo = f.binaria(op::DOT, t.real, delta, delta);
    let min = Matematica::escalar(f, 1e-12);
    let largo = f.glsl(t.real, op::GLSL_MAXIMO, &[largo, min]);
    let dot = f.binaria(op::DOT, t.real, pa, delta);
    let menos = Matematica::escalar(f, -1.0);
    let dot = f.binaria(op::F_MUL, t.real, dot, menos);
    let fraccion = f.binaria(op::F_DIV, t.real, dot, largo);
    let cero = Matematica::escalar(f, 0.0);
    let uno = Matematica::escalar(f, 1.0);
    let fraccion = f.glsl(t.real, 43, &[fraccion, cero, uno]);
    let offset = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, delta, fraccion);
    let punto = f.binaria(op::F_ADD, t.v3, pa, offset);
    let d2 = f.binaria(op::DOT, t.real, punto, punto);
    let escala = Matematica::escalar(f, -2.0 / (0.004 * 0.004));
    let potencia = f.binaria(op::F_MUL, t.real, d2, escala);
    let peso = f.glsl(t.real, 27, &[potencia]);
    let dz = f.binaria(op::F_SUB, t.real, zb, za);
    let dz = f.binaria(op::F_MUL, t.real, dz, fraccion);
    let z = f.binaria(op::F_ADD, t.real, za, dz);
    let delante = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, z, cero);
    f.elegir(t.real, delante, peso, cero)
}
