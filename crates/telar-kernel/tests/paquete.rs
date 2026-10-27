//! Lectura acotada, integridad y rechazo de archivos ajenos o corruptos.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de verificación")]

#[test]
fn archivo_tlar_conserva_datos_y_rechaza_truncamiento_corrupcion_y_exceso() {
    let mut cabecera = [0; 32];
    cabecera[..3].copy_from_slice(&[telar_kernel::arranque::FIRMA, 1, 32]);
    let codigo = telar_kernel::compilar_con_arranque();
    let bytes = telar_kernel::paquete::codificar(&cabecera, &codigo).unwrap();
    let paquete = telar_kernel::paquete::decodificar(&bytes).unwrap();
    assert_eq!(paquete.cabecera, cabecera);
    assert_eq!(paquete.codigo, codigo);
    for largo in [0, 1, 19, 20, 127, bytes.len() - 1] {
        assert!(telar_kernel::paquete::decodificar(&bytes[..largo]).is_err());
    }
    for indice in [0, 4, 8, 12, 16, 24, bytes.len() - 1] {
        let mut corrupto = bytes.clone();
        corrupto[indice] ^= 1;
        assert!(telar_kernel::paquete::decodificar(&corrupto).is_err());
    }
    let mut sobrante = bytes;
    sobrante.push(0);
    assert!(telar_kernel::paquete::decodificar(&sobrante).is_err());
    assert!(telar_kernel::paquete::codificar(&cabecera, &[0; 4]).is_err());
}
