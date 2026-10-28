//! Volumen procedural de vista previa: emisión/absorción, sin solver de fluidos.
use crate::{
    tipos::{Tipos, variable},
    trazado::Matematica,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

/// Marcha acotada. ABI: radio, turbulencia, emisión, absorción; tiempo, semilla,
/// exposición, aspecto; vista (0 belleza, 1 densidad, 2 temperatura, 3 llama), sRGB.
/// Los valores deben validarse antes de subirlos; no reserva campos voxelizados.
/// # Errors
/// Rechaza calidades fuera de 16..128 pasos por píxel.
#[allow(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "pasos <=128; un punto de entrada SSA con integración estructurada"
)]
pub fn fragmentos(pasos: u32) -> Result<Vec<u32>, &'static str> {
    if !(16..=128).contains(&pasos) {
        return Err("fuego: pasos fuera de 16..128");
    }
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let v2 = m.tipo_vector(t.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let fuente = t.cargar(&mut f, 0);
    let ajustes = t.cargar(&mut f, 1);
    let vista = t.cargar(&mut f, 2);
    let uv = f.cargar(v2, entrada);
    let constantes = [0.0, 1.0, 2.0, 4.0, 0.62].map(|v| Matematica::escalar(&mut f, v));
    let aspecto = f.extraer(t.real, ajustes, 3);
    let xy: [_; 2] = core::array::from_fn(|i| {
        let valor = f.extraer(t.real, uv, u32::try_from(i).unwrap_or(0));
        let valor = f.binaria(op::F_MUL, t.real, valor, constantes[2]);
        let valor = if i == 0 {
            f.binaria(op::F_SUB, t.real, valor, constantes[1])
        } else {
            f.binaria(op::F_SUB, t.real, constantes[1], valor)
        };
        let valor = f.binaria(op::F_MUL, t.real, valor, constantes[4]);
        if i == 0 { f.binaria(op::F_MUL, t.real, valor, aspecto) } else { valor }
    });
    let menos_uno = Matematica::escalar(&mut f, -1.0);
    let direccion = f.construir(t.v3, &[xy[0], xy[1], menos_uno]);
    let direccion = mat.normalizar(&mut f, direccion);
    let origen = mat.vector(&mut f, [0.0, 0.0, 4.0]);
    let eje = f.extraer(t.real, direccion, 2);
    let proyeccion = f.binaria(op::F_MUL, t.real, eje, constantes[3]);
    let proyeccion2 = f.binaria(op::F_MUL, t.real, proyeccion, proyeccion);
    let distancia = Matematica::escalar(&mut f, 16.0 - 2.4 * 2.4);
    let discriminante = f.binaria(op::F_SUB, t.real, proyeccion2, distancia);
    let existe = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, discriminante, constantes[0]);
    let discriminante = f.glsl(t.real, op::GLSL_MAXIMO, &[discriminante, constantes[0]]);
    let raiz = f.glsl(t.real, 31, &[discriminante]);
    let negativo_b = f.binaria(op::F_SUB, t.real, constantes[0], proyeccion);
    let cerca = f.binaria(op::F_SUB, t.real, negativo_b, raiz);
    let largo = f.binaria(op::F_MUL, t.real, raiz, constantes[2]);
    let cantidad = Matematica::escalar(&mut f, pasos as f32);
    let paso = f.binaria(op::F_DIV, t.real, largo, cantidad);
    let paso = f.elegir(t.real, existe, paso, constantes[0]);
    let inicial = f.construir(t.v4, &[constantes[0], constantes[0], constantes[0], constantes[1]]);
    let resultado = f.repetir(pasos, t.v4, inicial, |f, indice, acumulado| {
        let indice = f.unaria(op::U_A_F, t.real, indice);
        let medio = Matematica::escalar(f, 0.5);
        let indice = f.binaria(op::F_ADD, t.real, indice, medio);
        let distancia = f.binaria(op::F_MUL, t.real, indice, paso);
        let distancia = f.binaria(op::F_ADD, t.real, cerca, distancia);
        let desplazamiento = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, direccion, distancia);
        let punto = f.binaria(op::F_ADD, t.v3, origen, desplazamiento);
        let [densidad, temperatura, llama] = campos(f, &t, punto, fuente, ajustes);
        let modo = f.extraer(t.real, vista, 0);
        let mut color = color_fuego(f, &t, temperatura);
        let emision = f.extraer(t.real, fuente, 2);
        let brillo = f.binaria(op::F_MUL, t.real, emision, llama);
        color = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, brillo);
        for (n, valor) in [(1.0, densidad), (2.0, temperatura), (3.0, llama)] {
            let n = Matematica::escalar(f, n);
            let igual = f.binaria(180, mat.booleano, modo, n);
            let diagnostico = f.construir(t.v3, &[valor; 3]);
            color = mat.elegir_vector(f, igual, diagnostico, color);
        }
        let absorcion = f.extraer(t.real, fuente, 3);
        let base = Matematica::escalar(f, 0.5);
        let sigma = f.binaria(op::F_ADD, t.real, absorcion, base);
        let sigma = f.binaria(op::F_MUL, t.real, sigma, densidad);
        let espesor = f.binaria(op::F_MUL, t.real, sigma, paso);
        let espesor = f.binaria(op::F_SUB, t.real, constantes[0], espesor);
        let transmision = f.glsl(t.real, 27, &[espesor]);
        let alfa = f.binaria(op::F_SUB, t.real, constantes[1], transmision);
        let anterior_t = f.extraer(t.real, acumulado, 3);
        let ponderacion = f.binaria(op::F_MUL, t.real, anterior_t, alfa);
        let aporte = f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, color, ponderacion);
        let anterior = t.xyz(f, acumulado);
        let rgb = f.binaria(op::F_ADD, t.v3, anterior, aporte);
        let siguiente_t = f.binaria(op::F_MUL, t.real, anterior_t, transmision);
        let componentes: [_; 3] = core::array::from_fn(|i| f.extraer(t.real, rgb, u32::try_from(i).unwrap_or(0)));
        f.construir(t.v4, &[componentes[0], componentes[1], componentes[2], siguiente_t])
    });
    let rgb = t.xyz(&mut f, resultado);
    let exposicion = f.extraer(t.real, ajustes, 2);
    let codificar = f.extraer(t.real, vista, 1);
    let color = crate::superficie::salida(&mut f, &t, rgb, exposicion, codificar);
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    Ok(m.ensamblar())
}

