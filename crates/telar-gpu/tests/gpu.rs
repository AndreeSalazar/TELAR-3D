//! Las decisiones puras de la GPU y un contexto de Vulkan de verdad (sin
//! ventana), con las capas de validación: cero avisos.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    reason = "en los tests, un fallo es un panic con su mensaje; los floats comparados son exactos"
)]

use telar_gpu::eleccion::{
    FORMATO_SRGB, FORMATO_UNORM, TIPO_DEDICADA, TIPO_INTEGRADA, color_para, elegir_extension, elegir_formato,
    elegir_imagenes, hex, mezclar, puntuar, srgb_a_lineal,
};
use telar_gpu::{Calculo, Constantes, Opciones, TAMANO_CONSTANTES, avisos_de_validacion};

#[test]
fn la_gpu_dedicada_gana() {
    assert!(puntuar(TIPO_DEDICADA) > puntuar(TIPO_INTEGRADA));
    assert!(puntuar(TIPO_INTEGRADA) > puntuar(4)); // CPU
}

#[test]
fn formato_srgb_si_existe() {
    assert_eq!(elegir_formato(&[(FORMATO_UNORM, 0), (FORMATO_SRGB, 0)]), Some((FORMATO_SRGB, 0)));
    assert_eq!(elegir_formato(&[(FORMATO_UNORM, 0)]), Some((FORMATO_UNORM, 0)));
    assert_eq!(elegir_formato(&[(64, 7)]), Some((64, 7)));
    assert_eq!(elegir_formato(&[]), None);
}

#[test]
fn tamano_e_imagenes() {
    // La superficie fija el tamaño: manda ella.
    assert_eq!(elegir_extension((800, 600), (1, 1), (4096, 4096), (1280, 720)), (800, 600));
    // Lo deja libre (u32::MAX): el de la ventana, dentro de los límites.
    assert_eq!(elegir_extension((u32::MAX, u32::MAX), (1, 1), (1000, 1000), (1280, 720)), (1000, 720));
    assert_eq!(elegir_imagenes(2, 0), 3);
    assert_eq!(elegir_imagenes(2, 2), 2);
}

#[test]
fn colores_de_pantalla_a_luz_lineal() {
    assert_eq!(srgb_a_lineal(0.0), 0.0);
    assert_eq!(srgb_a_lineal(1.0), 1.0);
    assert!((srgb_a_lineal(0.5) - 0.214).abs() < 0.001);
    assert_eq!(hex(0x35_e3ff), [53.0 / 255.0, 227.0 / 255.0, 1.0, 1.0]);
    // En una imagen UNORM el color va tal cual; en una sRGB, en lineal.
    assert_eq!(color_para(FORMATO_UNORM, [0.5; 4]), [0.5; 4]);
    assert!(color_para(FORMATO_SRGB, [0.5; 4])[0] < 0.5);
    assert_eq!(mezclar([0.0; 4], [1.0; 4], 0.25), [0.25; 4]);
}

/// Si el sistema no tiene Vulkan (una máquina sin GPU), el test lo dice y no
/// cuenta como fallo: aquí se prueba TELAR, no el driver.
#[test]
fn contexto_de_vulkan_sin_avisos() {
    let calculo = match Calculo::nuevo(&Opciones::default()) {
        Ok(c) => c,
        Err(e) if e.contains("vulkan-1.dll") => {
            eprintln!("sin Vulkan en esta máquina: {e}");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    eprintln!("GPU: {} · validación: {}", calculo.dispositivo(), calculo.validacion());
    assert_ne!(calculo.dispositivo(), "");
    drop(calculo);
    assert_eq!(avisos_de_validacion(), 0, "las capas de validación avisaron (ver arriba)");
}

#[test]
fn memoria_con_lo_necesario_y_mejor_con_lo_preferido() {
    use telar_gpu::eleccion::{MEMORIA_COHERENTE, MEMORIA_CON_CACHE, MEMORIA_GPU, MEMORIA_VISIBLE, elegir_memoria};
    let visible = MEMORIA_VISIBLE | MEMORIA_COHERENTE;
    let tipos = [MEMORIA_GPU, visible, visible | MEMORIA_CON_CACHE];
    // Para leer en la CPU: visible y coherente; mejor con caché.
    assert_eq!(elegir_memoria(&tipos, 0b111, visible, MEMORIA_CON_CACHE), Some(2));
    // Si el recurso no admite el tipo con caché (bit 2 apagado), el otro.
    assert_eq!(elegir_memoria(&tipos, 0b011, visible, MEMORIA_CON_CACHE), Some(1));
    assert_eq!(elegir_memoria(&tipos, 0b111, MEMORIA_GPU, 0), Some(0));
    assert_eq!(elegir_memoria(&tipos, 0b001, visible, 0), None);
}

#[test]
fn grupos_y_tiempos() {
    use telar_gpu::eleccion::{grupos, nanosegundos};
    assert_eq!(grupos(1 << 24, 256), 65_536);
    assert_eq!(grupos(257, 256), 2); // el último grupo, incompleto
    assert_eq!(grupos(0, 256), 0);
    // 1000 tics de 1 ns.
    assert_eq!(nanosegundos(500, 1500, 64, 1.0), 1000.0);
    // Un contador de 36 bits que da la vuelta entre las dos lecturas.
    let maximo = (1u64 << 36) - 1;
    assert_eq!(nanosegundos(maximo - 9, 10, 36, 1.0), 20.0);
}

#[test]
fn demasiados_grupos_es_un_error_antes_de_tocar_la_gpu() {
    let Ok(calculo) = Calculo::nuevo(&Opciones::default()) else {
        return; // sin Vulkan: lo dice contexto_de_vulkan_sin_avisos
    };
    let error = calculo.ejecutar(&[], u32::MAX, 1).err().unwrap_or_default();
    assert!(error.contains("grupos"), "{error}");
}

#[test]
fn las_constantes_van_en_el_orden_de_los_shaders() {
    let identidad = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let c = Constantes { matriz: identidad, luz: [0.0, 1.0, 0.0], color: [0.5; 4], cielo: [1.0; 4], suelo: [0.0; 4] };
    let p = c.plana(FORMATO_UNORM);
    assert_eq!(core::mem::size_of_val(&p), TAMANO_CONSTANTES as usize);
    // c0…c3, luz (con w = 0), color, cielo, suelo.
    assert_eq!(p[..16], [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    assert_eq!(p[16..20], [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(p[20..24], [0.5; 4]);
    assert_eq!(p[24..32], [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    // En una imagen sRGB, los colores van en lineal (el alfa, tal cual).
    let s = c.plana(FORMATO_SRGB);
    assert!(s[20] < 0.5 && s[23] == 0.5);
    assert_eq!(s[..20], p[..20]);
}
