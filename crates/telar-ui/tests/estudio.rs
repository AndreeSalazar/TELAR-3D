//! Interacciones y distribución sin sistema operativo ni GPU.
#![allow(clippy::float_cmp, reason = "valores exactos de límites y controles")]
use telar_ui::{Distribucion, Estudio};

#[test]
fn paneles_consumen_eventos_y_los_deslizadores_limitan_sus_valores() {
    let diseno = Distribucion::nueva((1280, 720));
    let mut estudio = Estudio::default();
    assert!(!estudio.pulsar(diseno, 600.0, 300.0));
    assert!(estudio.pulsar(diseno, 50.0, 340.0));
    assert_eq!(estudio.seleccionado, 2);
    assert!(estudio.pulsar(diseno, 1100.0, 248.0));
    assert!(estudio.mover(diseno, 5000.0));
    assert_eq!(estudio.amplitud, 0.8);
    estudio.mover(diseno, -100.0);
    assert_eq!(estudio.amplitud, 0.0);
    estudio.soltar();
    assert!(!estudio.mover(diseno, 1200.0));
    assert!(estudio.pulsar(diseno, 1100.0, 324.0));
    estudio.mover(diseno, 5000.0);
    assert_eq!(estudio.velocidad, 2.0);
    estudio.soltar();
    estudio.pulsar(diseno, 1100.0, 390.0);
    assert!(estudio.pausado);
    estudio.pulsar(diseno, 1100.0, 390.0);
    assert!(!estudio.pausado);
    let cobre = estudio.color();
    estudio.pulsar(diseno, 1200.0, 480.0);
    assert_eq!(estudio.material, 2);
    assert_ne!(cobre, estudio.color());
}

#[test]
fn composicion_finitas_y_visor_adaptable() {
    for tamano in [(1280, 720), (800, 600), (720, 580), (300, 200), (0, 0)] {
        let diseno = Distribucion::nueva(tamano);
        let [x, y, ancho, alto] = diseno.area();
        assert!(ancho > 0 && alto > 0);
        assert!(x + ancho <= tamano.0.max(1) && y + alto <= tamano.1.max(1));
        let estudio = Estudio::default();
        let vertices = estudio.dibujar(diseno, 1_000_000);
        assert!(vertices.len() <= 120_000 && vertices.len().is_multiple_of(3));
        assert!(vertices.iter().flatten().all(|v| v.is_finite()));
        if tamano.0 >= 720 && tamano.1 >= 580 {
            assert!(vertices.len() > 1000);
        } else {
            assert_eq!(vertices, [] as [[f32; 6]; 0]);
        }
    }
    let diseno = Distribucion::nueva((300, 200));
    let mut estudio = Estudio::default();
    assert!(!estudio.pulsar(diseno, 50.0, 50.0));
}
