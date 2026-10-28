//! Historial HDR residente: suma RGB f32 y contador u32 por píxel, 16 B.
//! Set 0/binding 1; un propietario por píxel, sin atómicos ni lectura host.
use crate::{
    escena_rayos::Trabajo,
    tipos::{Tipos, variable},
    trazado::Matematica,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};

struct Memoria {
    buffer: Id,
    puntero: Id,
    entero: Id,
}
impl Memoria {
    fn nueva(m: &mut Modulo, solo_lectura: bool) -> Self {
        let entero = m.tipo_u32();
        let array = m.tipo_array_dinamico(entero, 4);
        let bloque = m.tipo_bloque(&[array], &[0]);
        let tipo = m.tipo_puntero(Clase::Almacen, bloque);
        let buffer = m.variable(tipo, Clase::Almacen);
        m.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
        m.decorar(buffer, op::DECORACION_ENLACE, &[1]);
        if solo_lectura {
            m.decorar(buffer, 24, &[]);
        }
        Self { buffer, puntero: m.tipo_puntero(Clase::Almacen, entero), entero }
    }
    fn direccion(&self, f: &mut Funcion, base: Id, canal: u32) -> Id {
        let cero = f.modulo().constante_u32(0);
        let canal = f.modulo().constante_u32(canal);
        let indice = f.binaria(op::I_ADD, self.entero, base, canal);
        f.acceso(self.puntero, self.buffer, &[cero, indice])
    }
    fn leer(&self, f: &mut Funcion, base: Id, canal: u32) -> Id {
        let direccion = self.direccion(f, base, canal);
        f.cargar(self.entero, direccion)
    }
}

/// Una muestra por píxel/llamada hasta `trabajo.muestras`. Grupo local X=64.
/// El propietario del historial debe inicializarlo a cero y ordenar sus accesos.
/// # Errors
/// Rechaza escena o viewport fuera de 1..1024 × 1..768.
pub fn acumular(trabajo: &Trabajo, tamano: [u32; 2]) -> Result<Vec<u32>, &'static str> {
    trabajo.validar()?;
    if !(1..=1024).contains(&tamano[0]) || !(1..=768).contains(&tamano[1]) {
        return Err("viewport fuera de presupuesto");
    }
    let mut m = Modulo::nuevo();
    let mat = Matematica::nueva(&mut m);
    let memoria = Memoria::nueva(&mut m, false);
    let vector = m.tipo_vector(mat.entero, 3);
    let tipo = m.tipo_puntero(Clase::Entrada, vector);
    let global = m.variable(tipo, Clase::Entrada);
    m.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let [cuatro, pixeles, limite, uno] = [4, tamano[0] * tamano[1], trabajo.muestras, 1].map(|v| m.constante_u32(v));
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let [entrada, cuerpo, actualizar, salida, fin] = core::array::from_fn(|_| f.etiqueta());
    f.bloque(entrada);
    let global_cargado = f.cargar(vector, global);
    let pixel = f.extraer(mat.entero, global_cargado, 0);
    let longitud = f.longitud(mat.entero, memoria.buffer, 0);
    let capacidad = f.binaria(op::U_DIV, mat.entero, longitud, cuatro);
    let cabe = f.binaria(op::U_LESS_THAN, mat.booleano, pixel, capacidad);
    let activo = f.binaria(op::U_LESS_THAN, mat.booleano, pixel, pixeles);
    let activo = f.binaria(op::LOGICAL_AND, mat.booleano, activo, cabe);
    f.fusion(fin);
    f.saltar_si(activo, cuerpo, fin);
    f.bloque(cuerpo);
    let base = f.binaria(op::I_MUL, mat.entero, pixel, cuatro);
    let contador = memoria.leer(&mut f, base, 3);
    let necesita = f.binaria(op::U_LESS_THAN, mat.booleano, contador, limite);
    f.fusion(salida);
    f.saltar_si(necesita, actualizar, salida);
    f.bloque(actualizar);
    let muestra = crate::muestra_rayos::muestra_dimension(&mut f, &mat, trabajo, pixel, contador, tamano);
    for canal in 0..3 {
        let previo = memoria.leer(&mut f, base, canal);
        let previo = f.unaria(op::BITCAST, mat.real, previo);
        let nuevo = f.extraer(mat.real, muestra, canal);
        let suma = f.binaria(op::F_ADD, mat.real, previo, nuevo);
        let bits = f.unaria(op::BITCAST, mat.entero, suma);
        let destino = memoria.direccion(&mut f, base, canal);
        // GPU: una invocación posee las cuatro palabras; barreras externas serializan fotogramas.
        f.guardar(destino, bits);
    }
    let contador = f.binaria(op::I_ADD, mat.entero, contador, uno);
    let destino = memoria.direccion(&mut f, base, 3);
    // GPU: publica el recuento de la misma muestra, sin atomicidad desordenada.
    f.guardar(destino, contador);
    f.saltar(salida);
    f.bloque(salida);
    f.saltar(fin);
    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    m.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    Ok(m.ensamblar())
}

/// Muestra la media del historial; sin trazado adicional ni escrituras.
/// Constantes: primer vec4 = exposición, codificar sRGB 0/1, ancho, alto.
/// Ancho/alto deben coincidir con la reserva; vertex shader `visor_rayos::vertices`.
#[must_use]
pub fn resolver() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let memoria = Memoria::nueva(&mut m, true);
    let v2 = m.tipo_vector(t.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let bloque = t.cargar(&mut f, 0);
    let uv = f.cargar(v2, entrada);
    let uno = f.modulo().constante_u32(1);
    let cuatro = f.modulo().constante_u32(4);
    let dimensiones: [_; 2] = core::array::from_fn(|i| {
        let real = f.extraer(t.real, bloque, u32::try_from(i + 2).unwrap_or(0));
        f.unaria(op::F_A_U, mat.entero, real)
    });
    let coords: [_; 2] = core::array::from_fn(|i| {
        let real = f.extraer(t.real, uv, u32::try_from(i).unwrap_or(0));
        let escala = f.unaria(op::U_A_F, t.real, dimensiones[i]);
        let real = f.binaria(op::F_MUL, t.real, real, escala);
        let entero = f.unaria(op::F_A_U, mat.entero, real);
        let maximo = f.binaria(op::I_SUB, mat.entero, dimensiones[i], uno);
        f.glsl(mat.entero, 38, &[entero, maximo])
    });
    let fila = f.binaria(op::I_MUL, mat.entero, coords[1], dimensiones[0]);
    let pixel = f.binaria(op::I_ADD, mat.entero, fila, coords[0]);
    let base = f.binaria(op::I_MUL, mat.entero, pixel, cuatro);
    let contador = memoria.leer(&mut f, base, 3);
    let contador = f.glsl(mat.entero, 41, &[contador, uno]);
    let divisor = f.unaria(op::U_A_F, t.real, contador);
    let canales: [_; 3] = core::array::from_fn(|i| {
        let valor = memoria.leer(&mut f, base, u32::try_from(i).unwrap_or(0));
        let real = f.unaria(op::BITCAST, t.real, valor);
        f.binaria(op::F_DIV, t.real, real, divisor)
    });
    let radiancia = f.construir(t.v3, &canales);
    let exposicion = f.extraer(t.real, bloque, 0);
    let codificar = f.extraer(t.real, bloque, 1);
    let color = crate::superficie::salida(&mut f, &t, radiancia, exposicion, codificar);
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    m.ensamblar()
}
