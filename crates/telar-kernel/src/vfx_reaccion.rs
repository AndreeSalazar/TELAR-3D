//! Modelo acotado de combustible/ignición; calor normalizado, sin cinética química de producción.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
pub(super) fn ajustar(f: &mut Funcion, m: &Memoria, pixel: Id, valor: Id, canal: u32) -> Id {
    let coords = m.coords3(f, pixel).map(|v| f.unaria(op::U_A_F, m.real, v));
    let indice = Memoria::u(f, 10);
    let dt = m.leer(f, indice, 16);
    let medio = Memoria::c(f, 0.5);
    let velocidad = core::array::from_fn::<_, 3, _>(|eje| {
        let mut p = coords;
        p[eje] = m.sub(f, p[eje], medio);
        m.muestra3(f, p, u32::try_from(eje).unwrap_or(0))
    });
    let escala = m.escala(f, dt, m.lado as f32);
    let anterior = core::array::from_fn(|eje| {
        let delta = m.mul(f, velocidad[eje], escala);
        m.sub(f, coords[eje], delta)
    });
    let fuel = m.muestra3(f, anterior, 17);
    let mascara = crate::humo3d_fuente::mascara(f, m, pixel);
    let (_, pulso) = crate::humo3d_fuente::pulso(f, m);
    let fuente = m.mul(f, mascara, pulso);
    let fuente = m.mul(f, fuente, dt);
    let tasa = m.ajuste(f, 0);
    let aporte = m.mul(f, fuente, tasa);
    let fuel = m.add(f, fuel, aporte);
    let fuel = m.limitar(f, fuel, 0.0, 10.0);
    let calor = m.muestra3(f, anterior, 4);
    let semilla = m.parametro(f, 1);
    let semilla = m.mul(f, semilla, fuente);
    let calor = m.add(f, calor, semilla);
    let umbral = m.ajuste(f, 2);
    let exceso = m.sub(f, calor, umbral);
    let cero = Memoria::c(f, 0.0);
    let exceso = f.glsl(m.real, op::GLSL_MAXIMO, &[exceso, cero]);
    let tasa = m.ajuste(f, 1);
    let tasa = m.mul(f, tasa, dt);
    let consumido = m.mul(f, tasa, exceso);
    let consumido = f.glsl(m.real, 37, &[consumido, fuel]);
    let rendimiento = m.ajuste(f, if canal == 3 { 4 } else { 3 });
    let aporte = m.mul(f, consumido, rendimiento);
    let valor = m.add(f, valor, aporte);
    let valor = if canal == 4 {
        let restante = m.sub(f, fuel, consumido);
        let perdida = m.ajuste(f, 26);
        let perdida = m.mul(f, perdida, dt);
        let uno = Memoria::c(f, 1.0);
        let conserva = m.sub(f, uno, perdida);
        let conserva = m.limitar(f, conserva, 0.0, 1.0);
        let restante = m.mul(f, restante, conserva);
        let fluido = m.fluido(f, pixel);
        let restante = f.elegir(m.real, fluido, restante, cero);
        // GPU: cada celda escribe solo su combustible provisional; se publica antes de proyección.
        m.guardar(f, pixel, 18, restante);
        let perdida = m.ajuste(f, 5);
        let perdida = m.mul(f, perdida, dt);
        let uno = Memoria::c(f, 1.0);
        let conservar = m.sub(f, uno, perdida);
        let conservar = f.glsl(m.real, op::GLSL_MAXIMO, &[conservar, cero]);
        m.mul(f, valor, conservar)
    } else {
        valor
    };
    m.limitar(f, valor, 0.0, 10.0)
}

#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
pub(super) fn detalle(f: &mut Funcion, m: &Memoria, coords: [Id; 3], canal: u32) -> Id {
    let eje = match canal {
        0 => 2,
        1 => 0,
        _ => 1,
    };
    let coord = m.escala(f, coords[eje], 1.0 / m.lado as f32);
    let frecuencia = m.ajuste(f, 25);
    let fase = m.mul(f, coord, frecuencia);
    let fase = m.escala(f, fase, core::f32::consts::TAU);
    let indice = Memoria::u(f, 16);
    let tiempo = m.leer(f, indice, 16);
    let tiempo = m.escala(f, tiempo, 2.0);
    let fase = m.add(f, fase, tiempo);
    let detalle = f.glsl(m.real, 14, &[fase]);
    let fase2 = m.escala(f, fase, 2.0);
    let segundo = f.glsl(m.real, 14, &[fase2]);
    let segundo = m.escala(f, segundo, 0.5);
    let detalle = m.add(f, detalle, segundo);
    let intensidad = m.ajuste(f, 24);
    let detalle = m.mul(f, detalle, intensidad);
    let calor = m.muestra3(f, coords, 4);
    m.mul(f, detalle, calor)
}
