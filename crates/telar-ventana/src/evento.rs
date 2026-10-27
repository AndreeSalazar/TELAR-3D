//! Los mensajes de Windows, traducidos a eventos de TELAR. Código puro: no
//! habla con el sistema, así que se prueba sin abrir ninguna ventana.

/// Lo que le pasa a la ventana.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evento {
    /// Se pulsó la X (o Alt+F4). La ventana no se cierra sola: decide la app.
    Cerrar,
    /// Nuevo tamaño del área de dibujo, en píxeles físicos. `0 × 0` al minimizar.
    Tamano {
        /// Ancho.
        ancho: u32,
        /// Alto.
        alto: u32,
    },
    /// Una tecla pulsada.
    Tecla(Tecla),
    /// Carácter generado por el teclado; independiente del código virtual.
    Texto(char),
    /// El ratón se movió (coordenadas del área de dibujo; pueden ser negativas).
    Raton {
        /// Horizontal.
        x: i32,
        /// Vertical.
        y: i32,
    },
    /// Un botón del ratón se pulsó (`pulsado`) o se soltó.
    Boton {
        /// Cuál.
        boton: Boton,
        /// `true` al pulsarlo, `false` al soltarlo.
        pulsado: bool,
    },
    /// La rueda del ratón giró: en muescas (positivo = hacia delante, alejándose
    /// de la mano). Puede ser fraccionario con ruedas o paneles de precisión.
    Rueda(f32),
}

/// Los botones del ratón.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boton {
    /// El izquierdo.
    Izquierdo,
    /// El derecho.
    Derecho,
    /// El de en medio (la rueda).
    Medio,
}

/// Las teclas que TELAR distingue; el resto llega con su código virtual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tecla {
    /// Esc.
    Escape,
    /// F11.
    F11,
    /// Cualquier otra, con su código virtual de Windows.
    Otra(u32),
}

/// `WM_SIZE`.
pub const WM_SIZE: u32 = 0x0005;
/// `WM_CLOSE`.
pub const WM_CLOSE: u32 = 0x0010;
/// `WM_KEYDOWN`.
pub const WM_KEYDOWN: u32 = 0x0100;
/// `WM_CHAR`: los campos numéricos solo necesitan caracteres del plano básico.
pub const WM_CHAR: u32 = 0x0102;
/// `WM_MOUSEMOVE`.
pub const WM_MOUSEMOVE: u32 = 0x0200;
/// `WM_LBUTTONDOWN`; `UP` es el siguiente. Los del derecho y el de en medio van
/// detrás, de tres en tres (`DOWN`, `UP`, `DBLCLK`).
pub const WM_LBUTTONDOWN: u32 = 0x0201;
/// `WM_LBUTTONUP`.
pub const WM_LBUTTONUP: u32 = 0x0202;
/// `WM_RBUTTONDOWN`.
pub const WM_RBUTTONDOWN: u32 = 0x0204;
/// `WM_RBUTTONUP`.
pub const WM_RBUTTONUP: u32 = 0x0205;
/// `WM_MBUTTONDOWN`.
pub const WM_MBUTTONDOWN: u32 = 0x0207;
/// `WM_MBUTTONUP`.
pub const WM_MBUTTONUP: u32 = 0x0208;
/// `WM_MOUSEWHEEL`: el giro va en la palabra alta del `wParam`, con signo.
pub const WM_MOUSEWHEEL: u32 = 0x020A;
/// `WHEEL_DELTA`: lo que mide una muesca.
const WHEEL_DELTA: f32 = 120.0;
/// `SIZE_MINIMIZED`, en el `wParam` de `WM_SIZE`.
const SIZE_MINIMIZED: usize = 1;

/// Traduce un mensaje de Windows (`mensaje`, `wParam`, `lParam`) a un evento,
/// si es uno de los que TELAR escucha.
#[must_use]
pub fn traducir(mensaje: u32, w: usize, l: isize) -> Option<Evento> {
    match mensaje {
        WM_CLOSE => Some(Evento::Cerrar),
        WM_SIZE if w == SIZE_MINIMIZED => Some(Evento::Tamano { ancho: 0, alto: 0 }),
        WM_SIZE => Some(Evento::Tamano { ancho: u32::from(bajo(l)), alto: u32::from(alto(l)) }),
        WM_CHAR => u32::try_from(w).ok().and_then(char::from_u32).map(Evento::Texto),
        WM_KEYDOWN => Some(Evento::Tecla(match w {
            0x1B => Tecla::Escape,
            0x7A => Tecla::F11,
            otra => Tecla::Otra(u32::try_from(otra).unwrap_or(0)),
        })),
        // Las coordenadas van con signo: a la izquierda de la ventana, son negativas.
        WM_MOUSEMOVE => Some(Evento::Raton {
            x: i32::from(i16::from_ne_bytes(bajo(l).to_ne_bytes())),
            y: i32::from(i16::from_ne_bytes(alto(l).to_ne_bytes())),
        }),
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP => {
            let boton = match mensaje {
                WM_LBUTTONDOWN | WM_LBUTTONUP => Boton::Izquierdo,
                WM_RBUTTONDOWN | WM_RBUTTONUP => Boton::Derecho,
                _ => Boton::Medio,
            };
            Some(Evento::Boton { boton, pulsado: matches!(mensaje, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN) })
        }
        WM_MOUSEWHEEL => {
            let giro = i16::from_ne_bytes(alto(w.cast_signed()).to_ne_bytes());
            Some(Evento::Rueda(f32::from(giro) / WHEEL_DELTA))
        }
        _ => None,
    }
}

/// `LOWORD`: los 16 bits bajos.
fn bajo(l: isize) -> u16 {
    u16::try_from(l & 0xFFFF).unwrap_or(0)
}

/// `HIWORD`: los 16 bits siguientes.
fn alto(l: isize) -> u16 {
    u16::try_from((l >> 16) & 0xFFFF).unwrap_or(0)
}

/// Texto en UTF-16 terminado en cero, como lo quiere Win32.
#[must_use]
pub fn utf16(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(core::iter::once(0)).collect()
}
