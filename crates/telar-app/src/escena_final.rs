//! Escena de ensayo: suelo y dos cajas independientes; se prepara una sola vez.
use nudo::mate::{cruz, normalizar, restar};
use telar_render::escena_rayos::{Camara, Triangulo};

pub(crate) fn camara(ojo: [f32; 3], objetivo: [f32; 3], aspecto: f32) -> Result<Camara, String> {
    let delta = restar(objetivo, ojo);
    if delta.iter().map(|v| v * v).sum::<f32>() < 1e-8 {
        return Err("ojo y objetivo deben ser distintos".into());
    }
    let frente = normalizar(delta);
    let horizontal = cruz(frente, [0.0, 1.0, 0.0]);
    if horizontal.iter().map(|v| v * v).sum::<f32>() < 1e-8 {
        return Err("cámara paralela al eje vertical".into());
    }
    let horizontal = normalizar(horizontal);
    let vertical = cruz(horizontal, frente);
    Ok(Camara {
        origen: ojo,
        frente,
        derecha: horizontal.map(|v| v * 0.5 * aspecto),
        arriba: vertical.map(|v| v * 0.5),
    })
}

pub(crate) fn triangulos(desplazamiento: f32) -> Vec<Triangulo> {
    let mut escena = Vec::new();
    cuadrado(&mut escena, [[-4.0, 0.0, -4.0], [4.0, 0.0, -4.0], [4.0, 0.0, 4.0], [-4.0, 0.0, 4.0]], [0.65, 0.65, 0.65]);
    caja(&mut escena, [-0.85 + desplazamiento, 0.65, 0.0], [0.6, 0.65, 0.6], [0.8, 0.18, 0.07]);
    caja(&mut escena, [0.9, 0.4, -0.55], [0.45, 0.4, 0.45], [0.08, 0.42, 0.65]);
    escena
}

fn cuadrado(escena: &mut Vec<Triangulo>, puntos: [[f32; 3]; 4], color: [f32; 3]) {
    for indices in [[0, 1, 2], [0, 2, 3]] {
        escena.push(Triangulo { vertices: indices.map(|i| puntos[i]), color });
    }
}

fn caja(escena: &mut Vec<Triangulo>, centro: [f32; 3], radio: [f32; 3], color: [f32; 3]) {
    let puntos: [[_; 3]; 8] = core::array::from_fn(|indice| {
        core::array::from_fn(|eje| centro[eje] + if indice & (1 << eje) == 0 { -radio[eje] } else { radio[eje] })
    });
    for cara in [[0, 1, 3, 2], [4, 6, 7, 5], [0, 4, 5, 1], [2, 3, 7, 6], [0, 2, 6, 4], [1, 5, 7, 3]] {
        cuadrado(escena, cara.map(|i| puntos[i]), color);
    }
}