fn campos(f: &mut Funcion, t: &Tipos, punto: Id, fuente: Id, ajustes: Id) -> [Id; 3] {
    let radio = f.extraer(t.real, fuente, 0);
    let turbulencia = f.extraer(t.real, fuente, 1);
    let tiempo = f.extraer(t.real, ajustes, 0);
    let semilla = f.extraer(t.real, ajustes, 1);
    // Bases oblicuas evitan que la trama sinusoidal siga los ejes de la esfera.
    let coordenadas = [[0.8, 0.36, 0.48], [-0.48, 0.85, 0.21], [-0.33, -0.4, 0.85]].map(|base| {
        let componentes = base.map(|v| Matematica::escalar(f, v));
        let base = f.construir(t.v3, &componentes);
        f.binaria(op::DOT, t.real, punto, base)
    });
    let mut ruido = Matematica::escalar(f, 0.0);
    for (frecuencia, amplitud) in [(5.7, 0.55), (11.3, 0.3), (23.1, 0.15)] {
        let mut producto = Matematica::escalar(f, amplitud);
        for (i, coord) in coordenadas.into_iter().enumerate() {
            let escala = Matematica::escalar(f, frecuencia);
            let coord = f.binaria(op::F_MUL, t.real, coord, escala);
            let fase = if i == 1 { tiempo } else { semilla };
            let coord = f.binaria(op::F_SUB, t.real, coord, fase);
            let seno = f.glsl(t.real, 13, &[coord]);
            producto = f.binaria(op::F_MUL, t.real, producto, seno);
        }
        ruido = f.binaria(op::F_ADD, t.real, ruido, producto);
    }
    let r2 = f.binaria(op::DOT, t.real, punto, punto);
    let r = f.glsl(t.real, 31, &[r2]);
    let perturbacion = f.binaria(op::F_MUL, t.real, ruido, turbulencia);
    let borde = f.binaria(op::F_SUB, t.real, radio, r);
    let borde = f.binaria(op::F_ADD, t.real, borde, perturbacion);
    let cuatro = Matematica::escalar(f, 4.0);
    let densidad = f.binaria(op::F_MUL, t.real, borde, cuatro);
    let densidad = t.saturar(f, densidad);
    let relativo = f.binaria(op::F_DIV, t.real, r, radio);
    let medio = Matematica::escalar(f, 0.5);
    let relativo = f.binaria(op::F_MUL, t.real, relativo, medio);
    let uno = Matematica::escalar(f, 1.0);
    let temperatura = f.binaria(op::F_SUB, t.real, uno, relativo);
    let temperatura = f.binaria(op::F_ADD, t.real, temperatura, perturbacion);
    let temperatura = t.saturar(f, temperatura);
    let llama = f.binaria(op::F_ADD, t.real, ruido, medio);
    let llama = t.saturar(f, llama);
    [densidad, temperatura, llama]
}
fn color_fuego(f: &mut Funcion, t: &Tipos, temperatura: Id) -> Id {
    // Paleta artística HDR; no es una conversión física de kelvin con Planck/CIE.
    let rojo = Matematica::escalar(f, 1.0);
    let verde = f.binaria(op::F_MUL, t.real, temperatura, temperatura);
    let verde = f.binaria(op::F_MUL, t.real, verde, temperatura);
    let azul = f.binaria(op::F_MUL, t.real, verde, verde);
    let azul = f.binaria(op::F_MUL, t.real, azul, verde);
    let escala = Matematica::escalar(f, 0.35);
    let azul = f.binaria(op::F_MUL, t.real, azul, escala);
    f.construir(t.v3, &[rojo, verde, azul])
}
