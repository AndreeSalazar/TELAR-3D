//! Esfera analítica del colisionador: corte primario y visibilidad de segmentos en GPU.
use crate::{tipos::Tipos, trazado::Matematica, volumen_muestreo::Volumen};
use hilo_spirv::{Funcion, Id, op};

pub(super) fn distancia(f: &mut Funcion, t: &Tipos, mat: &Matematica, ojo: Id, dir: Id) -> Id {
    let parametros = t.cargar(f, 3);
    let radio = f.extraer(t.real, parametros, 2);
    let altura = f.extraer(t.real, parametros, 3);
    let medio = Matematica::escalar(f, 0.5);
    let centro = f.construir(t.v3, &[medio, altura, medio]);
    let rel = f.binaria(op::F_SUB, t.v3, ojo, centro);
    let b = f.binaria(op::DOT, t.real, rel, dir);
    let largo2 = f.binaria(op::DOT, t.real, rel, rel);
    let radio2 = f.binaria(op::F_MUL, t.real, radio, radio);
    let c = f.binaria(op::F_SUB, t.real, largo2, radio2);
    let b2 = f.binaria(op::F_MUL, t.real, b, b);
    let discriminante = f.binaria(op::F_SUB, t.real, b2, c);
    let cero = Matematica::escalar(f, 0.0);
    let raiz = f.glsl(t.real, op::GLSL_MAXIMO, &[discriminante, cero]);
    let raiz = f.glsl(t.real, 31, &[raiz]);
    let menos = Matematica::escalar(f, -1.0);
    let negativo_b = f.binaria(op::F_MUL, t.real, b, menos);
    let cerca = f.binaria(op::F_SUB, t.real, negativo_b, raiz);
    let lejos = f.binaria(op::F_ADD, t.real, negativo_b, raiz);
    let cerca = f.glsl(t.real, op::GLSL_MAXIMO, &[cerca, cero]);
    let delante = f.binaria(190, mat.booleano, lejos, cero);
    let cruza = f.binaria(190, mat.booleano, discriminante, cero);
    let activo = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, radio, cero);
    let valido = f.binaria(op::LOGICAL_AND, mat.booleano, delante, cruza);
    let valido = f.binaria(op::LOGICAL_AND, mat.booleano, valido, activo);
    let infinito = Matematica::escalar(f, 1e6);
    f.elegir(t.real, valido, cerca, infinito)
}
pub(super) fn libre(f: &mut Funcion, t: &Tipos, mat: &Matematica, origen: Id, destino: Id) -> Id {
    let delta = f.binaria(op::F_SUB, t.v3, destino, origen);
    let largo2 = f.binaria(op::DOT, t.real, delta, delta);
    let largo = f.glsl(t.real, 31, &[largo2]);
    let minimo = Matematica::escalar(f, 1e-12);
    let den = f.glsl(t.real, op::GLSL_MAXIMO, &[largo2, minimo]);
    let inv = f.glsl(t.real, 32, &[den]);
    let dir = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, delta, inv);
    let hit = distancia(f, t, mat, origen, dir);
    let sesgo = Matematica::escalar(f, 1e-4);
    let limite = f.binaria(op::F_SUB, t.real, largo, sesgo);
    f.binaria(190, mat.booleano, hit, limite)
}
#[allow(clippy::too_many_arguments, reason = "cámara e intervalo primario explícitos")]
pub(super) fn limitar(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    v: &Volumen,
    camara: [Id; 3],
    inicio: Id,
) -> [Id; 2] {
    let [ojo, dir, vista] = camara;
    let impacto = distancia(f, t, mat, ojo, dir);
    let largo = Matematica::escalar(f, 1.8);
    let limite = f.binaria(op::F_ADD, t.real, inicio, largo);
    let corta = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, limite, impacto);
    let cero = Matematica::escalar(f, 0.0);
    let belleza = f.binaria(180, mat.booleano, vista, cero);
    let corta = f.binaria(op::LOGICAL_AND, mat.booleano, corta, belleza);
    let si = f.etiqueta();
    let no = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(corta, si, no);
    f.bloque(si);
    let offset = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, dir, impacto);
    let punto = f.binaria(op::F_ADD, t.v3, ojo, offset);
    let color = color(f, t, mat, v, punto);
    let recortado = f.binaria(op::F_SUB, t.real, impacto, inicio);
    let recortado = f.glsl(t.real, op::GLSL_MAXIMO, &[recortado, cero]);
    let puente = f.etiqueta();
    f.saltar(puente);
    f.bloque(puente);
    f.saltar(fin);
    f.bloque(no);
    let fondo = mat.vector(f, [0.008, 0.012, 0.018]);
    f.saltar(fin);
    f.bloque(fin);
    [f.phi(t.real, &[(recortado, puente), (largo, no)]), f.phi(t.v3, &[(color, puente), (fondo, no)])]
}
fn color(f: &mut Funcion, t: &Tipos, mat: &Matematica, v: &Volumen, punto: Id) -> Id {
    let parametros = t.cargar(f, 3);
    let altura = f.extraer(t.real, parametros, 3);
    let medio = Matematica::escalar(f, 0.5);
    let centro = f.construir(t.v3, &[medio, altura, medio]);
    let normal = f.binaria(op::F_SUB, t.v3, punto, centro);
    let normal2 = f.binaria(op::DOT, t.real, normal, normal);
    let epsilon = Matematica::escalar(f, 1e-12);
    let normal2 = f.glsl(t.real, op::GLSL_MAXIMO, &[normal2, epsilon]);
    let inv_normal = f.glsl(t.real, 32, &[normal2]);
    let normal = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, normal, inv_normal);
    let xyz = core::array::from_fn::<_, 3, _>(|eje| {
        crate::vfx_volumen::ajuste(f, t, v, 12 + u32::try_from(eje).unwrap_or(0))
    });
    let luz = f.construir(t.v3, &xyz);
    let hacia = f.binaria(op::F_SUB, t.v3, luz, punto);
    let r2 = f.binaria(op::DOT, t.real, hacia, hacia);
    let minimo = Matematica::escalar(f, 0.01);
    let den = f.glsl(t.real, op::GLSL_MAXIMO, &[r2, minimo]);
    let inv = f.glsl(t.real, 32, &[den]);
    let dir = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, hacia, inv);
    let cos = f.binaria(op::DOT, t.real, normal, dir);
    let cero = Matematica::escalar(f, 0.0);
    let cos = f.glsl(t.real, op::GLSL_MAXIMO, &[cos, cero]);
    let intensidad = crate::vfx_volumen::ajuste(f, t, v, 15);
    let intensidad = f.binaria(op::F_DIV, t.real, intensidad, den);
    let intensidad = f.binaria(op::F_MUL, t.real, intensidad, cos);
    let sesgo = Matematica::escalar(f, 0.001);
    let offset = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, normal, sesgo);
    let origen = f.binaria(op::F_ADD, t.v3, punto, offset);
    let hacia_sombra = f.binaria(op::F_SUB, t.v3, luz, origen);
    let distancia2 = f.binaria(op::DOT, t.real, hacia_sombra, hacia_sombra);
    let distancia = f.glsl(t.real, 31, &[distancia2]);
    let sombra = crate::vfx_volumen::sombra(f, t, mat, v, origen, hacia_sombra, distancia);
    let intensidad = f.binaria(op::F_MUL, t.real, intensidad, sombra);
    let pi = Matematica::escalar(f, core::f32::consts::PI);
    let intensidad = f.binaria(op::F_DIV, t.real, intensidad, pi);
    let ambiente = Matematica::escalar(f, 0.04);
    let intensidad = f.binaria(op::F_ADD, t.real, intensidad, ambiente);
    let base = mat.vector(f, [0.18, 0.25, 0.32]);
    f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, base, intensidad)
}
