//! El inspector real compone píxeles y cambia parámetros de la geometría GPU.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "asserts y fallos de verificación en tests")]

use nudo::Camara;
use telar_gpu::{
    Calculo, CapaInterfaz, Constantes, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores,
    avisos_de_validacion,
};
use telar_ui::{Distribucion, Estudio};

#[test]
fn estudio_capturado_dibuja_paneles_y_reacciona_al_inspector() {
    let gpu = Calculo::nuevo(&Opciones::default()).expect("esta prueba requiere Vulkan real");
    let codigo = [hilo_spirv::rejilla::preparar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let mut residente = gpu
        .procedural(64, KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] })
        .unwrap();
    let escena = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let shaders = Sombreadores { vertices: &escena[0], fragmentos: &escena[1] };
    let diseno = Distribucion::nueva((1280, 720));
    let mut estudio = Estudio::default();
    let constantes = Constantes {
        matriz: Camara { distancia: 7.0, ..Camara::default() }.matriz(diseno.visor.ancho / diseno.visor.alto),
        luz: [0.0, 1.0, 0.0],
        color: estudio.color(),
        cielo: [0.3; 4],
        suelo: [0.1; 4],
    };
    residente.paso(ParametrosProcedurales { lado: 64, fase: 0.6, amplitud: estudio.amplitud }).unwrap();
    let triangulos = estudio.dibujar(diseno, 4096);
    let captura = |vertices| CapaInterfaz {
        shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
        vertices,
        area: diseno.area(),
    };
    let fondo = [0.04, 0.06, 0.09, 1.0];
    let primera = residente.capturar_estudio(shaders, (1280, 720), fondo, &constantes, captura(&triangulos)).unwrap();
    let pixel = |datos: &[u8], x: usize, y: usize| datos[(y * 1280 + x) * 4..(y * 1280 + x) * 4 + 4].to_vec();
    assert_eq!(pixel(&primera, 4, 20), vec![16, 20, 25, 255], "fondo del encabezado");
    assert!(pixel(&primera, 22, 22)[1] > 180, "acento cian del logo");
    estudio.pulsar(diseno, 1230.0, 248.0);
    estudio.mover(diseno, 5000.0);
    estudio.soltar();
    residente.paso(ParametrosProcedurales { lado: 64, fase: 0.6, amplitud: estudio.amplitud }).unwrap();
    let triangulos = estudio.dibujar(diseno, 4096);
    let segunda = residente.capturar_estudio(shaders, (1280, 720), fondo, &constantes, captura(&triangulos)).unwrap();
    let diferencias = (100..570)
        .flat_map(|y| (230..1000).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&primera, x, y) != pixel(&segunda, x, y))
        .count();
    assert!(diferencias > 1000, "el deslizador cambia la superficie, no solo el rótulo");
    assert_eq!(residente.actividad().geometria_subida, 0);
    drop(residente);
    drop(gpu);
    assert_eq!(avisos_de_validacion(), 0);
}
