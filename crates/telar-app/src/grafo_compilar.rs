//! Fusión de operaciones geométricas alcanzables en un kernel HILO por punto.
use hilo::{Id, Operacion as Op, Programa, Salida};
use telar_ui::{Estudio, OperacionRed};

struct Constructor {
    operaciones: Vec<Op>,
}
impl Constructor {
    fn emitir(&mut self, op: Op) -> Id {
        let id = Id(self.operaciones.len());
        self.operaciones.push(op);
        id
    }
}

pub(crate) fn compilar(estudio: &Estudio) -> Result<Programa, String> {
    let mut constructor = Constructor { operaciones: Vec::new() };
    let posicion = constructor.emitir(Op::Posicion);
    let x = constructor.emitir(Op::Extraer(posicion, 0));
    let z = constructor.emitir(Op::Extraer(posicion, 2));
    let dos = constructor.emitir(Op::Escalar(2.0));
    let x = constructor.emitir(Op::Multiplicar(x, dos));
    let z = constructor.emitir(Op::Multiplicar(z, dos));
    let tiempo = constructor.emitir(Op::Fase);
    let cero = constructor.emitir(Op::Escalar(0.0));
    let (mut altura, mut dx, mut dz) = (cero, cero, cero);
    for operacion in estudio.operaciones_red()? {
        match operacion {
            OperacionRed::Onda { amplitud, velocidad } => {
                let velocidad = constructor.emitir(Op::Escalar(velocidad));
                let fase = constructor.emitir(Op::Multiplicar(tiempo, velocidad));
                let eje = constructor.emitir(Op::Sumar(x, fase));
                let sx = constructor.emitir(Op::Seno(eje));
                let cx = constructor.emitir(Op::Coseno(eje));
                let sz = constructor.emitir(Op::Seno(z));
                let cz = constructor.emitir(Op::Coseno(z));
                let a = constructor.emitir(Op::Escalar(amplitud));
                let doble = constructor.emitir(Op::Escalar(2.0 * amplitud));
                let negativo = constructor.emitir(Op::Escalar(-2.0 * amplitud));
                let h = constructor.emitir(Op::Multiplicar(a, sx));
                let h = constructor.emitir(Op::Multiplicar(h, cz));
                altura = constructor.emitir(Op::Sumar(altura, h));
                let derivada = constructor.emitir(Op::Multiplicar(doble, cx));
                let derivada = constructor.emitir(Op::Multiplicar(derivada, cz));
                dx = constructor.emitir(Op::Sumar(dx, derivada));
                let derivada = constructor.emitir(Op::Multiplicar(negativo, sx));
                let derivada = constructor.emitir(Op::Multiplicar(derivada, sz));
                dz = constructor.emitir(Op::Sumar(dz, derivada));
            }
            OperacionRed::Relieve { factor } => {
                let factor = constructor.emitir(Op::Escalar(factor));
                altura = constructor.emitir(Op::Multiplicar(altura, factor));
                dx = constructor.emitir(Op::Multiplicar(dx, factor));
                dz = constructor.emitir(Op::Multiplicar(dz, factor));
            }
        }
    }
    let nx = constructor.emitir(Op::Restar(cero, dx));
    let nz = constructor.emitir(Op::Restar(cero, dz));
    let uno = constructor.emitir(Op::Escalar(1.0));
    let normal = constructor.emitir(Op::Componer([nx, uno, nz]));
    let normal = constructor.emitir(Op::Normalizar(normal));
    Programa::nuevo(constructor.operaciones, Salida { altura, normal }).map_err(|e| e.to_string())
}
