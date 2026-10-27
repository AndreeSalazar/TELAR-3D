//! Recorte de triángulos UI en NDC; solo al recomponer el lienzo.
use alloc::vec::Vec;

pub(crate) fn recortar(vertices: &mut Vec<[f32; 8]>, inicio: usize, limites: [f32; 4]) {
    let originales = vertices.split_off(inicio);
    for triangulo in originales.as_chunks::<3>().0 {
        let mut poligono = triangulo.to_vec();
        for (eje, limite, signo) in
            [(0, limites[0], 1.0), (0, limites[2], -1.0), (1, limites[1], 1.0), (1, limites[3], -1.0)]
        {
            if poligono.is_empty() {
                break;
            }
            let mut salida = Vec::new();
            let mut anterior = poligono[poligono.len() - 1];
            for actual in poligono {
                let da = (anterior[eje] - limite) * signo;
                let db = (actual[eje] - limite) * signo;
                if (da >= 0.0) != (db >= 0.0) {
                    let fraccion = da / (da - db);
                    let mut corte = core::array::from_fn(|i| anterior[i] + (actual[i] - anterior[i]) * fraccion);
                    corte[eje] = limite;
                    salida.push(corte);
                }
                if db >= 0.0 {
                    salida.push(actual);
                }
                anterior = actual;
            }
            poligono = salida;
        }
        for i in 1..poligono.len().saturating_sub(1) {
            vertices.extend([poligono[0], poligono[i], poligono[i + 1]]);
        }
    }
}
