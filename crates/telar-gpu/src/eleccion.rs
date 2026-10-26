//! Las decisiones de la GPU que no necesitan hablar con ella: qué dispositivo,
//! qué formato, qué tamaño y cuántas imágenes. Código puro, sin `unsafe`, y
//! probado sin GPU.

/// `VK_FORMAT_B8G8R8A8_UNORM`.
pub const FORMATO_UNORM: u32 = 44;
/// `VK_FORMAT_B8G8R8A8_SRGB`: el preferido (la GPU convierte a sRGB al escribir).
pub const FORMATO_SRGB: u32 = 50;
/// `VK_COLOR_SPACE_SRGB_NONLINEAR_KHR`.
pub const ESPACIO_SRGB: u32 = 0;
/// `VK_PHYSICAL_DEVICE_TYPE_*`: integrada, dedicada, virtual y CPU.
pub const TIPO_INTEGRADA: u32 = 1;
/// GPU dedicada (la RTX 3060).
pub const TIPO_DEDICADA: u32 = 2;

/// Cuánto vale un tipo de dispositivo: la GPU dedicada por delante de todo.
#[must_use]
pub fn puntuar(tipo: u32) -> u32 {
    match tipo {
        TIPO_DEDICADA => 1000,
        TIPO_INTEGRADA => 100,
        3 => 10, // virtual
        _ => 1,  // CPU u otro
    }
}

/// El formato de la swapchain: sRGB si existe, si no UNORM, y si no el primero.
/// Cada opción es `(formato, espacio de color)`.
#[must_use]
pub fn elegir_formato(disponibles: &[(u32, u32)]) -> Option<(u32, u32)> {
    [FORMATO_SRGB, FORMATO_UNORM]
        .iter()
        .find_map(|&f| disponibles.iter().copied().find(|&d| d == (f, ESPACIO_SRGB)))
        .or_else(|| disponibles.first().copied())
}

/// El tamaño de las imágenes. Si la superficie lo fija (`actual`), ese; si lo deja
/// libre (`u32::MAX`), el de la ventana, dentro de los límites.
#[must_use]
pub fn elegir_extension(actual: (u32, u32), minima: (u32, u32), maxima: (u32, u32), ventana: (u32, u32)) -> (u32, u32) {
    if actual.0 != u32::MAX {
        return actual;
    }
    (ventana.0.clamp(minima.0, maxima.0), ventana.1.clamp(minima.1, maxima.1))
}

/// Cuántas imágenes en la swapchain: una más que el mínimo (para no esperar al
/// driver), sin pasar del máximo (`0` = sin máximo).
#[must_use]
pub fn elegir_imagenes(minimo: u32, maximo: u32) -> u32 {
    let deseadas = minimo + 1;
    if maximo == 0 { deseadas } else { deseadas.min(maximo) }
}

/// Un canal sRGB (0–1) en luz lineal: lo que hay que escribir en una imagen sRGB
/// para que en pantalla salga ese color.
#[must_use]
pub fn srgb_a_lineal(c: f32) -> f32 {
    if c <= 0.040_45 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Un color de pantalla (sRGB, como en CSS) preparado para el formato: en lineal
/// si la imagen es sRGB, tal cual si no. El alfa no se toca.
#[must_use]
pub fn color_para(formato: u32, srgb: [f32; 4]) -> [f32; 4] {
    if formato == FORMATO_SRGB {
        [srgb_a_lineal(srgb[0]), srgb_a_lineal(srgb[1]), srgb_a_lineal(srgb[2]), srgb[3]]
    } else {
        srgb
    }
}

/// `#rrggbb` → `[r, g, b, 1]` en sRGB (0–1).
#[must_use]
pub fn hex(rgb: u32) -> [f32; 4] {
    let canal = |desplazamiento: u32| f32::from(u8::try_from((rgb >> desplazamiento) & 0xFF).unwrap_or(0)) / 255.0;
    [canal(16), canal(8), canal(0), 1.0]
}

/// Mezcla dos colores: `t = 0` es `a`, `t = 1` es `b`.
#[must_use]
pub fn mezclar(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    core::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}
