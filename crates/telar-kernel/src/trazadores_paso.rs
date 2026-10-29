//! Trazadores: leer caras proyectadas, interpolar y actualizar una plaza propia.
use crate::humo_memoria::Memoria;
use hilo_spirv::{Funcion, Id, op};
const N: u32 = crate::trazadores::CAPACIDAD;
fn leer(f: &mut Funcion, m: &Memoria, id: Id, canal: u32) -> Id {
    let base = Memoria::u(f, canal * N);
    let indice = f.binaria(op::I_ADD, m.entero, base, id);
    m.leer(f, indice, 17)
}
fn guardar(f: &mut Funcion, m: &Memoria, id: Id, canal: u32, valor: Id) {
    let base = Memoria::u(f, canal * N);
    let indice = f.binaria(op::I_ADD, m.entero, base, id);
    // GPU: solo esta invocación escribe su plaza en la cola; no modifica ningún campo MAC.
    m.guardar(f, indice, 17, valor);
}
pub(super) fn emitir(f: &mut Funcion, m: &Memoria, id: Id) {
    let cantidad = Memoria::u(f, N);
    let dentro = f.binaria(op::U_LESS_THAN, m.booleano, id, cantidad);
    let longitud = f.longitud(m.entero, m.buffer, 0);
    let minimo = Memoria::u(f, 17 * m.celdas() + crate::trazadores::PALABRAS);
    let cabe = f.binaria(op::U_LESS_THAN_EQUAL, m.booleano, minimo, longitud);
    let dentro = f.binaria(op::LOGICAL_AND, m.booleano, dentro, cabe);
    let cuerpo = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(dentro, cuerpo, fin);
    f.bloque(cuerpo);
    let previo = core::array::from_fn::<_, 11, _>(|i| leer(f, m, id, u32::try_from(i).unwrap_or(0)));
    let valido = validar(f, m, &previo);
    let bien = f.etiqueta();
    let mal = f.etiqueta();
    let union = f.etiqueta();
    f.fusion(union);
    f.saltar_si(valido, bien, mal);
    f.bloque(bien);
    avanzar(f, m, id, previo);
    f.saltar(union);
    f.bloque(mal);
    let menos = Memoria::c(f, -1.0);
    guardar(f, m, id, 10, menos);
    f.saltar(union);
    f.bloque(union);
    f.saltar(fin);
    f.bloque(fin);
}
fn control(f: &mut Funcion, m: &Memoria, i: u32) -> Id {
    let indice = Memoria::u(f, i);
    m.leer(f, indice, 16)
}
fn validar(f: &mut Funcion, m: &Memoria, previo: &[Id; 11]) -> Id {
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let inactiva = f.binaria(180, m.booleano, previo[10], cero);
    let activa = f.binaria(180, m.booleano, previo[10], uno);
    let mut valido = f.binaria(166, m.booleano, inactiva, activa);
    let limite = Memoria::c(f, 1e20);
    for valor in previo.iter().take(10) {
        let abs = f.glsl(m.real, 4, &[*valor]);
        let finito = f.binaria(188, m.booleano, abs, limite);
        valido = f.binaria(op::LOGICAL_AND, m.booleano, valido, finito);
    }
    for i in [6, 7, 8, 9] {
        let positivo = f.binaria(190, m.booleano, previo[i], cero);
        valido = f.binaria(op::LOGICAL_AND, m.booleano, valido, positivo);
    }
    let max = Memoria::c(f, 65535.0);
    let menor = f.binaria(188, m.booleano, previo[8], max);
    let entero = f.glsl(m.real, 8, &[previo[8]]);
    let entero = f.binaria(180, m.booleano, previo[8], entero);
    valido = f.binaria(op::LOGICAL_AND, m.booleano, valido, menor);
    f.binaria(op::LOGICAL_AND, m.booleano, valido, entero)
}
fn avanzar(f: &mut Funcion, m: &Memoria, id: Id, previo: [Id; 11]) {
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let tiempo = control(f, m, 16); // Congelado antes del subpaso; el diagnóstico escribe otro reloj.
    let dt = control(f, m, 10);
    let estado = control(f, m, 5);
    let tres = Memoria::c(f, 3.0);
    let seguro = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, tres, estado);
    let avanza = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, dt, cero);
    let avanza = f.binaria(op::LOGICAL_AND, m.booleano, seguro, avanza);
    let cuerpo = f.etiqueta();
    let fin = f.etiqueta();
    f.fusion(fin);
    f.saltar_si(avanza, cuerpo, fin);
    f.bloque(cuerpo);
    actualizar(f, m, id, previo, [tiempo, dt, cero, uno]);
    f.saltar(fin);
    f.bloque(fin);
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64")]
fn actualizar(f: &mut Funcion, m: &Memoria, id: Id, previo: [Id; 11], reloj: [Id; 4]) {
    let [tiempo, dt, cero, uno] = reloj;
    let duracion = m.parametro(f, 8);
    let continuo = f.binaria(180, m.booleano, duracion, cero);
    let ventana = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, duracion, tiempo);
    let ventana = f.binaria(166, m.booleano, continuo, ventana);
    let inactiva = f.binaria(180, m.booleano, previo[10], cero);
    let limite = Memoria::c(f, 65535.0);
    let libre = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, limite, previo[8]);
    let nace = f.binaria(op::LOGICAL_AND, m.booleano, ventana, inactiva);
    let nace = f.binaria(op::LOGICAL_AND, m.booleano, nace, libre);
    let nueva = origen(f, m, id);
    let posicion = core::array::from_fn::<_, 3, _>(|i| f.elegir(m.real, nace, nueva[i], previo[i]));
    let velocidad = core::array::from_fn::<_, 3, _>(|eje| {
        let mut coords = posicion.map(|valor| m.escala(f, valor, m.lado as f32));
        for (i, coord) in coords.iter_mut().enumerate() {
            let offset = Memoria::c(f, if i == eje { 1.0 } else { 0.5 });
            *coord = m.sub(f, *coord, offset);
        }
        m.muestra3(f, coords, u32::try_from(eje).unwrap_or(0))
    });
    let limite = Memoria::c(f, 1e20);
    let mut finita = f.binaria(180, m.booleano, cero, cero);
    let velocidad = velocidad.map(|valor| {
        let abs = f.glsl(m.real, 4, &[valor]);
        let valido = f.binaria(188, m.booleano, abs, limite);
        finita = f.binaria(op::LOGICAL_AND, m.booleano, finita, valido);
        f.elegir(m.real, valido, valor, cero)
    });
    let nueva_p = core::array::from_fn::<_, 3, _>(|i| {
        let delta = m.mul(f, velocidad[i], dt);
        m.add(f, posicion[i], delta)
    });
    let vida = Memoria::c(f, 2.0);
    let edad = f.elegir(m.real, nace, cero, previo[6]);
    let edad = m.add(f, edad, dt);
    let edad = f.glsl(m.real, 37, &[edad, vida]);
    let generacion = m.add(f, previo[8], uno);
    let generacion = f.elegir(m.real, nace, generacion, previo[8]);
    let nacimiento = f.elegir(m.real, nace, tiempo, previo[9]);
    let activa = f.binaria(180, m.booleano, previo[10], uno);
    let mueve = f.binaria(166, m.booleano, activa, nace);
    let joven = f.binaria(op::F_ORD_GREATER_THAN, m.booleano, vida, edad);
    let dentro = destino(f, m, nueva_p);
    let encendida = f.binaria(op::LOGICAL_AND, m.booleano, joven, dentro);
    let encendida = f.binaria(op::LOGICAL_AND, m.booleano, encendida, mueve);
    let encendida = f.elegir(m.real, encendida, uno, cero);
    let menos = Memoria::c(f, -1.0);
    let encendida = f.elegir(m.real, finita, encendida, menos);
    let valores = [
        nueva_p[0],
        nueva_p[1],
        nueva_p[2],
        velocidad[0],
        velocidad[1],
        velocidad[2],
        edad,
        vida,
        generacion,
        nacimiento,
    ];
    for (i, valor) in valores.into_iter().enumerate() {
        let valor = f.elegir(m.real, mueve, valor, previo[i]);
        let valor = f.elegir(m.real, finita, valor, previo[i]);
        guardar(f, m, id, u32::try_from(i).unwrap_or(0), valor);
    }
    guardar(f, m, id, 10, encendida);
}
#[allow(clippy::cast_precision_loss, reason = "retícula de 4x8x4 dentro del emisor")]
fn origen(f: &mut Funcion, m: &Memoria, id: Id) -> [Id; 3] {
    let cuatro = Memoria::u(f, 4);
    let ocho = Memoria::u(f, 8);
    let columna = f.binaria(op::U_MOD, m.entero, id, cuatro);
    let fila = f.binaria(op::U_DIV, m.entero, id, cuatro);
    let fila_y = f.binaria(op::U_MOD, m.entero, fila, ocho);
    let plano = f.binaria(op::U_DIV, m.entero, fila, ocho);
    let radio = m.parametro(f, 0);
    let coords = [columna, fila_y, plano];
    core::array::from_fn(|i| {
        let valor = coords[i];
        let valor = f.unaria(op::U_A_F, m.real, valor);
        let valor = m.escala(f, valor, if i == 1 { 1.0 / 7.0 } else { 1.0 / 3.0 });
        let medio = Memoria::c(f, 0.5);
        let valor = m.sub(f, valor, medio);
        let valor = m.mul(f, valor, radio);
        let centro = Memoria::c(f, if i == 1 { 0.18 + 0.5 / m.lado as f32 } else { 0.5 });
        m.add(f, valor, centro)
    })
}
#[allow(clippy::cast_precision_loss, reason = "lado <=64 fila_y conversión tras clamp")]
fn destino(f: &mut Funcion, m: &Memoria, posicion: [Id; 3]) -> Id {
    let cero = Memoria::c(f, 0.0);
    let uno = Memoria::c(f, 1.0);
    let mut valido = f.binaria(180, m.booleano, cero, cero);
    let indices = posicion.map(|valor| {
        let min = f.binaria(190, m.booleano, valor, cero);
        let max = f.binaria(188, m.booleano, valor, uno);
        let dentro = f.binaria(op::LOGICAL_AND, m.booleano, min, max);
        valido = f.binaria(op::LOGICAL_AND, m.booleano, valido, dentro);
        let valor = m.limitar(f, valor, 0.0, 1.0);
        let valor = m.escala(f, valor, m.lado as f32);
        let valor = f.unaria(op::F_A_U, m.entero, valor);
        let max = Memoria::u(f, m.lado - 1);
        f.glsl(m.entero, 38, &[valor, max])
    });
    let lado = Memoria::u(f, m.lado);
    let plano = f.binaria(op::I_MUL, m.entero, indices[2], lado);
    let fila = f.binaria(op::I_ADD, m.entero, plano, indices[1]);
    let fila = f.binaria(op::I_MUL, m.entero, fila, lado);
    let pixel = f.binaria(op::I_ADD, m.entero, fila, indices[0]);
    let fluido = m.fluido(f, pixel);
    f.binaria(op::LOGICAL_AND, m.booleano, valido, fluido)
}
