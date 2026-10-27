//! Geometría de interfaz. Fuente 5×7 provisional, dibujada por la GPU sin dependencias.
use alloc::vec::Vec;

pub(super) struct Lienzo {
    pub(super) vertices: Vec<[f32; 6]>,
    tamano: (f32, f32),
}

impl Lienzo {
    #[allow(clippy::cast_precision_loss, reason = "dimensiones de la ventana en píxeles")]
    pub(super) fn nuevo(tamano: (u32, u32)) -> Self {
        Self { vertices: Vec::new(), tamano: (tamano.0.max(1) as f32, tamano.1.max(1) as f32) }
    }

    pub(super) fn rect(&mut self, x: f32, y: f32, ancho: f32, alto: f32, color: [f32; 3]) {
        let puntos = [[x, y], [x + ancho, y], [x + ancho, y + alto], [x, y + alto]];
        for indice in [0, 1, 2, 0, 2, 3] {
            let punto = puntos[indice];
            self.vertices.push([
                punto[0] * 2.0 / self.tamano.0 - 1.0,
                punto[1] * 2.0 / self.tamano.1 - 1.0,
                0.0,
                color[0],
                color[1],
                color[2],
            ]);
        }
    }

    pub(super) fn texto(&mut self, x: f32, y: f32, texto: &str, escala: f32, color: [f32; 3]) {
        let mut cursor = x;
        for letra in texto.chars() {
            for (fila, bits) in glifo(letra).into_iter().enumerate() {
                for columna in 0..5u8 {
                    if bits & (1 << (4 - columna)) != 0 {
                        #[allow(clippy::cast_precision_loss, reason = "siete filas del glifo")]
                        self.rect(
                            cursor + f32::from(columna) * escala,
                            y + fila as f32 * escala,
                            escala,
                            escala,
                            color,
                        );
                    }
                }
            }
            cursor += 6.0 * escala;
        }
    }
}

fn glifo(letra: char) -> [u8; 7] {
    match letra.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        _ => [0; 7],
    }
}
