//! Visor diagnóstico de splats gaussianos emisivos, sin sombras ni raster de malla por partícula.
use crate::{
    tipos::{Tipos, variable},
    trazado::Matematica,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
/// Capacidad compartida por contrato, sin dependencia del render hacia el kernel.
pub const CAPACIDAD: u32 = 128;
/// Triángulo de pantalla del visor existente; campo de partículas en binding 1.
#[must_use]
pub fn fragmentos() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let t = Tipos::nuevos(&mut m);
    let mat = Matematica::nueva(&mut m);
    let array = m.tipo_array_dinamico(t.real, 4);
    let bloque = m.tipo_bloque(&[array], &[0]);
    let ptr = m.tipo_puntero(Clase::Almacen, bloque);
    let buffer = m.variable(ptr, Clase::Almacen);
    m.decorar(buffer, op::DECORACION_CONJUNTO, &[0]);
    m.decorar(buffer, op::DECORACION_ENLACE, &[1]);
    m.decorar(buffer, 24, &[]);
    let puntero = m.tipo_puntero(Clase::Almacen, t.real);
    let v2 = m.tipo_vector(t.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, t.v4, Clase::Salida, 0);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let uv = f.cargar(v2, entrada);
    let ojo4 = t.cargar(&mut f, 7);
    let ojo = t.xyz(&mut f, ojo4);
    let direccion = crate::volumen::direccion(&mut f, &t, &mat, uv);
    let datos = Datos { buffer, puntero };
    let fondo = mat.vector(&mut f, [0.008, 0.012, 0.018]);
    let radio4 = t.cargar(&mut f, 1);
    let radio = f.extraer(t.real, radio4, 2);
    let rgb = f.repetir(CAPACIDAD, t.v3, fondo, |f, id, previo| {
        let activa = datos.leer(f, &t, id, 10);
        let cero = Matematica::escalar(f, 0.0);
        let activa = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, activa, cero);
        let si = f.etiqueta();
        let no = f.etiqueta();
        let fin = f.etiqueta();
        f.fusion(fin);
        f.saltar_si(activa, si, no);
        f.bloque(si);
        let rgb = datos.splat(f, &t, &mat, id, [ojo, direccion, radio]);
        let suma = f.binaria(op::F_ADD, t.v3, previo, rgb);
        f.saltar(fin);
        f.bloque(no);
        f.saltar(fin);
        f.bloque(fin);
        f.phi(t.v3, &[(suma, si), (previo, no)])
    });
    let opciones = t.cargar(&mut f, 2);
    let expo = f.extraer(t.real, opciones, 2);
    let srgb = f.extraer(t.real, opciones, 3);
    let color = crate::superficie::salida(&mut f, &t, rgb, expo, srgb);
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    m.ensamblar()
}
struct Datos {
    buffer: Id,
    puntero: Id,
}
impl Datos {
    fn leer(&self, f: &mut Funcion, t: &Tipos, id: Id, canal: u32) -> Id {
        let cero = f.modulo().constante_u32(0);
        let base = f.modulo().constante_u32(canal * CAPACIDAD);
        let entero = f.modulo().tipo_u32();
        let indice = f.binaria(op::I_ADD, entero, base, id);
        let ptr = f.acceso(self.puntero, self.buffer, &[cero, indice]);
        f.cargar(t.real, ptr)
    }
    fn splat(&self, f: &mut Funcion, t: &Tipos, mat: &Matematica, id: Id, camara: [Id; 3]) -> Id {
        let [ojo, direccion, radio] = camara;
        let canales = core::array::from_fn::<_, 3, _>(|i| self.leer(f, t, id, u32::try_from(i).unwrap_or(0)));
        let punto = f.construir(t.v3, &canales);
        let relativo = f.binaria(op::F_SUB, t.v3, punto, ojo);
        let profundidad = f.binaria(op::DOT, t.real, relativo, direccion);
        let profundidad2 = f.binaria(op::F_MUL, t.real, profundidad, profundidad);
        let distancia = f.binaria(op::DOT, t.real, relativo, relativo);
        let distancia = f.binaria(op::F_SUB, t.real, distancia, profundidad2);
        let cero = Matematica::escalar(f, 0.0);
        let uno = Matematica::escalar(f, 1.0);
        let distancia = f.glsl(t.real, op::GLSL_MAXIMO, &[distancia, cero]);
        let radio2 = f.binaria(op::F_MUL, t.real, radio, radio);
        let minimo = Matematica::escalar(f, 1e-8);
        let radio2 = f.glsl(t.real, op::GLSL_MAXIMO, &[radio2, minimo]);
        let distancia = f.binaria(op::F_DIV, t.real, distancia, radio2);
        let menos = Matematica::escalar(f, -2.0);
        let exponente = f.binaria(op::F_MUL, t.real, distancia, menos);
        let peso = f.glsl(t.real, 27, &[exponente]);
        let delante = f.binaria(op::F_ORD_GREATER_THAN, mat.booleano, profundidad, cero);
        let peso = f.elegir(t.real, delante, peso, cero);
        let edad = self.leer(f, t, id, 6);
        let vida = self.leer(f, t, id, 7);
        let minimo = Matematica::escalar(f, 0.001);
        let vida = f.glsl(t.real, op::GLSL_MAXIMO, &[vida, minimo]);
        let fraccion = f.binaria(op::F_DIV, t.real, edad, vida);
        let calor = f.binaria(op::F_SUB, t.real, uno, fraccion);
        let calor = f.glsl(t.real, 43, &[calor, cero, uno]);
        let calor2 = f.binaria(op::F_MUL, t.real, calor, calor);
        let escala = Matematica::escalar(f, 0.75);
        let verde = f.binaria(op::F_MUL, t.real, calor2, escala);
        let base = Matematica::escalar(f, 0.12);
        let verde = f.binaria(op::F_ADD, t.real, verde, base);
        let azul = Matematica::escalar(f, 0.015);
        let emision = f.construir(t.v3, &[uno, verde, azul]);
        let emision = self.vista(f, t, mat, id, emision, fraccion);
        let opciones = t.cargar(f, 3);
        let vista = f.extraer(t.real, opciones, 0);
        let belleza = f.binaria(180, mat.booleano, vista, cero);
        let atenuacion = f.elegir(t.real, belleza, calor, uno);
        let peso = f.binaria(op::F_MUL, t.real, peso, atenuacion);
        f.binaria(op::VECTOR_TIMES_SCALAR, t.v3, emision, peso)
    }
    fn vista(&self, f: &mut Funcion, t: &Tipos, mat: &Matematica, id: Id, mut rgb: Id, edad: Id) -> Id {
        let opciones = t.cargar(f, 3);
        let vista = f.extraer(t.real, opciones, 0);
        let uno = Matematica::escalar(f, 1.0);
        let azul = Matematica::escalar(f, 0.15);
        let joven = f.binaria(op::F_SUB, t.real, uno, edad);
        let edad_rgb = f.construir(t.v3, &[edad, joven, azul]);
        let codigo = Matematica::escalar(f, 1.0);
        let coincide = f.binaria(180, mat.booleano, vista, codigo);
        rgb = mat.elegir_vector(f, coincide, edad_rgb, rgb);
        let mut velocidad = Matematica::escalar(f, 0.0);
        for eje in 3..6 {
            let v = self.leer(f, t, id, eje);
            let v = f.glsl(t.real, 4, &[v]);
            velocidad = f.binaria(op::F_ADD, t.real, velocidad, v);
        }
        let mitad = Matematica::escalar(f, 0.5);
        let velocidad = f.binaria(op::F_MUL, t.real, velocidad, mitad);
        let velocidad = f.glsl(t.real, 37, &[velocidad, uno]);
        let cian = f.construir(t.v3, &[azul, velocidad, uno]);
        let codigo = Matematica::escalar(f, 2.0);
        let coincide = f.binaria(180, mat.booleano, vista, codigo);
        rgb = mat.elegir_vector(f, coincide, cian, rgb);
        let entero = f.modulo().tipo_u32();
        let generacion = self.leer(f, t, id, 8);
        let generacion = f.unaria(op::F_A_U, entero, generacion);
        let capacidad = f.modulo().constante_u32(CAPACIDAD);
        let clave = f.binaria(op::I_MUL, entero, generacion, capacidad);
        let mut clave = f.binaria(op::I_ADD, entero, clave, id);
        let canales = core::array::from_fn::<_, 3, _>(|_| {
            let a = f.modulo().constante_u32(1_664_525);
            let b = f.modulo().constante_u32(1_013_904_223);
            let bits = f.modulo().constante_u32(16);
            clave = f.binaria(op::I_MUL, entero, clave, a);
            clave = f.binaria(op::I_ADD, entero, clave, b);
            let alto = f.binaria(194, entero, clave, bits);
            let alto = f.unaria(op::U_A_F, t.real, alto);
            let escala = Matematica::escalar(f, 0.8 / 65535.0);
            let valor = f.binaria(op::F_MUL, t.real, alto, escala);
            let base = Matematica::escalar(f, 0.2);
            f.binaria(op::F_ADD, t.real, base, valor)
        });
        let identidad = f.construir(t.v3, &canales);
        let codigo = Matematica::escalar(f, 3.0);
        let coincide = f.binaria(180, mat.booleano, vista, codigo);
        mat.elegir_vector(f, coincide, identidad, rgb)
    }
}
