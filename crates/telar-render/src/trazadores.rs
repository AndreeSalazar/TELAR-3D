//! Vista diagnóstica de puntos gaussianos MAC; no oculta por profundidad del humo.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use hilo_spirv::{Funcion, Id, op};
pub(super) fn sumar(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    v: &Volumen,
    camara: [Id; 3],
    previo: impl FnOnce(&mut Funcion) -> Id,
) -> Id {
    let [ojo, direccion, vista] = camara;
    let codigo = Matematica::escalar(f, 3.0);
    let mostrar = f.binaria(180, mat.booleano, vista, codigo);
    let si = f.etiqueta();
    let no = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(mostrar, si, no);
    f.bloque(si);
    let fondo = mat.vector(f, [0.008, 0.012, 0.018]);
    let rgb = f.repetir(128, t.v3, fondo, |f, id, antes| {
        let flag = leer(f, t, v, id, 10);
        let uno = Matematica::escalar(f, 1.0);
        let encendida = f.binaria(180, mat.booleano, flag, uno);
        let viva = f.etiqueta();
        let muerta = f.etiqueta();
        let union = f.etiqueta();
        f.fusion(union);
        f.saltar_si(encendida, viva, muerta);
        f.bloque(viva);
        let color = splat(f, t, mat, v, id, [ojo, direccion]);
        let suma = f.binaria(op::F_ADD, t.v3, antes, color);
        f.saltar(union);
        f.bloque(muerta);
        f.saltar(union);
        f.bloque(union);
        f.phi(t.v3, &[(suma, viva), (antes, muerta)])
    });
    // El bucle termina en otra etiqueta: capturar el predecesor real con bloque puente.
    let puente = f.etiqueta();
    f.saltar(puente);
    f.bloque(puente);
    f.saltar(fin);
    f.bloque(no);
    let previo = previo(f);
    let puente_no = f.etiqueta();
    f.saltar(puente_no);
    f.bloque(puente_no);
    f.saltar(fin);
    f.bloque(fin);
    f.phi(t.v3, &[(rgb, puente), (previo, puente_no)])
}
fn leer(f: &mut Funcion, t: &Tipos, v: &Volumen, id: Id, canal: u32) -> Id {
    let entero = f.modulo().tipo_u32();
    let cero = f.modulo().constante_u32(0);
    let base = f.modulo().constante_u32((if v.vfx { 20 } else { 17 }) * v.lado * v.lado * v.lado + canal * 128);
    let indice = f.binaria(op::I_ADD, entero, base, id);
    let ptr = f.acceso(v.puntero, v.buffer, &[cero, indice]);
    f.cargar(t.real, ptr)
}
fn splat(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, id: Id, camara: [Id; 2]) -> Id {
    let [ojo, direccion] = camara;
    let xyz = core::array::from_fn::<_, 3, _>(|i| leer(f, t, v, id, u32::try_from(i).unwrap_or(0)));
    let p = f.construir(t.v3, &xyz);
    let rel = f.binaria(op::F_SUB, t.v3, p, ojo);
    let proy = f.binaria(op::DOT, t.real, rel, direccion);
    let largo = f.binaria(op::DOT, t.real, rel, rel);
    let cuadrado = f.binaria(op::F_MUL, t.real, proy, proy);
    let distancia = f.binaria(op::F_SUB, t.real, largo, cuadrado);
    let cero = Matematica::escalar(f, 0.0);
    let distancia = f.glsl(t.real, op::GLSL_MAXIMO, &[distancia, cero]);
    let escala = Matematica::escalar(f, -2.0 / (0.008 * 0.008));
    let exponente = f.binaria(op::F_MUL, t.real, distancia, escala);
    let peso = f.glsl(t.real, 27, &[exponente]);
    let delante = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, proy, cero);
    let peso = f.elegir(t.real, delante, peso, cero);
    let edad = leer(f, t, v, id, 6);
    let medio = if v.vfx {
        let vida = crate::vfx_volumen::ajuste(f, t, v, 6);
        let uno = Matematica::escalar(f, 1.0);
        f.binaria(op::F_DIV, t.real, uno, vida)
    } else {
        Matematica::escalar(f, 0.5)
    };
    let edad = f.binaria(op::F_MUL, t.real, edad, medio);
    let uno = Matematica::escalar(f, 1.0);
    let joven = f.binaria(op::F_SUB, t.real, uno, edad);
    let azul = Matematica::escalar(f, 0.2);
    let rgb = f.construir(t.v3, &[uno, joven, azul]);
    let rgb = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, rgb, peso);
    if v.vfx { crate::vfx_estelas::sumar(f, t, mat, v, id, [ojo, direccion, p, rgb]) } else { rgb }
}
