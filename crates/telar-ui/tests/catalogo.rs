//! Navegación por capacidades reales, sin lanzar procesos ni tocar escenas.
use telar_ui::{CATEGORIAS, Catalogo, DisenoCatalogo, Fuente};
fn fuente() -> Fuente {
    Fuente {
        ancho: 8,
        alto: 8,
        celda: 8,
        altura: 8.0,
        primero: u32::from('?'),
        avances: vec![4.0],
        cobertura: vec![255; 64],
    }
}
#[test]
fn capacidades_pendientes_no_abren_editores_y_la_seleccion_no_lanza() {
    let mut catalogo = Catalogo::default();
    let d = DisenoCatalogo::nuevo((1280, 800));
    for (i, categoria) in CATEGORIAS.iter().enumerate() {
        assert!(!categoria.herramientas.is_empty() && categoria.herramientas.len() <= 6);
        let rect = d.categoria(i);
        catalogo.pulsar(d, rect.x + 10.0, rect.y + 10.0);
        assert_eq!(catalogo.indice_categoria(), i);
        assert!(catalogo.tomar_accion().is_none());
        for (j, herramienta) in categoria.herramientas.iter().enumerate() {
            let rect = d.herramienta(j);
            catalogo.pulsar(d, rect.x + 10.0, rect.y + 10.0);
            assert_eq!(catalogo.indice_herramienta(), j);
            assert!(catalogo.tomar_accion().is_none());
            let rect = d.abrir();
            catalogo.pulsar(d, rect.x + 10.0, rect.y + 10.0);
            assert_eq!(catalogo.tomar_accion(), herramienta.espacio);
            assert!(catalogo.tomar_accion().is_none());
        }
    }
    let antes = catalogo.indice_categoria();
    assert!(!catalogo.seleccionar_categoria(usize::MAX));
    assert_eq!(antes, catalogo.indice_categoria());
    assert!(!catalogo.seleccionar_herramienta(usize::MAX));
}
#[test]
fn tarjetas_y_texto_permanecen_dentro_de_la_ventana() -> Result<(), &'static str> {
    let mut catalogo = Catalogo::default();
    let fuente = fuente();
    for tamano in [(1280, 800), (1080, 700), (1920, 1080), (640, 480)] {
        let d = DisenoCatalogo::nuevo(tamano);
        for i in 0..CATEGORIAS.len() {
            catalogo.seleccionar_categoria(i);
            let vertices = catalogo.dibujar(d, &fuente)?;
            assert!(!vertices.is_empty() && vertices.len() <= 120_000);
            for v in vertices {
                assert!(v.iter().all(|v| v.is_finite()));
                assert!(v[0].abs() <= 1.0001 && v[1].abs() <= 1.0001);
            }
        }
    }
    Ok(())
}
#[test]
fn css_invalido_es_transaccional_y_la_ventana_pequena_no_interactua() -> Result<(), String> {
    let mut catalogo = Catalogo::default();
    let d = DisenoCatalogo::nuevo((1280, 800));
    let fuente = fuente();
    let original = catalogo.dibujar(d, &fuente)?;
    assert!(catalogo.cargar_estilos(".herramienta { propiedad-inexistente: rojo; }").is_err());
    assert_eq!(original, catalogo.dibujar(d, &fuente)?);
    catalogo.cargar_estilos(".herramienta { background-color: #203040; }")?;
    assert_ne!(original, catalogo.dibujar(d, &fuente)?);
    let anterior = catalogo.indice_categoria();
    catalogo.pulsar(DisenoCatalogo::nuevo((640, 480)), 40.0, 158.0);
    assert_eq!(anterior, catalogo.indice_categoria());
    assert!(catalogo.tomar_accion().is_none());
    Ok(())
}
