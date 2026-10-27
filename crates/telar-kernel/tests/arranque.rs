//! El constructor host rechaza límites incompatibles antes de cualquier reserva.
#![allow(clippy::unwrap_used, reason = "fallos explícitos de verificación")]
use telar_kernel::arranque::{self, Dispositivo};

fn perfil() -> Dispositivo {
    Dispositivo {
        fabricante: 0x1234,
        dispositivo: 5,
        tipo: 2,
        controlador: 9,
        vulkan: 0x0040_3000,
        grupos_x: 65535,
        invocaciones: 128,
        hilos_x: 128,
        compartida: 16384,
        rango_almacen: 128 * 1024 * 1024,
        almacenes: 8,
        memoria_local: 8 * 1024 * 1024 * 1024,
    }
}

#[test]
fn encabezado_versionado_no_depende_del_fabricante_y_declara_regiones() {
    let header = arranque::cabecera(perfil(), 9, 4).unwrap();
    assert_eq!(&header[..5], &[arranque::FIRMA, arranque::VERSION, 32, 3, 3]);
    assert_eq!(&header[16..24], &[9, 4, 486, 384, 32, 64, 4, 2]);
    assert_eq!(&header[14..16], &[0, 2], "8 GiB como dos mitades u32");
    assert!(arranque::cabecera(perfil(), 1024, 32).is_ok());
    let mut otro = perfil();
    otro.fabricante = 0x5678;
    assert!(arranque::cabecera(otro, 9, 4).is_ok(), "no hay lista de fabricantes privilegiados");
}

#[test]
fn limites_y_desbordamientos_se_rechazan_en_host() {
    let mut reducido = perfil();
    reducido.rango_almacen = 96;
    assert!(arranque::cabecera(reducido, 2, 2).is_err(), "deben caber también control y cabecera");
    for (capacidad, etapas) in [(0, 4), (1, 4), (1025, 4), (u32::MAX, 4), (9, 0), (9, 33)] {
        assert!(arranque::cabecera(perfil(), capacidad, etapas).is_err());
    }
    for campo in 0..6 {
        let mut gpu = perfil();
        match campo {
            0 => gpu.vulkan = 0,
            1 => gpu.grupos_x = 1,
            2 => gpu.invocaciones = 63,
            3 => gpu.hilos_x = 63,
            4 => gpu.rango_almacen = 128,
            _ => gpu.almacenes = 4,
        }
        assert!(arranque::cabecera(gpu, 9, 4).is_err());
    }
}
