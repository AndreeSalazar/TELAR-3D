//! Perfil de vida separado de las muestras: precisión, límites y rechazo de mezclas.
use telar_intercambio::{Particula, codificar_particulas_geo, decodificar_geo, decodificar_particulas_geo};
#[test]
fn vida_y_nacimiento_no_se_pierden_al_intercambiar() -> Result<(), &'static str> {
    let ejemplo = decodificar_particulas_geo(include_str!("../../../docs/ejemplos/particulas-telar.geo"))?;
    assert_eq!(ejemplo.len(), 2);
    assert_eq!(ejemplo[0].punto.id, 128);
    let base = telar_intercambio::decodificar_geo(include_str!("../../../docs/ejemplos/muestras-telar.geo"))?;
    let p = Particula { punto: base[0], edad: 0.0625, vida: 1.5, nacimiento: 0.0625 };
    let texto = codificar_particulas_geo(&[p])?;
    assert_eq!(decodificar_particulas_geo(&texto)?, [p]);
    assert!(decodificar_geo(&texto).is_err());
    assert!(decodificar_particulas_geo(include_str!("../../../docs/ejemplos/muestras-telar.geo")).is_err());
    for q in [
        Particula { vida: 0.0, ..p },
        Particula { edad: 1.5, ..p },
        Particula { edad: f32::NAN, ..p },
        Particula { nacimiento: 0.25, ..p },
        Particula { edad: -0.01, ..p },
    ] {
        assert!(codificar_particulas_geo(&[q]).is_err());
    }
    assert!(decodificar_particulas_geo(&texto.replace("age 1 float", "age 2 float")).is_err());
    let fin = texto.find("endExtra").ok_or("sin terminador")? + 7;
    for i in 0..fin {
        assert!(decodificar_particulas_geo(&texto[..i]).is_err());
    }
    assert_eq!(decodificar_particulas_geo(&codificar_particulas_geo(&[])?)?, [] as [Particula; 0]);
    Ok(())
}
