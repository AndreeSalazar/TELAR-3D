//! Cada inspector modifica su propio estado y conserva los demás nodos.
#![allow(clippy::float_cmp, reason = "límites y estados exactos de controles")]
use telar_ui::Estudio;

fn arrastrar(estudio: &mut Estudio, indice: usize, fraccion: f32) {
    let diseno = estudio.distribucion((1280, 720));
    let rect = diseno.control(indice);
    estudio.pulsar(diseno, rect.x + rect.ancho * 0.5, rect.y + rect.alto * 0.5);
    estudio.mover(diseno, rect.x + rect.ancho * fraccion);
    estudio.soltar();
}

#[test]
fn configuraciones_por_nodo_no_se_pisan_y_controles_ocultos_no_actuan() {
    let mut estudio = Estudio::default();
    estudio.configurar_capacidad(64);
    let diseno = estudio.distribucion((1280, 720));
    estudio.seleccionado = 0;
    assert_eq!(estudio.control(0).map(|c| c.nombre), Some("Puntos por lado"));
    assert!(estudio.control(1).is_none());
    arrastrar(&mut estudio, 0, 0.0);
    assert_eq!(estudio.rejilla.lado, 2);
    arrastrar(&mut estudio, 0, 1.0);
    assert_eq!(estudio.rejilla.lado, 64);
    arrastrar(&mut estudio, 1, 1.0);
    assert_eq!(estudio.onda.velocidad, 1.0, "un control invisible no debe actuar");
    estudio.seleccionado = 1;
    arrastrar(&mut estudio, 0, 1.0);
    arrastrar(&mut estudio, 1, 0.0);
    assert_eq!((estudio.onda.amplitud, estudio.onda.velocidad), (0.8, 0.0));
    estudio.seleccionado = 2;
    arrastrar(&mut estudio, 0, 1.0);
    assert_eq!(estudio.relieve.factor, 3.0);
    assert_eq!((estudio.onda.amplitud, estudio.onda.velocidad), (0.8, 0.0));
    estudio.seleccionado = 3;
    arrastrar(&mut estudio, 0, 1.0);
    arrastrar(&mut estudio, 1, 0.0);
    assert_eq!((estudio.salida.exposicion, estudio.salida.rugosidad), (4.0, 0.08));
    let muestra = diseno.muestra(1);
    estudio.pulsar(diseno, muestra.x + 10.0, muestra.y + 10.0);
    assert_eq!(estudio.salida.material, 1);
    assert_eq!(estudio.salida.rugosidad, 0.42, "elegir un perfil carga su rugosidad");
    estudio.seleccionado = 2;
    estudio.pulsar(diseno, muestra.x + 10.0, muestra.y + 10.0);
    assert_eq!(estudio.salida.material, 1, "muestras ocultas no cambian el material");
    assert_eq!(estudio.relieve.factor, 3.0);
    assert_eq!(estudio.rejilla.lado, 64);
}

#[test]
fn cambiar_seleccion_durante_arrastre_no_transfiere_el_control() {
    let mut estudio = Estudio::default();
    let diseno = estudio.distribucion((1280, 720));
    let rect = diseno.control(0);
    estudio.pulsar(diseno, rect.x, rect.y + rect.alto * 0.5);
    estudio.seleccionado = 2;
    assert!(!estudio.mover(diseno, rect.x + rect.ancho));
    assert_eq!(estudio.relieve.factor, 1.3);
}

#[test]
fn calidad_es_propia_de_salida_y_no_solapa_los_deslizadores() {
    let mut estudio = Estudio::default();
    for tamano in [(800, 600), (1280, 720), (1920, 1080)] {
        let diseno = estudio.distribucion(tamano);
        assert!(diseno.control(1).y + diseno.control(1).alto < diseno.calidad(0).y);
        assert!(diseno.calidad(0).y + diseno.calidad(0).alto < diseno.pausa().y);
        estudio.seleccionado = 3;
        for (indice, muestras) in [1, 2, 4].into_iter().enumerate() {
            let boton = diseno.calidad(indice);
            estudio.pulsar(diseno, boton.x + 10.0, boton.y + 10.0);
            assert_eq!(estudio.salida.muestras, muestras);
            assert_eq!(estudio.salida.exposicion, 1.4);
            assert_eq!(estudio.onda.amplitud, 0.35);
        }
        estudio.seleccionado = 1;
        let oculto = diseno.calidad(0);
        estudio.pulsar(diseno, oculto.x + 10.0, oculto.y + 10.0);
        assert_eq!(estudio.salida.muestras, 4);
    }
}
