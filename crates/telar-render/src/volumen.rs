//! Inspección volumétrica 3D por emisión/absorción; no ilumina todavía humo de producción.
use crate::{
    tipos::{Tipos, variable},
    trazado::Matematica,
    volumen_muestreo::Volumen,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
/// ABI de campos MAC 3D de diecisiete planos. Constantes físicas [0..13],
/// cámara: frente [16..19], derecha [20..23], arriba [24..27], ojo [28..31].
/// # Errors
/// Rechaza lados fuera de 4..64 o pasos de integración fuera de 16..128.
pub fn fragmentos(lado: u32, pasos: u32) -> Result<Vec<u32>, &'static str> {
    if !(4..=64).contains(&lado) || !(16..=128).contains(&pasos) {
        return Err("visor volumétrico fuera de presupuesto");
    }
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let volumen = Volumen::nuevo(&mut m, &t, lado);
    let v2 = m.tipo_vector(t.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let uv = f.cargar(v2, entrada);
    let bloque = t.cargar(&mut f, 1);
    let expo = f.extraer(t.real, bloque, 2);
    let vista = f.extraer(t.real, bloque, 3);
    let opciones = t.cargar(&mut f, 2);
    let srgb = f.extraer(t.real, opciones, 1);
    let origen4 = t.cargar(&mut f, 7);
    let origen = t.xyz(&mut f, origen4);
    let direccion = direccion(&mut f, &t, &mat, uv);
    let rgb = integrar(&mut f, &t, &mat, &volumen, origen, direccion, vista, pasos);
    let color = crate::superficie::salida(&mut f, &t, rgb, expo, srgb);
    let color =
        crate::humo::indicador(&mut f, &t, &mat, volumen.buffer, volumen.puntero, uv, 16 * lado * lado * lado, color);
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    Ok(m.ensamblar())
}
pub(super) fn direccion(f: &mut Funcion, t: &Tipos, mat: &Matematica, uv: Id) -> Id {
    let base = t.cargar(f, 4);
    let mut direccion = t.xyz(f, base);
    let uno = Matematica::escalar(f, 1.0);
    let dos = Matematica::escalar(f, 2.0);
    for eje in 0..2 {
        let coord = f.extraer(t.real, uv, eje);
        let coord = f.binaria(op::F_MUL, t.real, coord, dos);
        let coord =
            if eje == 0 { f.binaria(op::F_SUB, t.real, coord, uno) } else { f.binaria(op::F_SUB, t.real, uno, coord) };
        let base = t.cargar(f, eje + 5);
        let base = t.xyz(f, base);
        let avance = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, base, coord);
        direccion = f.binaria(op::F_ADD, t.v3, direccion, avance);
    }
    mat.normalizar(f, direccion)
}
#[allow(clippy::too_many_arguments, clippy::cast_precision_loss, reason = "ABI explícita del integrador, pasos <=128")]
fn integrar(
    f: &mut Funcion,
    t: &Tipos,
    mat: &Matematica,
    v: &Volumen,
    origen: Id,
    direccion: Id,
    vista: Id,
    pasos: u32,
) -> Id {
    let cero = Matematica::escalar(f, 0.0);
    let uno = Matematica::escalar(f, 1.0);
    let centro = mat.vector(f, [0.5; 3]);
    let relativo = f.binaria(op::F_SUB, t.v3, origen, centro);
    let proyeccion = f.binaria(op::DOT, t.real, relativo, direccion);
    let radio2 = Matematica::escalar(f, 0.9 * 0.9);
    let distancia = f.binaria(op::DOT, t.real, relativo, relativo);
    let constante = f.binaria(op::F_SUB, t.real, distancia, radio2);
    let proyeccion2 = f.binaria(op::F_MUL, t.real, proyeccion, proyeccion);
    let discriminante = f.binaria(op::F_SUB, t.real, proyeccion2, constante);
    let discriminante = f.glsl(t.real, op::GLSL_MAXIMO, &[discriminante, cero]);
    let raiz = f.glsl(t.real, 31, &[discriminante]);
    let negativo = f.binaria(op::F_SUB, t.real, cero, proyeccion);
    let inicio = f.binaria(op::F_SUB, t.real, negativo, raiz);
    let inicio = f.glsl(t.real, op::GLSL_MAXIMO, &[inicio, cero]);
    let fin = f.binaria(op::F_ADD, t.real, negativo, raiz);
    let largo = f.binaria(op::F_SUB, t.real, fin, inicio);
    let largo = f.glsl(t.real, op::GLSL_MAXIMO, &[largo, cero]);
    let cantidad = Matematica::escalar(f, pasos as f32);
    let paso = f.binaria(op::F_DIV, t.real, largo, cantidad);
    let inicial = f.construir(t.v4, &[cero, cero, cero, uno]);
    let resultado = f.repetir(pasos, t.v4, inicial, |f, indice, previo| {
        let indice = f.unaria(op::U_A_F, t.real, indice);
        let medio = Matematica::escalar(f, 0.5);
        let indice = f.binaria(op::F_ADD, t.real, indice, medio);
        let avance = f.binaria(op::F_MUL, t.real, indice, paso);
        let avance = f.binaria(op::F_ADD, t.real, inicio, avance);
        let avance = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, direccion, avance);
        let punto = f.binaria(op::F_ADD, t.v3, origen, avance);
        let valor = v.campo(f, t, mat, punto, vista);
        let solido = v.solido(f, t, mat, punto);
        let opaco = Matematica::escalar(f, 40.0);
        let densidad = f.glsl(t.real, 4, &[valor]);
        let densidad = f.elegir(t.real, solido, opaco, densidad);
        let densidad = f.binaria(op::F_MUL, t.real, densidad, paso);
        let factor = Matematica::escalar(f, -6.0);
        let espesor = f.binaria(op::F_MUL, t.real, densidad, factor);
        let transmision = f.glsl(t.real, 27, &[espesor]);
        let alfa = f.binaria(op::F_SUB, t.real, uno, transmision);
        let antes_t = f.extraer(t.real, previo, 3);
        let ponderacion = f.binaria(op::F_MUL, t.real, antes_t, alfa);
        let color = color_campo(f, mat, vista, valor);
        let obstaculo = mat.vector(f, [0.12, 0.25, 0.35]);
        let color = mat.elegir_vector(f, solido, obstaculo, color);
        let aporte = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, ponderacion);
        let previo_rgb = t.xyz(f, previo);
        let rgb = f.binaria(op::F_ADD, t.v3, previo_rgb, aporte);
        let nuevo_t = f.binaria(op::F_MUL, t.real, antes_t, transmision);
        let canales = core::array::from_fn::<_, 3, _>(|i| f.extraer(t.real, rgb, u32::try_from(i).unwrap_or(0)));
        f.construir(t.v4, &[canales[0], canales[1], canales[2], nuevo_t])
    });
    let rgb = t.xyz(f, resultado);
    let fondo = mat.vector(f, [0.008, 0.012, 0.018]);
    let transmision = f.extraer(t.real, resultado, 3);
    let fondo = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, fondo, transmision);
    f.binaria(op::F_ADD, t.v3, rgb, fondo)
}
fn color_campo(f: &mut Funcion, mat: &Matematica, vista: Id, valor: Id) -> Id {
    let mut color = mat.vector(f, [0.65, 0.7, 0.78]);
    for (indice, (codigo, rgb)) in [(1.0, [1.0, 0.25, 0.03]), (2.0, [0.03, 0.7, 0.8])].into_iter().enumerate() {
        let alternativa = if indice == 0 { color_calor(f, mat, valor) } else { mat.vector(f, rgb) };
        let codigo = Matematica::escalar(f, codigo);
        let coincide = f.binaria(180, mat.booleano, vista, codigo);
        color = mat.elegir_vector(f, coincide, alternativa, color);
    }
    let tres = Matematica::escalar(f, 3.0);
    let div = f.binaria(180, mat.booleano, vista, tres);
    let cero = Matematica::escalar(f, 0.0);
    let positivo = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, valor, cero);
    let rojo = mat.vector(f, [0.9, 0.05, 0.02]);
    let azul = mat.vector(f, [0.02, 0.15, 0.9]);
    let signo = mat.elegir_vector(f, positivo, rojo, azul);
    mat.elegir_vector(f, div, signo, color)
}
/// Paleta artística de calor normalizado; no es radiancia de cuerpo negro.
fn color_calor(f: &mut Funcion, mat: &Matematica, valor: Id) -> Id {
    let cero = Matematica::escalar(f, 0.0);
    let uno = Matematica::escalar(f, 1.0);
    let canales = [(0.0, 2.0), (0.1, 0.4), (1.0, 0.2)].map(|(inicio, escala)| {
        let inicio = Matematica::escalar(f, inicio);
        let escala = Matematica::escalar(f, escala);
        let canal = f.binaria(op::F_SUB, mat.real, valor, inicio);
        let canal = f.binaria(op::F_MUL, mat.real, canal, escala);
        f.glsl(mat.real, 43, &[canal, cero, uno])
    });
    f.construir(mat.vector, &canales)
}
