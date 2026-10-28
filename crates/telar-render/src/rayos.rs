//! Intersección bilateral rayo–triángulo emitida como SPIR-V, sin lecturas host.
//! Base experimental de P3: Möller–Trumbore, sin BVH ni garantía watertight.
use hilo_spirv::{Funcion, Id, op};

/// Valores SSA de un rayo; origen/dirección son vec3, límites son f32.
/// `t` parametriza O+tD: solo es distancia si D es unitario.
pub struct Rayo {
    /// Origen en el mismo espacio que los vértices.
    pub origen: Id,
    /// Dirección no necesariamente normalizada.
    pub direccion: Id,
    /// Extremo inferior inclusivo, no negativo.
    pub minimo: Id,
    /// Extremo superior inclusivo y finito.
    pub maximo: Id,
}

/// Resultado SSA; `t`, `u` y `v` valen cero cuando no hay impacto.
pub struct Impacto {
    /// Booleano: intersección dentro del segmento y del triángulo.
    pub existe: Id,
    /// Parámetro del rayo, no distancia euclídea en general.
    pub t: Id,
    /// Peso del segundo vértice; el primero pesa 1-u-v.
    pub u: Id,
    /// Peso del tercer vértice.
    pub v: Id,
}

/// Emite una prueba sobre tres vec3. Los Id deben pertenecer a `f` y tener
/// los tipos documentados. No accede a buffers ni sincroniza memoria.
/// Rechaza valores no finitos, intervalo inválido y |det| <= 1e-7.
/// Este umbral absoluto restringe escala/ángulos; no es una solución robusta
/// para todas las escenas ni garantiza unión sin grietas en aristas compartidas.
pub fn intersectar(f: &mut Funcion, rayo: &Rayo, vertices: [Id; 3]) -> Impacto {
    let real = f.modulo().tipo_f32();
    let vector = f.modulo().tipo_vector(real, 3);
    let booleano = f.modulo().tipo_bool();
    let cero = f.modulo().constante_f32(0.0);
    let uno = f.modulo().constante_f32(1.0);
    let epsilon = f.modulo().constante_f32(1e-7);
    let finito = f.modulo().constante_f32(f32::MAX);
    let [primero, segundo, tercero] = vertices;
    let e1 = f.binaria(op::F_SUB, vector, segundo, primero);
    let e2 = f.binaria(op::F_SUB, vector, tercero, primero);
    let cruz_direccion = f.glsl(vector, 68, &[rayo.direccion, e2]); // Cross.
    let det = f.binaria(op::DOT, real, e1, cruz_direccion);
    let absoluto = f.glsl(real, op::GLSL_ABSOLUTO, &[det]);
    let valido = f.binaria(op::F_ORD_GREATER_THAN, booleano, absoluto, epsilon);
    // La división recibe 1 cuando det es degenerado/NaN: no genera 1/0.
    let divisor = f.elegir(real, valido, det, uno);
    let inversa = f.binaria(op::F_DIV, real, uno, divisor);
    let desplazamiento = f.binaria(op::F_SUB, vector, rayo.origen, primero);
    let cruz_origen = f.glsl(vector, 68, &[desplazamiento, e1]);
    let numeradores = [
        f.binaria(op::DOT, real, e2, cruz_origen),
        f.binaria(op::DOT, real, desplazamiento, cruz_direccion),
        f.binaria(op::DOT, real, rayo.direccion, cruz_origen),
    ];
    let [t, u, v] = numeradores.map(|n| f.binaria(op::F_MUL, real, n, inversa));
    let suma = f.binaria(op::F_ADD, real, u, v);
    let mut existe = valido;
    // Comparaciones ordenadas: NaN no puede convertir un fallo en impacto.
    for (inferior, superior) in [
        (cero, rayo.minimo),
        (rayo.minimo, rayo.maximo),
        (rayo.minimo, t),
        (t, rayo.maximo),
        (cero, u),
        (cero, v),
        (suma, uno),
    ] {
        // OpFOrdLessThanEqual=188, inclusivo en aristas y extremos del segmento.
        let dentro = f.binaria(188, booleano, inferior, superior);
        existe = f.binaria(op::LOGICAL_AND, booleano, existe, dentro);
    }
    let mut escalares = alloc::vec![rayo.minimo, rayo.maximo, det, t, u, v];
    for valor in [primero, segundo, tercero, rayo.origen, rayo.direccion] {
        escalares.extend((0..3).map(|i| f.extraer(real, valor, i)));
    }
    for valor in escalares {
        let absoluto = f.glsl(real, op::GLSL_ABSOLUTO, &[valor]);
        let valido = f.binaria(188, booleano, absoluto, finito);
        existe = f.binaria(op::LOGICAL_AND, booleano, existe, valido);
    }
    Impacto {
        existe,
        t: f.elegir(real, existe, t, cero),
        u: f.elegir(real, existe, u, cero),
        v: f.elegir(real, existe, v, cero),
    }
}
