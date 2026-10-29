//! Emisión determinista, Euler semiimplícito, suelo y ciclo de vida por plaza.
use crate::particulas_memoria::Estado;
use hilo_spirv::{Funcion, Id, op};
pub(super) fn emitir(f: &mut Funcion, e: &Estado, id: Id) {
    let previo = core::array::from_fn::<_, 11, _>(|i| e.leer(f, id, u32::try_from(i).unwrap_or(0)));
    let cero = Estado::c(f, 0.0);
    let uno = Estado::c(f, 1.0);
    let valido = validar(f, e, &previo);
    let solicitado = e.p(f, 0);
    let realid = f.unaria(op::U_A_F, e.real, id);
    let habilitado = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, solicitado, realid);
    let antes_viva = f.binaria(180, e.booleano, previo[10], uno);
    let viva = f.binaria(op::LOGICAL_AND, e.booleano, antes_viva, habilitado);
    let tiempo = e.control(f, 0);
    let dt = e.dt(f);
    let fin = e.add(f, tiempo, dt);
    let duracion = e.p(f, 8);
    let continuo = f.binaria(180, e.booleano, duracion, cero);
    let ventana = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, duracion, tiempo);
    let ventana = f.binaria(166, e.booleano, continuo, ventana);
    let fase = Estado::c(f, 0.5);
    let fase = f.elegir(e.real, continuo, fase, duracion);
    let fase = e.mul(f, fase, realid);
    let fase = f.binaria(op::F_DIV, e.real, fase, solicitado);
    let inicial = f.binaria(180, e.booleano, previo[8], cero);
    let inicio = f.glsl(e.real, op::GLSL_MAXIMO, &[fase, tiempo]);
    let inicio = f.elegir(e.real, inicial, inicio, tiempo);
    let listo = f.binaria(190, e.booleano, fin, inicio);
    let inactiva = f.binaria(180, e.booleano, previo[10], cero);
    let limite = Estado::c(f, 65_535.0);
    let generacion = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, limite, previo[8]);
    let avanza = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, dt, cero);
    let nace = [habilitado, ventana, listo, inactiva, generacion, avanza, valido]
        .into_iter()
        .reduce(|a, b| f.binaria(op::LOGICAL_AND, e.booleano, a, b))
        .unwrap_or(valido);
    let generacion = f.elegir(e.real, valido, previo[8], cero);
    let nueva = nacer(f, e, id, generacion, inicio);
    let parcial = e.sub(f, fin, inicio);
    let delta = f.elegir(e.real, nace, parcial, dt);
    let base = core::array::from_fn(|i| f.elegir(e.real, nace, nueva[i], previo[i]));
    let movido = integrar(f, e, base, delta);
    let mueve = f.binaria(166, e.booleano, viva, nace);
    let dentro = dentro(f, e, &movido);
    let joven = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, movido[7], movido[6]);
    let activa = f.binaria(op::LOGICAL_AND, e.booleano, mueve, dentro);
    let activa = f.binaria(op::LOGICAL_AND, e.booleano, activa, joven);
    let activa = f.elegir(e.real, activa, uno, cero);
    let cuarentena = Estado::c(f, -1.0);
    let activa = f.elegir(e.real, valido, activa, cuarentena);
    for i in 0..10 {
        let valor = f.elegir(e.real, mueve, movido[i], previo[i]);
        let valor = f.elegir(e.real, valido, valor, previo[i]);
        e.guardar(f, id, u32::try_from(i).unwrap_or(0), valor);
    }
    e.guardar(f, id, 10, activa);
}
fn validar(f: &mut Funcion, e: &Estado, p: &[Id; 11]) -> Id {
    let cero = Estado::c(f, 0.0);
    let uno = Estado::c(f, 1.0);
    let limite = Estado::c(f, 1e20);
    let mut valido = f.binaria(180, e.booleano, p[10], cero);
    let viva = f.binaria(180, e.booleano, p[10], uno);
    valido = f.binaria(166, e.booleano, valido, viva);
    for valor in p.iter().take(10) {
        let abs = f.glsl(e.real, 4, &[*valor]);
        let finito = f.binaria(188, e.booleano, abs, limite);
        valido = f.binaria(op::LOGICAL_AND, e.booleano, valido, finito);
    }
    for i in [6, 7, 8, 9] {
        let positivo = f.binaria(190, e.booleano, p[i], cero);
        valido = f.binaria(op::LOGICAL_AND, e.booleano, valido, positivo);
    }
    let limite = Estado::c(f, 65_535.0);
    let generacion = f.binaria(188, e.booleano, p[8], limite);
    let entero = f.glsl(e.real, 8, &[p[8]]);
    let entero = f.binaria(180, e.booleano, p[8], entero);
    valido = f.binaria(op::LOGICAL_AND, e.booleano, valido, generacion);
    f.binaria(op::LOGICAL_AND, e.booleano, valido, entero)
}
fn nacer(f: &mut Funcion, e: &Estado, id: Id, generacion: Id, inicio: Id) -> [Id; 10] {
    let uno = Estado::c(f, 1.0);
    let generacion = e.add(f, generacion, uno);
    let gen_u = f.unaria(op::F_A_U, e.entero, generacion);
    let semilla = e.p(f, 7);
    let semilla = f.unaria(op::F_A_U, e.entero, semilla);
    let a = Estado::u(f, 747_796_405);
    let b = Estado::u(f, 2_891_336_453);
    let clave = f.binaria(op::I_MUL, e.entero, id, a);
    let mezcla = f.binaria(op::I_MUL, e.entero, gen_u, b);
    let clave = f.binaria(198, e.entero, clave, mezcla);
    let mut clave = f.binaria(198, e.entero, clave, semilla);
    let azar = core::array::from_fn::<_, 3, _>(|_| {
        let a = Estado::u(f, 1_664_525);
        let b = Estado::u(f, 1_013_904_223);
        let bits = Estado::u(f, 16);
        clave = f.binaria(op::I_MUL, e.entero, clave, a);
        clave = f.binaria(op::I_ADD, e.entero, clave, b);
        let alto = f.binaria(194, e.entero, clave, bits);
        let valor = f.unaria(op::U_A_F, e.real, alto);
        let factor = Estado::c(f, 1.0 / 65_535.0);
        e.mul(f, valor, factor)
    });
    let dos = Estado::c(f, 2.0);
    let medio = Estado::c(f, 0.5);
    let tres = Estado::c(f, 0.75);
    let vertical = e.mul(f, azar[2], medio);
    let vertical = e.add(f, tres, vertical);
    let vida = e.p(f, 1);
    let vida = e.mul(f, vida, vertical);
    let impulso = e.p(f, 2);
    let dispersion = e.p(f, 9);
    let lateral = e.mul(f, impulso, dispersion);
    let vx = e.mul(f, azar[0], dos);
    let vx = e.sub(f, vx, uno);
    let vx = e.mul(f, vx, lateral);
    let vz = e.mul(f, azar[1], dos);
    let vz = e.sub(f, vz, uno);
    let vz = e.mul(f, vz, lateral);
    let vy = e.mul(f, impulso, vertical);
    let altura = Estado::c(f, 0.18);
    let cero = Estado::c(f, 0.0);
    [medio, altura, medio, vx, vy, vz, cero, vida, generacion, inicio]
}
fn integrar(f: &mut Funcion, e: &Estado, mut p: [Id; 10], dt: Id) -> [Id; 10] {
    let uno = Estado::c(f, 1.0);
    let cero = Estado::c(f, 0.0);
    let gravedad = e.p(f, 3);
    let gravedad = e.mul(f, gravedad, dt);
    p[4] = e.sub(f, p[4], gravedad);
    let arrastre = e.p(f, 4);
    let perdida = e.mul(f, arrastre, dt);
    let conservar = e.sub(f, uno, perdida);
    let conservar = f.glsl(e.real, op::GLSL_MAXIMO, &[conservar, cero]);
    for eje in 0..3 {
        p[eje + 3] = e.mul(f, p[eje + 3], conservar);
        let avance = e.mul(f, p[eje + 3], dt);
        p[eje] = e.add(f, p[eje], avance);
    }
    let toca = f.binaria(op::F_ORD_GREATER_THAN, e.booleano, cero, p[1]);
    let rebote = Estado::c(f, -0.4);
    let rebote = e.mul(f, p[4], rebote);
    p[4] = f.elegir(e.real, toca, rebote, p[4]);
    p[1] = f.glsl(e.real, op::GLSL_MAXIMO, &[p[1], cero]);
    let edad = e.add(f, p[6], dt);
    p[6] = f.glsl(e.real, 37, &[edad, p[7]]);
    p
}
fn dentro(f: &mut Funcion, e: &Estado, p: &[Id; 10]) -> Id {
    let cero = Estado::c(f, 0.0);
    let uno = Estado::c(f, 1.0);
    let mut valido = f.binaria(188, e.booleano, p[1], uno);
    for i in [0, 2] {
        let min = f.binaria(190, e.booleano, p[i], cero);
        let max = f.binaria(188, e.booleano, p[i], uno);
        let eje = f.binaria(op::LOGICAL_AND, e.booleano, min, max);
        valido = f.binaria(op::LOGICAL_AND, e.booleano, valido, eje);
    }
    valido
}
