//! Vista previa directa en fragmentos: 1 spp, sin descargas de píxeles al host.
use crate::{
    escena_rayos::Trabajo,
    tipos::{Tipos, variable},
    trazado::Matematica,
};
use alloc::vec::Vec;
use hilo_spirv::{
    Modulo,
    op::{self, Clase},
};

/// Vértices de un triángulo que cubre el viewport; posición vec3 en ubicación 0.
/// Conserva el ABI de malla: ubicación 1 es vec3 y su Z=1 produce profundidad 0.5.
#[must_use]
pub fn vertices() -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let real = m.tipo_f32();
    let v2 = m.tipo_vector(real, 2);
    let v3 = m.tipo_vector(real, 3);
    let v4 = m.tipo_vector(real, 4);
    let posicion = variable(&mut m, v3, Clase::Entrada, 0);
    let normal = variable(&mut m, v3, Clase::Entrada, 1);
    let uv = variable(&mut m, v2, Clase::Salida, 0);
    let puntero = m.tipo_puntero(Clase::Salida, v4);
    let clip = m.variable(puntero, Clase::Salida);
    m.decorar(clip, op::DECORACION_INTEGRADA, &[op::INTEGRADA_POSICION]);
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let entrada = f.etiqueta();
    f.bloque(entrada);
    let punto = f.cargar(v3, posicion);
    let x = f.extraer(real, punto, 0);
    let y = f.extraer(real, punto, 1);
    let medio = f.modulo().constante_f32(0.5);
    let uno = f.modulo().constante_f32(1.0);
    let datos = f.cargar(v3, normal);
    let profundidad = f.extraer(real, datos, 2);
    let profundidad = f.binaria(op::F_MUL, real, profundidad, medio);
    let salida = f.construir(v4, &[x, y, profundidad, uno]);
    f.guardar(clip, salida);
    let coords = [x, y].map(|eje| {
        let eje = f.binaria(op::F_ADD, real, eje, uno);
        f.binaria(op::F_MUL, real, eje, medio)
    });
    let coords = f.construir(v2, &coords);
    f.guardar(uv, coords);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_VERTICES, main, "main", &[posicion, normal, uv, clip]);
    m.ensamblar()
}

/// Emite un visor de iluminación directa, usando la misma escena que el HDR.
/// `codificar_srgb` debe ser falso si el adjunto ya aplica la transferencia sRGB.
/// La app limita el viewport físico y recompila solamente al confirmar cambios.
/// # Errors
/// Rechaza escenas fuera de contrato antes de crear recursos gráficos.
#[allow(clippy::cast_precision_loss, reason = "dimensiones <=512")]
pub fn fragmentos(trabajo: &Trabajo, codificar_srgb: bool) -> Result<Vec<u32>, &'static str> {
    trabajo.validar()?;
    let mut m = Modulo::nuevo();
    let mat = Matematica::nueva(&mut m);
    let tipos = Tipos::nuevos(&mut m);
    let v2 = m.tipo_vector(mat.real, 2);
    let entrada = variable(&mut m, v2, Clase::Entrada, 0);
    let salida = variable(&mut m, tipos.v4, Clase::Salida, 0);
    let coordenada_tipo = m.tipo_puntero(Clase::Entrada, tipos.v4);
    let coordenada = m.variable(coordenada_tipo, Clase::Entrada);
    m.decorar(coordenada, op::DECORACION_INTEGRADA, &[15]); // FragCoord.
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let inicio = f.etiqueta();
    f.bloque(inicio);
    let uv = f.cargar(v2, entrada);
    let uno = Matematica::escalar(&mut f, 1.0);
    let dos = Matematica::escalar(&mut f, 2.0);
    let mut direccion = mat.vector(&mut f, trabajo.camara.frente);
    for (i, base) in [trabajo.camara.derecha, trabajo.camara.arriba].into_iter().enumerate() {
        let coord = f.extraer(mat.real, uv, u32::try_from(i).unwrap_or(0));
        let coord = f.binaria(op::F_MUL, mat.real, coord, dos);
        let coord = if i == 0 {
            f.binaria(op::F_SUB, mat.real, coord, uno)
        } else {
            f.binaria(op::F_SUB, mat.real, uno, coord)
        };
        let base = mat.vector(&mut f, base);
        let avance = f.binaria(op::VECTOR_TIMES_SCALAR, mat.vector, base, coord);
        direccion = f.binaria(op::F_ADD, mat.vector, direccion, avance);
    }
    let direccion = mat.normalizar(&mut f, direccion);
    let fisicas = f.cargar(tipos.v4, coordenada);
    let coordenadas: [_; 2] = core::array::from_fn(|i| {
        let coord = f.extraer(mat.real, fisicas, u32::try_from(i).unwrap_or(0));
        f.unaria(op::F_A_U, mat.entero, coord)
    });
    let ancho = f.modulo().constante_u32(4096);
    let fila = f.binaria(op::I_MUL, mat.entero, coordenadas[1], ancho);
    let indice = f.binaria(op::I_ADD, mat.entero, fila, coordenadas[0]);
    let semilla = f.modulo().constante_u32(trabajo.semilla);
    let semilla = f.binaria(op::I_ADD, mat.entero, semilla, indice);
    let radiancia = crate::muestra_rayos::iluminar(&mut f, &mat, trabajo, direccion, semilla);
    let exposicion = Matematica::escalar(&mut f, trabajo.exposicion);
    let codificar = Matematica::escalar(&mut f, if codificar_srgb { 1.0 } else { 0.0 });
    let color = crate::superficie::salida(&mut f, &tipos, radiancia, exposicion, codificar);
    f.guardar(salida, color);
    f.volver();
    let main = f.terminar();
    m.entrada(op::MODELO_FRAGMENTOS, main, "main", &[entrada, coordenada, salida]);
    m.modo(main, op::MODO_ORIGEN_ARRIBA, &[]);
    Ok(m.ensamblar())
}
