//! La fuente real conserva métricas proporcionales y cobertura suave.
#![allow(clippy::expect_used, reason = "fallos explícitos de verificación")]

#[test]
fn atlas_del_sistema_tiene_glifos_proporcionales_y_antialias() {
    let fuente = telar_ventana::fuente_interfaz().expect("GDI disponible");
    assert_eq!(fuente.cobertura.len(), (fuente.ancho * fuente.alto) as usize);
    assert_eq!(fuente.cobertura[0], 255);
    assert!(fuente.cobertura.iter().any(|v| *v > 0 && *v < 255));
    let avance = |letra| fuente.avances[(u32::from(letra) - fuente.primero) as usize];
    assert!(avance('W') > avance('i') * 1.5);
    assert!(avance(' ') > 0.0);
}
