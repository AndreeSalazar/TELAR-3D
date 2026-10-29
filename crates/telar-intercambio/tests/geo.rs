//! Casos independientes y bordes del contrato; no requieren Houdini ni bibliotecas externas.
use telar_intercambio::{MAX_BYTES, MAX_PUNTOS, Punto, codificar_geo, decodificar_geo, validar};
const FIJO: &str = include_str!("../../../docs/ejemplos/muestras-telar.geo");
#[test]
fn lee_datos_conocidos_y_conserva_precision() -> Result<(), &'static str> {
    assert_eq!(core::mem::size_of::<Punto>(), 40);
    let mut puntos = decodificar_geo(FIJO)?;
    assert_eq!(
        puntos[0],
        Punto {
            posicion: [1.0, 2.0, 3.0],
            velocidad: [4.0, 5.0, 6.0],
            densidad: 0.25,
            calor: 0.5,
            id: 7,
            tiempo: 0.125
        }
    );
    assert_eq!(puntos[1].id, 9);
    puntos[0].posicion = [-0.0, f32::from_bits(1), 1e20];
    puntos[1].velocidad = [-1e-20, 1e-30, -1e20];
    let texto = codificar_geo(&puntos)?;
    let vuelta = decodificar_geo(&texto)?;
    for (a, b) in puntos.iter().zip(&vuelta) {
        assert_eq!(a.posicion.map(f32::to_bits), b.posicion.map(f32::to_bits));
        assert_eq!(a.velocidad.map(f32::to_bits), b.velocidad.map(f32::to_bits));
        assert_eq!(a.densidad.to_bits(), b.densidad.to_bits());
        assert_eq!(a.calor.to_bits(), b.calor.to_bits());
        assert_eq!(a.tiempo.to_bits(), b.tiempo.to_bits());
        assert_eq!(a.id, b.id);
    }
    assert_eq!(texto, codificar_geo(&puntos)?);
    assert_eq!(decodificar_geo(&codificar_geo(&[])?)?, [] as [Punto; 0]);
    Ok(())
}
#[test]
fn rechaza_variantes_y_truncamiento() {
    for (original, cambio) in [
        ("V5", "V6"),
        ("NPrims 0", "NPrims 1"),
        ("NPointAttrib 5", "NPointAttrib 6"),
        ("id 1 int", "id 1 float"),
        ("v 3 float", "v 2 float"),
        ("heat 1", "temperature 1"),
        ("0.25", "NaN"),
        ("0.5 7", "-1 7"),
        ("0.125", "inf"),
        ("6 0.25", "6e999 0.25"),
        ("0.5 7", "0.5 2147483648"),
        ("2 9", "2 7"),
        ("1(4", "2(4"),
    ] {
        assert!(decodificar_geo(&FIJO.replace(original, cambio)).is_err(), "{cambio}");
    }
    let fin = FIJO.find("endExtra").unwrap_or(0) + 7;
    for i in 0..fin {
        assert!(decodificar_geo(&FIJO[..i]).is_err(), "truncado {i}");
    }
    assert!(decodificar_geo(&format!("{FIJO} contenido")).is_err());
    assert!(decodificar_geo("{\"fileversion\":1}").is_err());
    assert!(decodificar_geo("\u{7f}NSJ").is_err());
    assert!(decodificar_geo(&FIJO.replace("heat", "calór")).is_err());
}
#[test]
fn limites_e_identidades_se_validan_antes_de_salida() -> Result<(), &'static str> {
    let puntos = decodificar_geo(FIJO)?;
    assert!(validar(&vec![puntos[0]; MAX_PUNTOS + 1]).is_err());
    assert!(codificar_geo(&[puntos[0], puntos[0]]).is_err());
    assert!(decodificar_geo(&FIJO.replace("NPoints 2", "NPoints 65537")).is_err());
    assert!(decodificar_geo(&FIJO.replace("NPoints 2", "NPoints 999999999999999999999999")).is_err());
    assert!(decodificar_geo(&" ".repeat(MAX_BYTES + 1)).is_err());
    let mut invalido = puntos[0];
    invalido.velocidad[0] = f32::INFINITY;
    assert!(codificar_geo(&[invalido]).is_err());
    Ok(())
}
