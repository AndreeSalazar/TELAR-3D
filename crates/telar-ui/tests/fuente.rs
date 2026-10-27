//! Rechazo de atlas incompletos antes de calcular coordenadas de glifos.
use telar_ui::{Distribucion, Estudio, Fuente};

#[test]
fn fuente_incompleta_se_rechaza_y_caracteres_ausentes_usan_respaldo() {
    let mut fuente = Fuente {
        ancho: 8,
        alto: 8,
        celda: 8,
        altura: 8.0,
        primero: u32::from('?'),
        avances: vec![4.0],
        cobertura: vec![255; 64],
    };
    let estudio = Estudio::default();
    let diseno = Distribucion::nueva((1280, 720));
    assert!(fuente.valida());
    assert!(estudio.dibujar_con_fuente(diseno, 4096, &fuente).is_ok());
    fuente.avances[0] = f32::NAN;
    assert!(estudio.dibujar_con_fuente(diseno, 4096, &fuente).is_err());
    fuente.avances[0] = 4.0;
    fuente.cobertura.pop();
    assert!(!fuente.valida());
    fuente.cobertura.push(255);
    fuente.primero = u32::from('@');
    assert!(!fuente.valida(), "falta el carácter de respaldo");
    fuente.primero = u32::from('?');
    fuente.alto = u32::MAX;
    assert!(!fuente.valida(), "dimensiones desbordadas");
}
