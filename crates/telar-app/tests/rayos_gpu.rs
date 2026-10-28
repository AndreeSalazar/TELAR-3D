//! P3 experimental: impactos de varios triángulos, contra planos/baricéntricas f64.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos en pruebas GPU")]
use hilo_spirv::{
    Funcion, Id, Modulo,
    op::{self, Clase},
};
use telar_render::rayos::{Rayo, intersectar};

const TRIANGULOS: [[[f32; 3]; 3]; 3] = [
    [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
    [[-1.0, -1.0, -2.0], [0.0, 1.0, -2.0], [1.0, -1.0, -2.0]],
    [[2.0, -1.0, 0.0], [4.0, -1.0, 0.0], [3.0, 1.0, 0.0]],
];

// Origen xyz, dirección xyz, tmin, tmax. Incluye ambos lados, extremos y rechazos.
const CASOS: [[f32; 8]; 19] = [
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 2.1, 10.0],
    [3.0, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, -4.0, 0.0, 0.0, 1.0, 0.0, 10.0],
    [0.0, 1.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [-1.0, -1.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, -1.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -2.0, 1.0, 1.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, 1.9],
    [0.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, 1.0, 0.0, 10.0],
    [8.0, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 5.0, 1.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, -1.0, 10.0],
    [f32::NAN, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 0.0, f32::INFINITY],
    [0.0, 0.0, 2.0, 0.25, 0.0, -1.0, 0.0, 10.0],
    [0.0, 0.0, 2.0, 0.0, 0.0, -1.0, 2.0, 2.0],
];

fn vector(f: &mut Funcion, tipo: Id, valores: [f32; 3]) -> Id {
    let ids = valores.map(|v| f.modulo().constante_f32(v));
    f.construir(tipo, &ids)
}

fn codigo(triangulos: &[[[f32; 3]; 3]]) -> Vec<u32> {
    let mut m = Modulo::nuevo();
    let entero = m.tipo_u32();
    let real = m.tipo_f32();
    let v3 = m.tipo_vector(real, 3);
    let booleano = m.tipo_bool();
    let global_tipo = m.tipo_vector(entero, 3);
    let entrada = m.tipo_puntero(Clase::Entrada, global_tipo);
    let global = m.variable(entrada, Clase::Entrada);
    m.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let array = m.tipo_array_dinamico(entero, 4);
    let bloque = m.tipo_bloque(&[array], &[0]);
    let tipo = m.tipo_puntero(Clase::Almacen, bloque);
    let datos = m.variable(tipo, Clase::Almacen);
    m.decorar(datos, op::DECORACION_CONJUNTO, &[0]);
    m.decorar(datos, op::DECORACION_ENLACE, &[0]);
    let puntero = m.tipo_puntero(Clase::Almacen, entero);
    let [cero, cuatro, ausente] = [0, 4, u32::MAX].map(|v| m.constante_u32(v));
    let void = m.tipo_void();
    let firma = m.tipo_funcion(void, &[]);
    let mut f = m.funcion(void, firma);
    let [entrada, cuerpo, fin] = core::array::from_fn(|_| f.etiqueta());
    f.bloque(entrada);
    let invocacion = f.cargar(global_tipo, global);
    let i = f.extraer(entero, invocacion, 0);
    let longitud = f.longitud(entero, datos, 0);
    let cantidad = f.binaria(op::U_DIV, entero, longitud, cuatro);
    let limite = f.modulo().constante_u32(u32::try_from(CASOS.len()).unwrap());
    let capacidad = f.binaria(op::U_LESS_THAN, booleano, i, cantidad);
    let caso = f.binaria(op::U_LESS_THAN, booleano, i, limite);
    let dentro = f.binaria(op::LOGICAL_AND, booleano, capacidad, caso);
    f.fusion(fin);
    f.saltar_si(dentro, cuerpo, fin);
    f.bloque(cuerpo);
    let campos: [Id; 8] = core::array::from_fn(|campo| {
        let mut valor = f.modulo().constante_f32(CASOS[0][campo]);
        for (indice, caso) in CASOS.iter().enumerate().skip(1) {
            let n = f.modulo().constante_u32(u32::try_from(indice).unwrap());
            let coincide = f.binaria(op::I_EQUAL, booleano, i, n);
            let nuevo = f.modulo().constante_f32(caso[campo]);
            valor = f.elegir(real, coincide, nuevo, valor);
        }
        valor
    });
    let rayo = Rayo {
        origen: f.construir(v3, &campos[..3]),
        direccion: f.construir(v3, &campos[3..6]),
        minimo: campos[6],
        maximo: campos[7],
    };
    let mut objeto = ausente;
    let mut mejor = campos[7];
    let cero_real = f.modulo().constante_f32(0.0);
    let mut resultado = [cero_real; 3];
    for (indice, vertices) in triangulos.iter().enumerate() {
        let vertices = vertices.map(|v| vector(&mut f, v3, v));
        let impacto = intersectar(&mut f, &rayo, vertices);
        let antes = f.binaria(op::F_ORD_GREATER_THAN, booleano, mejor, impacto.t);
        let primero = f.binaria(op::I_EQUAL, booleano, objeto, ausente);
        let elegir = f.binaria(op::LOGICAL_OR, booleano, primero, antes);
        let elegir = f.binaria(op::LOGICAL_AND, booleano, impacto.existe, elegir);
        mejor = f.elegir(real, elegir, impacto.t, mejor);
        let id = f.modulo().constante_u32(u32::try_from(indice).unwrap());
        objeto = f.elegir(entero, elegir, id, objeto);
        for (destino, nuevo) in resultado.iter_mut().zip([impacto.t, impacto.u, impacto.v]) {
            *destino = f.elegir(real, elegir, nuevo, *destino);
        }
    }
    let base = f.binaria(op::I_MUL, entero, i, cuatro);
    let valores = [objeto, resultado[0], resultado[1], resultado[2]];
    for (indice, valor) in valores.into_iter().enumerate() {
        let desplazamiento = f.modulo().constante_u32(u32::try_from(indice).unwrap());
        let direccion = f.binaria(op::I_ADD, entero, base, desplazamiento);
        let destino = f.acceso(puntero, datos, &[cero, direccion]);
        let valor = if indice == 0 { valor } else { f.unaria(op::BITCAST, entero, valor) };
        // GPU: cada invocación posee cuatro palabras; floor(longitud/4) limita accesos.
        f.guardar(destino, valor);
    }
    f.saltar(fin);
    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    m.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    m.ensamblar()
}

fn restar(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    core::array::from_fn(|i| a[i] - b[i])
}
fn punto(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}
fn cruz(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

// Referencia independiente: intersección con plano y coordenadas por matriz de Gram.
#[allow(clippy::many_single_char_names, reason = "notación de plano y matriz de Gram en referencia matemática")]
fn referencia(caso: [f32; 8], triangulos: &[[[f32; 3]; 3]]) -> (u32, [f64; 3]) {
    let mut salida = (u32::MAX, [0.0; 3]);
    if !caso.iter().all(|v| v.is_finite()) || caso[6] < 0.0 || caso[6] > caso[7] {
        return salida;
    }
    let o = [caso[0], caso[1], caso[2]].map(f64::from);
    let d = [caso[3], caso[4], caso[5]].map(f64::from);
    for (indice, vertices) in triangulos.iter().enumerate() {
        let [a, b, c] = vertices.map(|v| v.map(f64::from));
        let e1 = restar(b, a);
        let e2 = restar(c, a);
        let normal = cruz(e1, e2);
        let divisor = punto(normal, d);
        if divisor.abs() <= 1e-7 {
            continue;
        }
        let t = punto(normal, restar(a, o)) / divisor;
        let p = core::array::from_fn(|i| o[i] + t * d[i] - a[i]);
        let aa = punto(e1, e1);
        let bb = punto(e1, e2);
        let cc = punto(e2, e2);
        let denominador = aa * cc - bb * bb;
        let u = (punto(p, e1) * cc - punto(p, e2) * bb) / denominador;
        let v = (punto(p, e2) * aa - punto(p, e1) * bb) / denominador;
        if t >= f64::from(caso[6])
            && t <= f64::from(caso[7])
            && u >= 0.0
            && v >= 0.0
            && u + v <= 1.0
            && (salida.0 == u32::MAX || t < salida.1[0])
        {
            salida = (u32::try_from(indice).unwrap(), [t, u, v]);
        }
    }
    salida
}

#[test]
fn impactos_gpu_eligen_objeto_cercano_y_respetan_segmentos_y_guardas() {
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones::default()).expect("Vulkan real");
    assert!(gpu.validacion());
    // También prueba degenerados y empate determinista: gana el primer triángulo.
    let degenerado = [[0.0; 3]; 3];
    let inclinado = [[-1.0, -1.0, -0.5], [1.0, -1.0, 0.5], [0.0, 1.0, 0.0]];
    for triangulos in [TRIANGULOS.to_vec(), vec![degenerado], vec![TRIANGULOS[0]; 2], vec![inclinado]] {
        let shader = codigo(&triangulos);
        validar(&shader);
        let palabras = u32::try_from(CASOS.len() * 4 + 3).unwrap();
        let resultado = gpu.ejecutar(&shader, palabras, 64).unwrap();
        for (indice, caso) in CASOS.into_iter().enumerate() {
            let actual = &resultado.datos[indice * 4..indice * 4 + 4];
            let esperado = referencia(caso, &triangulos);
            assert_eq!(actual[0], esperado.0, "caso {indice}");
            for (actual, esperado) in actual[1..].iter().zip(esperado.1) {
                let actual = f64::from(f32::from_bits(*actual));
                assert!((actual - esperado).abs() < 2e-5, "caso {indice}: {actual} != {esperado}");
            }
        }
        assert_eq!(&resultado.datos[CASOS.len() * 4..], &[0, 0, 0], "cola incompleta intacta");
    }
    let shader = codigo(&TRIANGULOS);
    for palabras in 1..4 {
        let resultado = gpu.ejecutar(&shader, palabras, 64).unwrap();
        assert!(resultado.datos.iter().all(|v| *v == 0), "sin espacio para un registro, ninguna escritura");
    }
    eprintln!("IGUAL: 19 rayos × 4 escenas · {} · tolerancia 2e-5", gpu.dispositivo());
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

fn validar(codigo: &[u32]) {
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|p| std::path::PathBuf::from(p).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let ruta = std::env::temp_dir().join(format!("telar-rayos-{}.spv", std::process::id()));
    std::fs::write(&ruta, codigo.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let salida = std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(&ruta).output().unwrap();
    std::fs::remove_file(ruta).unwrap();
    assert!(salida.status.success(), "{}", String::from_utf8_lossy(&salida.stderr));
}
