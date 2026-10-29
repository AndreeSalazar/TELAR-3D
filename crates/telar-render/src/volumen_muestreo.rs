//! Lectura trilineal de campos residentes con coordenadas normalizadas del dominio.
use crate::{tipos::Tipos, trazado::Matematica};
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
pub(super) struct Volumen {
    pub buffer: Id,
    pub puntero: Id,
    pub lado: u32,
    pub vfx: bool,
}
impl Volumen {
    pub(super) fn nuevo(m: &mut Modulo, t: &Tipos, lado: u32) -> Self {
        Self::crear(m, t, lado, true)
    }
    pub(super) fn nuevo_calculo(m: &mut Modulo, t: &Tipos, lado: u32) -> Self {
        Self::crear(m, t, lado, false)
    }
    fn crear(m: &mut Modulo, t: &Tipos, lado: u32, lectura: bool) -> Self {
        let array = m.tipo_array_dinamico(t.real, 4);
        let bloque = m.tipo_bloque(&[array], &[0]);
        let ptr = m.tipo_puntero(Clase::Almacen, bloque);
        let buffer = m.variable(ptr, Clase::Almacen);
        m.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
        m.decorar(buffer, op::DECORACION_ENLACE, &[1]);
        if lectura {
            m.decorar(buffer, 24, &[]);
        }
        Self { buffer, puntero: m.tipo_puntero(Clase::Almacen, t.real), lado, vfx: false }
    }
    #[allow(clippy::cast_precision_loss, reason = "lado <=64")]
    pub(super) fn muestra(&self, f: &mut Funcion, t: &Tipos, mat: &Matematica, punto: Id, canal: Id) -> Id {
        let cero = f.modulo().constante_f32(0.0);
        let uno = f.modulo().constante_f32(1.0);
        let medio = f.modulo().constante_f32(0.5);
        let escala = f.modulo().constante_f32(self.lado as f32);
        let max = f.modulo().constante_f32((self.lado - 1) as f32);
        let mut bases = [canal; 3];
        let mut pesos = [cero; 3];
        let mut dentro = f.binaria(180, mat.booleano, cero, cero);
        for eje in 0..3 {
            let coord = f.extraer(t.real, punto, u32::try_from(eje).unwrap_or(0));
            let inicio = f.binaria(190, mat.booleano, coord, cero);
            let fin = f.binaria(188, mat.booleano, coord, uno);
            let valido = f.binaria(op::LOGICAL_AND, mat.booleano, inicio, fin);
            dentro = f.binaria(op::LOGICAL_AND, mat.booleano, dentro, valido);
            let coord = f.binaria(op::F_MUL, t.real, coord, escala);
            let coord = f.binaria(op::F_SUB, t.real, coord, medio);
            let coord = f.glsl(t.real, 43, &[coord, cero, max]);
            bases[eje] = f.unaria(op::F_A_U, mat.entero, coord);
            let base = f.unaria(op::U_A_F, t.real, bases[eje]);
            pesos[eje] = f.binaria(op::F_SUB, t.real, coord, base);
        }
        let lado = f.modulo().constante_u32(self.lado);
        let limite = f.modulo().constante_u32(self.lado - 1);
        let cero_u = f.modulo().constante_u32(0);
        let celdas = f.modulo().constante_u32(self.lado * self.lado * self.lado);
        let plano = f.binaria(op::I_MUL, mat.entero, canal, celdas);
        let mut valores = [cero; 8];
        for (i, valor) in valores.iter_mut().enumerate() {
            let indices = core::array::from_fn::<_, 3, _>(|eje| {
                let d = f.modulo().constante_u32(u32::try_from((i >> eje) & 1).unwrap_or(0));
                let p = f.binaria(op::I_ADD, mat.entero, bases[eje], d);
                f.glsl(mat.entero, 38, &[p, limite])
            });
            let capa = f.binaria(op::I_MUL, mat.entero, indices[2], lado);
            let fila = f.binaria(op::I_ADD, mat.entero, capa, indices[1]);
            let fila = f.binaria(op::I_MUL, mat.entero, fila, lado);
            let pixel = f.binaria(op::I_ADD, mat.entero, fila, indices[0]);
            let indice = f.binaria(op::I_ADD, mat.entero, pixel, plano);
            let ptr = f.acceso(self.puntero, self.buffer, &[cero_u, indice]);
            *valor = f.cargar(t.real, ptr);
        }
        for (eje, peso) in pesos.into_iter().enumerate() {
            for i in 0..(4 >> eje) {
                let a = valores[i * 2];
                let b = valores[i * 2 + 1];
                let delta = f.binaria(op::F_SUB, t.real, b, a);
                let delta = f.binaria(op::F_MUL, t.real, delta, peso);
                valores[i] = f.binaria(op::F_ADD, t.real, a, delta);
            }
        }
        f.elegir(t.real, dentro, valores[0], cero)
    }
    #[allow(clippy::cast_precision_loss, reason = "lado <=64")]
    pub(super) fn solido(&self, f: &mut Funcion, t: &Tipos, mat: &Matematica, punto: Id) -> Id {
        let bloque = t.cargar(f, 3);
        let radio = f.extraer(t.real, bloque, 2);
        let altura = f.extraer(t.real, bloque, 3);
        let medio = Matematica::escalar(f, 0.5);
        let escala = Matematica::escalar(f, self.lado as f32);
        let cero = Matematica::escalar(f, 0.0);
        let mut distancia = cero;
        for eje in 0..3 {
            let coord = f.extraer(t.real, punto, eje);
            let coord = f.binaria(op::F_MUL, t.real, coord, escala);
            let coord = f.glsl(t.real, 8, &[coord]);
            let coord = f.binaria(op::F_ADD, t.real, coord, medio);
            let coord = f.binaria(op::F_DIV, t.real, coord, escala);
            let delta = f.binaria(op::F_SUB, t.real, coord, if eje == 1 { altura } else { medio });
            let cuadrado = f.binaria(op::F_MUL, t.real, delta, delta);
            distancia = f.binaria(op::F_ADD, t.real, distancia, cuadrado);
        }
        let radio2 = f.binaria(op::F_MUL, t.real, radio, radio);
        let dentro = f.binaria(188, mat.booleano, distancia, radio2);
        let activo = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, radio, cero);
        f.binaria(op::LOGICAL_AND, mat.booleano, dentro, activo)
    }
    pub(super) fn campo(&self, f: &mut Funcion, t: &Tipos, mat: &Matematica, punto: Id, vista: Id) -> Id {
        let dos = f.modulo().constante_f32(2.0);
        let es_velocidad = f.binaria(180, mat.booleano, vista, dos);
        let velocidad = f.etiqueta();
        let escalar = f.etiqueta();
        let fin = f.etiqueta();
        f.fusion(fin);
        f.saltar_si(es_velocidad, velocidad, escalar);
        f.bloque(velocidad);
        let mut suma = f.modulo().constante_f32(0.0);
        for canal in 0..3 {
            let canal = f.modulo().constante_u32(canal);
            let valor = self.muestra(f, t, mat, punto, canal);
            let cuadrado = f.binaria(op::F_MUL, t.real, valor, valor);
            suma = f.binaria(op::F_ADD, t.real, suma, cuadrado);
        }
        let rapidez = f.glsl(t.real, 31, &[suma]);
        let factor = f.modulo().constante_f32(4.0);
        let rapidez = f.binaria(op::F_MUL, t.real, rapidez, factor);
        f.saltar(fin);
        f.bloque(escalar);
        let uno = f.modulo().constante_f32(1.0);
        let tres = f.modulo().constante_f32(3.0);
        let calor = f.binaria(180, mat.booleano, vista, uno);
        let div = f.binaria(180, mat.booleano, vista, tres);
        let densidad = f.modulo().constante_u32(3);
        let temperatura = f.modulo().constante_u32(4);
        let divergencia = f.modulo().constante_u32(13);
        let canal = f.elegir(mat.entero, calor, temperatura, densidad);
        let canal = f.elegir(mat.entero, div, divergencia, canal);
        let valor = self.muestra(f, t, mat, punto, canal);
        f.saltar(fin);
        f.bloque(fin);
        f.phi(t.real, &[(rapidez, velocidad), (valor, escalar)])
    }
}
