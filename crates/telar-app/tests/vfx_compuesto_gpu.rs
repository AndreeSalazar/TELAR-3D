//! Composición real en HDR: cobertura, extinción, intensidad y pase independiente.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de integración GPU")]
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
const CELDAS: usize = 32768;
#[derive(Clone, Copy)]
struct Escena {
    densidad: f32,
    brillo: f32,
    posicion: [f32; 3],
    viva: bool,
    estela: bool,
    diagnostico: bool,
    radio: f32,
    altura: f32,
    calor_detras: f32,
    ojo: [f32; 3],
}
impl Default for Escena {
    fn default() -> Self {
        Self {
            densidad: 0.0,
            brillo: 1.0,
            posicion: [0.5, 0.5, 0.8],
            viva: true,
            estela: false,
            diagnostico: false,
            radio: 0.0,
            altura: 0.5,
            calor_detras: 0.0,
            ojo: [0.5, 0.5, 2.0],
        }
    }
}
fn preparar() -> (telar_ventana::Ventana, Gpu) {
    let ventana = telar_ventana::Ventana::nueva(&telar_ventana::Config {
        ancho: 64,
        alto: 64,
        visible: false,
        ..Default::default()
    })
    .unwrap();
    let manejadores = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, ..Default::default() },
        Superficie { instancia: manejadores.instancia, ventana: manejadores.ventana },
        ventana.tamano(),
    )
    .unwrap();
    let codigos =
        [telar_render::vfx_final::programa(32, false).unwrap(), telar_render::vfx_final::programa(32, true).unwrap()];
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::volumen::fragmentos_vfx(32, 16).unwrap(),
        },
        PlanCampos { elementos: 32768, palabras: 31, programas: &codigos, secuencia: &[1, 0] },
    )
    .unwrap();
    gpu.activar_campos(true).unwrap();
    (ventana, gpu)
}
fn mostrar(gpu: &mut Gpu, escena: Escena) -> f32 {
    let mut estado = vec![0.0f32; CELDAS * 31];
    let mut ajustes = telar_ui::Vfx::default();
    ajustes.0[15] = 0.0;
    ajustes.0[16] = 1.0;
    ajustes.0[18] = if escena.calor_detras > 0.0 { 3.0 } else { 0.0 };
    ajustes.0[27] = escena.brillo;
    estado[3 * CELDAS..4 * CELDAS].fill(escena.densidad);
    if escena.calor_detras > 0.0 {
        estado[4 * CELDAS..4 * CELDAS + 8 * 32 * 32].fill(escena.calor_detras);
    }
    estado[19 * CELDAS..19 * CELDAS + 28].copy_from_slice(&ajustes.0);
    for eje in 0..3 {
        estado[20 * CELDAS + eje * 128] = escena.posicion[eje];
        for historia in 0..4 {
            let valor = if escena.estela && eje == 0 { 0.46 } else { escena.posicion[eje] };
            estado[20 * CELDAS + 1536 + (historia * 3 + eje) * 128] = valor;
        }
    }
    estado[20 * CELDAS + 10 * 128] = if escena.viva { 1.0 } else { 0.0 };
    gpu.cargar_campos(&estado.iter().map(|v| v.to_bits()).collect::<Vec<_>>()).unwrap();
    let mut bloque = [0.0; 32];
    bloque[6] = 1.0;
    bloque[7] = if escena.diagnostico { 3.0 } else { 0.0 };
    bloque[14] = escena.radio;
    bloque[15] = escena.altura;
    bloque[16..19].copy_from_slice(&[0.0, 0.0, -1.0]);
    bloque[20..23].copy_from_slice(&[0.5, 0.0, 0.0]);
    bloque[24..27].copy_from_slice(&[0.0, 0.5, 0.0]);
    bloque[28..31].copy_from_slice(&escena.ojo);
    gpu.ejecutar_campos(&bloque).unwrap();
    let centro = telar_render::vfx_final::base(32) + (128 * 256 + 128) * 5;
    f32::from_bits(gpu.inspeccionar_rango_campos(centro, 1).unwrap()[0])
}
fn contribucion(gpu: &mut Gpu, escena: Escena) -> f32 {
    mostrar(gpu, escena) - mostrar(gpu, Escena { viva: false, ..escena })
}
#[test]
fn chispas_y_estelas_se_integran_en_belleza_con_extincion() {
    let (ventana, mut gpu) = preparar();
    let base = Escena::default();
    let clara = contribucion(&mut gpu, base);
    assert!(clara > 0.1);
    let delante = contribucion(&mut gpu, Escena { densidad: 1.0, ..base }) / clara;
    let atras = Escena { posicion: [0.5, 0.5, 0.2], ..base };
    let limpia_atras = contribucion(&mut gpu, atras);
    let detras = contribucion(&mut gpu, Escena { densidad: 1.0, ..atras }) / limpia_atras;
    // Aproximación de ocho muestras: margen por la frontera del cubo y jitter del píxel.
    assert!((delante - (-0.2f32).exp()).abs() < 0.12, "delante {delante}");
    assert!((detras - (-0.8f32).exp()).abs() < 0.12, "detrás {detras}");
    assert!(detras < delante);
    assert!(contribucion(&mut gpu, Escena { brillo: 0.0, ..base }).abs() < 1e-6);
    let doble = contribucion(&mut gpu, Escena { brillo: 2.0, ..base });
    assert!((doble / clara - 2.0).abs() < 1e-4);
    assert!(contribucion(&mut gpu, Escena { posicion: [0.5, 0.5, 2.2], ..base }).abs() < 1e-6);
    let estela = Escena { posicion: [0.54, 0.5, 0.5], estela: true, ..base };
    let lineal = contribucion(&mut gpu, estela);
    assert!(lineal > 0.1);
    assert!(contribucion(&mut gpu, Escena { densidad: 2.0, ..estela }) < lineal * 0.8);
    let diagnostico = Escena { diagnostico: true, ..estela };
    assert!(contribucion(&mut gpu, Escena { brillo: 0.0, ..diagnostico }).abs() < 1e-6);
    assert_eq!(mostrar(&mut gpu, diagnostico), mostrar(&mut gpu, Escena { densidad: 2.0, ..diagnostico }));
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn esfera_corta_el_volumen_y_bloquea_secundarios_detras() {
    let (ventana, mut gpu) = preparar();
    let esfera = Escena { radio: 0.2, ..Default::default() };
    let superficie = mostrar(&mut gpu, Escena { viva: false, ..esfera });
    assert!((superficie - 0.18 * 0.04).abs() < 1e-6);
    assert!(contribucion(&mut gpu, esfera) > 0.1);
    for profundidad in [0.2, 0.5] {
        let oculta = Escena { posicion: [0.5, 0.5, profundidad], ..esfera };
        assert!(contribucion(&mut gpu, oculta).abs() < 1e-6);
    }
    let pequena = Escena { radio: 0.001, posicion: [0.5, 0.5, 0.2], ..esfera };
    assert!(contribucion(&mut gpu, pequena).abs() < 1e-6);
    let estela = Escena { posicion: [0.54, 0.5, 0.2], estela: true, ..esfera };
    assert!(contribucion(&mut gpu, estela).abs() < 1e-6);
    assert!(contribucion(&mut gpu, Escena { diagnostico: true, ..estela }) > 0.1);
    let calor = Escena { viva: false, calor_detras: 1.0, ..esfera };
    assert!((mostrar(&mut gpu, calor) - superficie).abs() < 1e-6);
    assert!(mostrar(&mut gpu, Escena { radio: 0.0, ..calor }) > superficie + 0.1);
    let vacio = Escena { viva: false, radio: 0.0, ..esfera };
    assert_eq!(mostrar(&mut gpu, vacio), mostrar(&mut gpu, Escena { radio: 0.2, altura: 0.1, ..vacio }));
    let dentro = mostrar(&mut gpu, Escena { ojo: [0.5; 3], viva: false, ..esfera });
    assert!(dentro.is_finite());
    drop(gpu);
    drop(ventana);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
