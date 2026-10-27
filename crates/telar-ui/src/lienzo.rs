//! Geometría de interfaz y glifos de atlas; fuente 5×7 de respaldo para pruebas.
use alloc::vec::Vec;

pub(super) struct Lienzo<'a> {
    pub(super) vertices: Vec<[f32; 8]>,
    tamano: (f32, f32),
    fuente: Option<&'a crate::Fuente>,
}

impl<'a> Lienzo<'a> {
    fn triangulo(&mut self, puntos: [[f32; 2]; 3], color: [f32; 3]) {
        for punto in puntos {
            self.vertices.push([
                punto[0] * 2.0 / self.tamano.0 - 1.0,
                punto[1] * 2.0 / self.tamano.1 - 1.0,
                0.0,
                0.0,
                color[0],
                color[1],
                color[2],
                1.0,
            ]);
        }
    }

    pub(super) fn linea(&mut self, a: [f32; 2], b: [f32; 2], grosor: f32, color: [f32; 3]) {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let mut longitud = dx.abs().max(dy.abs());
        if longitud == 0.0 {
            return;
        }
        // Newton sobre una distancia de interfaz, sin dependencia matemática externa.
        for _ in 0..4 {
            longitud = f32::midpoint(longitud, (dx * dx + dy * dy) / longitud);
        }
        let (nx, ny) = (-dy / longitud * grosor * 0.5, dx / longitud * grosor * 0.5);
        let puntos = [[a[0] + nx, a[1] + ny], [b[0] + nx, b[1] + ny], [b[0] - nx, b[1] - ny], [a[0] - nx, a[1] - ny]];
        self.triangulo([puntos[0], puntos[1], puntos[2]], color);
        self.triangulo([puntos[0], puntos[2], puntos[3]], color);
    }

    pub(super) fn hilo(&mut self, a: [f32; 2], b: [f32; 2], color: [f32; 3]) {
        let medio = f32::midpoint(a[1], b[1]);
        let mut previo = a;
        for paso in 1..=24u16 {
            let t = f32::from(paso) / 24.0;
            let u = 1.0 - t;
            let punto = [
                u * u * u * a[0] + 3.0 * u * u * t * a[0] + 3.0 * u * t * t * b[0] + t * t * t * b[0],
                u * u * u * a[1] + 3.0 * u * u * t * medio + 3.0 * u * t * t * medio + t * t * t * b[1],
            ];
            self.linea(previo, punto, 1.5, color);
            previo = punto;
        }
    }

    pub(super) fn redondo(&mut self, rect: crate::Rectangulo, radio: f32, color: [f32; 3]) {
        let r = radio.min(rect.ancho * 0.5).min(rect.alto * 0.5);
        let centro = [rect.x + rect.ancho * 0.5, rect.y + rect.alto * 0.5];
        let mut puntos = Vec::with_capacity(20);
        for esquina in 0..4 {
            for (seno, coseno) in [
                (0.0, 1.0),
                (0.382_683, 0.923_880),
                (core::f32::consts::FRAC_1_SQRT_2, core::f32::consts::FRAC_1_SQRT_2),
                (0.923_880, 0.382_683),
                (1.0, 0.0),
            ] {
                let (x, y) = match esquina {
                    0 => (rect.x + rect.ancho - r + seno * r, rect.y + r - coseno * r),
                    1 => (rect.x + rect.ancho - r + coseno * r, rect.y + rect.alto - r + seno * r),
                    2 => (rect.x + r - seno * r, rect.y + rect.alto - r + coseno * r),
                    _ => (rect.x + r - coseno * r, rect.y + r - seno * r),
                };
                puntos.push([x, y]);
            }
        }
        for indice in 0..puntos.len() {
            self.triangulo([centro, puntos[indice], puntos[(indice + 1) % puntos.len()]], color);
        }
    }
    #[allow(clippy::cast_precision_loss, reason = "dimensiones de la ventana en píxeles")]
    pub(super) fn nuevo(tamano: (u32, u32), fuente: Option<&'a crate::Fuente>) -> Self {
        Self { vertices: Vec::new(), tamano: (tamano.0.max(1) as f32, tamano.1.max(1) as f32), fuente }
    }

    pub(super) fn rect(&mut self, x: f32, y: f32, ancho: f32, alto: f32, color: [f32; 3]) {
        let puntos = [[x, y], [x + ancho, y], [x + ancho, y + alto], [x, y + alto]];
        for indice in [0, 1, 2, 0, 2, 3] {
            let punto = puntos[indice];
            self.vertices.push([
                punto[0] * 2.0 / self.tamano.0 - 1.0,
                punto[1] * 2.0 / self.tamano.1 - 1.0,
                0.0,
                0.0,
                color[0],
                color[1],
                color[2],
                1.0,
            ]);
        }
    }

    pub(super) fn texto(&mut self, x: f32, y: f32, texto: &str, escala: f32, color: [f32; 3]) {
        if self.fuente.is_some() {
            self.texto_suave(x, y, texto, escala, color);
            return;
        }
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

    #[allow(clippy::cast_precision_loss, reason = "métricas del atlas y coordenadas de glifos")]
    fn texto_suave(&mut self, x: f32, y: f32, texto: &str, escala: f32, color: [f32; 3]) {
        let Some(fuente) = self.fuente else {
            return;
        };
        let factor = escala * 12.0 / fuente.altura;
        let celda = fuente.celda as f32;
        let mut cursor = x;
        for letra in texto.chars() {
            let codigo = u32::from(letra)
                .checked_sub(fuente.primero)
                .filter(|i| (*i as usize) < fuente.avances.len())
                .unwrap_or(u32::from('?') - fuente.primero);
            let columnas = fuente.ancho / fuente.celda;
            if letra == ' ' {
                cursor += fuente.avances[codigo as usize] * factor;
                continue;
            }
            let (u, v) = ((codigo % columnas * fuente.celda) as f32, (codigo / columnas * fuente.celda) as f32);
            let (izquierda, arriba, lado) = (cursor - 2.0 * factor, y - 2.0 * factor, celda * factor);
            let esquinas = [
                [izquierda, arriba, u, v],
                [izquierda + lado, arriba, u + celda, v],
                [izquierda + lado, arriba + lado, u + celda, v + celda],
                [izquierda, arriba + lado, u, v + celda],
            ];
            for indice in [0, 1, 2, 0, 2, 3] {
                let punto = esquinas[indice];
                self.vertices.push([
                    punto[0] * 2.0 / self.tamano.0 - 1.0,
                    punto[1] * 2.0 / self.tamano.1 - 1.0,
                    punto[2],
                    punto[3],
                    color[0],
                    color[1],
                    color[2],
                    1.0,
                ]);
            }
            cursor += fuente.avances[codigo as usize] * factor;
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
