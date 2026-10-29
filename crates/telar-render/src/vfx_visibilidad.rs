//! Transmisión hacia secundarios: integración acotada solo para coberturas significativas.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use hilo_spirv::{Funcion, Id, op};

pub(super) fn filtrar(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, datos: [Id; 3]) -> Id {
    let [ojo, punto, peso] = datos;
    let umbral = Matematica::escalar(f, 1e-5);
    let visible = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, peso, umbral);
    let libre = crate::vfx_esfera::libre(f, t, mat, ojo, punto);
    let visible = f.binaria(op::LOGICAL_AND, mat.booleano, visible, libre);
    let integrar = f.etiqueta();
    let omitir = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(visible, integrar, omitir);
    f.bloque(integrar);
    let transmision = transmitir(f, t, mat, v, ojo, punto);
    let atenuado = f.binaria(op::F_MUL, t.real, peso, transmision);
    let puente = f.etiqueta();
    f.saltar(puente);
    f.bloque(puente);
    f.saltar(fin);
    f.bloque(omitir);
    let cero = Matematica::escalar(f, 0.0);
    f.saltar(fin);
    f.bloque(fin);
    f.phi(t.real, &[(atenuado, puente), (cero, omitir)])
}
fn transmitir(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, ojo: Id, punto: Id) -> Id {
    let delta = f.binaria(op::F_SUB, t.v3, ojo, punto);
    let distancia2 = f.binaria(op::DOT, t.real, delta, delta);
    let distancia = f.glsl(t.real, 31, &[distancia2]);
    let minimo = Matematica::escalar(f, 1e-12);
    let divisor = f.glsl(t.real, op::GLSL_MAXIMO, &[distancia2, minimo]);
    let inverso = f.glsl(t.real, 32, &[divisor]);
    let dir = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, delta, inverso);
    // Secundarios e historial dentro del cubo: su diagonal es menor que 1.8.
    let diagonal = Matematica::escalar(f, 1.8);
    let largo = f.glsl(t.real, 37, &[distancia, diagonal]);
    let octavo = Matematica::escalar(f, 0.125);
    let paso = f.binaria(op::F_MUL, t.real, largo, octavo);
    let cero = Matematica::escalar(f, 0.0);
    let canal = f.modulo().constante_u32(3);
    let ext = crate::vfx_volumen::ajuste(f, t, v, 16);
    let espesor = f.repetir(8, t.real, cero, |f, i, anterior| {
        let indice = f.unaria(op::U_A_F, t.real, i);
        let medio = Matematica::escalar(f, 0.5);
        let indice = f.binaria(op::F_ADD, t.real, indice, medio);
        let recorrido = f.binaria(op::F_MUL, t.real, indice, paso);
        let offset = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, dir, recorrido);
        let posicion = f.binaria(op::F_ADD, t.v3, punto, offset);
        let densidad = v.muestra(f, t, mat, posicion, canal);
        let sigma = f.binaria(op::F_MUL, t.real, densidad, ext);
        let aporte = f.binaria(op::F_MUL, t.real, sigma, paso);
        f.binaria(op::F_ADD, t.real, anterior, aporte)
    });
    let negativo = Matematica::escalar(f, -1.0);
    let espesor = f.binaria(op::F_MUL, t.real, espesor, negativo);
    f.glsl(t.real, 27, &[espesor])
}
