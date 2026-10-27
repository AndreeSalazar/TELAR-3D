//! El visor de punta a punta, sin ventana: geometría de NUDO → malla → shaders
//! de hilo-spirv → pipeline de la GPU → una captura que vuelve a la CPU, y se
//! miran sus píxeles. Comprueba lo que un test de la CPU no puede: que arriba es
//! arriba, que se descartan las caras de detrás (y no las de delante) y que la
//! profundidad tapa lo de atrás. Con las capas de validación: cero avisos.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use nudo::mate::normalizar;
use nudo::{Camara, Malla, Nodo};
use telar_gpu::{Calculo, Constantes, Opciones, Sombreadores, avisos_de_validacion};
use telar_nodos::Esfera;

const LADO: u32 = 64;
const NEGRO: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BLANCO: [f32; 4] = [1.0; 4];

/// La GPU sin ventana, o `None` si esta máquina no tiene Vulkan (no es un fallo de TELAR).
fn gpu() -> Option<Calculo> {
    match Calculo::nuevo(&Opciones::default()) {
        Ok(c) => Some(c),
        Err(e) if e.contains("vulkan-1.dll") => {
            eprintln!("sin Vulkan en esta máquina: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

/// Esferas de NUDO `(radio, centro)` en una sola malla, en ese orden.
fn esferas(cuales: &[(f32, [f32; 3])]) -> Malla {
    let mut malla = Malla { vertices: Vec::new(), indices: Vec::new() };
    for &(radio, centro) in cuales {
        let m = Malla::de(&Esfera { radio, filas: 48, columnas: 64, centro }.cocinar(&[]).unwrap());
        let base = u32::try_from(malla.vertices.len()).unwrap();
        malla.indices.extend(m.indices.iter().map(|i| i + base));
        malla.vertices.extend(m.vertices);
    }
    malla
}

/// Captura `malla` vista desde `camara` y devuelve el brillo (canal rojo, 0–255) de cada píxel.
fn capturar(gpu: &Calculo, malla: &Malla, camara: &Camara, luz: [f32; 3], cielo: [f32; 4], suelo: [f32; 4]) -> Vec<u8> {
    let (vertices, fragmentos) = (hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos());
    let sombreadores = Sombreadores { vertices: &vertices, fragmentos: &fragmentos };
    let constantes = Constantes { matriz: camara.matriz(1.0), luz, color: BLANCO, cielo, suelo };
    let pixeles =
        gpu.capturar(sombreadores, &malla.vertices, &malla.indices, (LADO, LADO), NEGRO, &constantes).unwrap();
    assert_eq!(pixeles.len(), (LADO * LADO * 4) as usize);
    assert_eq!(avisos_de_validacion(), 0, "las capas de validación avisaron");
    pixeles.as_chunks::<4>().0.iter().map(|p| p[0]).collect()
}

fn en(brillo: &[u8], x: u32, y: u32) -> u8 {
    brillo[(y * LADO + x) as usize]
}

/// Una cámara a la altura del centro, sin inclinar: la vertical de la imagen es la del mundo.
fn de_frente() -> Camara {
    Camara { inclinacion: 0.0, giro: 0.0, ..Camara::default() }
}

#[test]
fn se_ven_las_caras_de_delante_iluminadas_y_el_fondo_alrededor() {
    let Some(gpu) = gpu() else { return };
    let camara = de_frente();
    let malla = esferas(&[(1.0, [0.0; 3])]);
    let hacia_la_camara = normalizar(camara.posicion());
    let desde_atras = hacia_la_camara.map(|c| -c);
    // Sin luz ambiente: solo la difusa.
    let de_cara = capturar(&gpu, &malla, &camara, hacia_la_camara, NEGRO, NEGRO);
    let de_espaldas = capturar(&gpu, &malla, &camara, desde_atras, NEGRO, NEGRO);
    let centro = LADO / 2;
    // Si se descartaran las de delante, se verían las de dentro de la otra mitad, de
    // espaldas a la cámara: el centro saldría oscuro con la luz de cara.
    assert!(en(&de_cara, centro, centro) > 200, "centro con la luz de cara: {}", en(&de_cara, centro, centro));
    assert!(en(&de_espaldas, centro, centro) < 10, "centro con la luz detrás: {}", en(&de_espaldas, centro, centro));
    assert_eq!(en(&de_cara, 0, 0), 0, "la esquina es fondo");
    assert_eq!(en(&de_cara, LADO - 1, LADO - 1), 0, "la esquina es fondo");
}

#[test]
fn arriba_es_arriba() {
    let Some(gpu) = gpu() else { return };
    let camara = de_frente();
    let malla = esferas(&[(0.5, [0.0, 0.8, 0.0])]);
    let brillo = capturar(&gpu, &malla, &camara, normalizar(camara.posicion()), NEGRO, NEGRO);
    let encendidos = |filas: core::ops::Range<u32>| {
        filas.flat_map(|y| (0..LADO).map(move |x| (x, y))).filter(|&(x, y)| en(&brillo, x, y) > 0).count()
    };
    let (arriba, abajo) = (encendidos(0..LADO / 2), encendidos(LADO / 2..LADO));
    assert!(arriba > 100 && abajo == 0, "la esfera está arriba en el mundo: {arriba} píxeles arriba, {abajo} abajo");
}

#[test]
fn lo_de_delante_tapa_lo_de_atras_aunque_se_dibuje_antes() {
    let Some(gpu) = gpu() else { return };
    let camara = de_frente();
    let hacia_la_camara = normalizar(camara.posicion());
    // Una esferita entre la cámara y la grande, algo por encima del eje: el rayo del
    // centro le da en la parte de abajo (normal hacia abajo). Se dibuja primero.
    let cerca = hacia_la_camara.map(|c| c * 2.0);
    let pequena = (0.3, [cerca[0], cerca[1] + 0.25, cerca[2]]);
    let malla = esferas(&[pequena, (1.0, [0.0; 3])]);
    // Solo luz ambiente, blanca desde arriba y negra desde abajo: el brillo dice hacia
    // dónde mira la normal (0 abajo, 0,5 de lado, 1 arriba).
    let brillo = capturar(&gpu, &malla, &camara, [0.0; 3], BLANCO, NEGRO);
    let centro = en(&brillo, LADO / 2, LADO / 2);
    // La grande, de frente, daría ~127; la pequeña, por debajo, ~20.
    assert!(centro < 70, "en el centro se ve la esfera de atrás (brillo {centro}): la profundidad no tapa");
    // Al lado, fuera de la pequeña, sí se ve la grande, de lado (la grande ocupa ±15 píxeles).
    let lado = en(&brillo, LADO / 2 + 12, LADO / 2);
    assert!(lado > 100, "al lado del centro tendría que verse la grande (brillo {lado})");
}
