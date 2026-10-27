//! Interacciones y distribución sin sistema operativo ni GPU.
#![allow(clippy::float_cmp, reason = "valores exactos de límites y controles")]
use telar_ui::{Distribucion, Estudio};

#[test]
fn paneles_consumen_eventos_y_los_deslizadores_limitan_sus_valores() {
    let diseno = Distribucion::nueva((1280, 720));
    let mut estudio = Estudio::default();
    assert!(!estudio.pulsar(diseno, 600.0, 300.0));
    let nodo = estudio.nodo(diseno, 2);
    assert!(estudio.pulsar(diseno, nodo.x + 10.0, nodo.y + 10.0));
    assert_eq!(estudio.seleccionado, 2);
    let control = diseno.control(0);
    assert!(estudio.pulsar(diseno, control.x + 10.0, control.y + 12.0));
    assert!(estudio.mover(diseno, 5000.0));
    assert_eq!(estudio.relieve.factor, 3.0);
    assert_eq!(estudio.onda.amplitud, 0.35);
    estudio.mover(diseno, -100.0);
    assert_eq!(estudio.relieve.factor, 0.0);
    estudio.soltar();
    assert!(!estudio.mover(diseno, 1200.0));
    estudio.seleccionado = 1;
    let control = diseno.control(1);
    assert!(estudio.pulsar(diseno, control.x + 10.0, control.y + 12.0));
    estudio.mover(diseno, 5000.0);
    assert_eq!(estudio.onda.velocidad, 2.0);
    estudio.soltar();
    let pausa = diseno.pausa();
    estudio.pulsar(diseno, pausa.x + 10.0, pausa.y + 10.0);
    assert!(estudio.pausado);
    estudio.pulsar(diseno, pausa.x + 10.0, pausa.y + 10.0);
    assert!(!estudio.pausado);
    let cobre = estudio.color();
    estudio.seleccionado = 3;
    let muestra = diseno.muestra(2);
    estudio.pulsar(diseno, muestra.x + 10.0, muestra.y + 10.0);
    assert_eq!(estudio.salida.material, 2);
    assert_ne!(cobre, estudio.color());
}

#[test]
fn composicion_finitas_y_visor_adaptable() {
    for tamano in [(1280, 720), (800, 600), (720, 600), (720, 580), (300, 200), (0, 0)] {
        let diseno = Distribucion::nueva(tamano);
        let [x, y, ancho, alto] = diseno.area();
        assert!(ancho > 0 && alto > 0);
        assert!(x + ancho <= tamano.0.max(1) && y + alto <= tamano.1.max(1));
        let estudio = Estudio::default();
        let vertices = estudio.dibujar(diseno, 1_000_000);
        assert!(vertices.len() <= 120_000 && vertices.len().is_multiple_of(3));
        assert!(vertices.iter().flatten().all(|v| v.is_finite()));
        if tamano.0 >= 720 && tamano.1 >= 600 {
            assert!(vertices.len() > 1000);
        } else {
            assert_eq!(vertices, [] as [[f32; 8]; 0]);
        }
    }
    let diseno = Distribucion::nueva((300, 200));
    let mut estudio = Estudio::default();
    assert!(!estudio.pulsar(diseno, 50.0, 50.0));
}

#[test]
fn mover_nudos_conserva_parametros_y_limita_el_lienzo() {
    let diseno = Distribucion::nueva((1280, 720));
    let mut estudio = Estudio::default();
    let inicial = estudio.nodo(diseno, 1);
    let antes = estudio.dibujar(diseno, 4096);
    estudio.pulsar(diseno, inicial.x + 8.0, inicial.y + 8.0);
    assert!(estudio.mover_en(diseno, diseno.grafo.x + 20.0, diseno.grafo.y + 120.0));
    let movido = estudio.nodo(diseno, 1);
    assert_ne!(movido.x, inicial.x);
    assert_ne!(antes, estudio.dibujar(diseno, 4096), "nodo y cables se recomponen");
    assert_eq!((estudio.onda.amplitud, estudio.onda.velocidad), (0.35, 1.0));
    estudio.mover_en(diseno, -5000.0, 5000.0);
    let limitado = estudio.nodo(diseno, 1);
    assert!(limitado.x >= diseno.grafo.x && limitado.y + limitado.alto <= diseno.grafo.y + diseno.grafo.alto + 0.001);
    estudio.soltar();
    assert!(!estudio.mover_en(diseno, 0.0, 0.0));
    let estrecho = Distribucion::nueva((800, 600));
    let adaptado = estudio.nodo(estrecho, 1);
    assert!(adaptado.x >= estrecho.grafo.x && adaptado.x + adaptado.ancho <= estrecho.grafo.x + estrecho.grafo.ancho);
}
#[test]
fn realce_solo_recompone_al_cambiar_objetivo_y_no_modifica_parametros() {
    let mut estudio = telar_ui::Estudio::default();
    let diseno = telar_ui::Distribucion::nueva((1280, 720));
    let antes = estudio.dibujar(diseno, 4096);
    let nodo = estudio.nodo(diseno, 0);
    assert!(estudio.apuntar(diseno, nodo.x + 8.0, nodo.y + 8.0));
    assert!(!estudio.apuntar(diseno, nodo.x + 10.0, nodo.y + 10.0));
    assert_ne!(antes, estudio.dibujar(diseno, 4096));
    assert_eq!(estudio.seleccionado, 1);
    assert_eq!(estudio.onda.amplitud, 0.35);
    assert!(estudio.apuntar(diseno, 0.0, 0.0));
    assert_eq!(antes, estudio.dibujar(diseno, 4096));
}
