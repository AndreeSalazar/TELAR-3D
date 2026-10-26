//! La única puerta de la ventana al sistema. Todo el `unsafe` del crate vive
//! aquí, cada uno con su `// SAFETY:`; hacia fuera solo salen funciones seguras
//! que reciben y devuelven tipos de Rust.

#![allow(unsafe_code, reason = "la puerta a Win32: cada unsafe lleva su // SAFETY:")]

mod win32;

use core::cell::RefCell;
use core::ffi::c_void;

use crate::evento::{Evento, WM_CLOSE, traducir, utf16};
use win32::{
    AdjustWindowRectEx, ClaseVentana, CreateSolidBrush, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, DwmSetWindowAttribute, GetClientRect, GetDpiForSystem, GetLastError, GetModuleHandleW,
    GetSystemMetrics, LoadCursorW, Mensaje, PeekMessageW, Rect, RegisterClassExW, SetProcessDpiAwarenessContext,
    SetWindowPos, SetWindowTextW, ShowWindow, TranslateMessage,
};

/// El fondo mientras Vulkan no ha pintado el primer fotograma: el negro del cosmos
/// (#05070a), sin el destello blanco al abrir.
const FONDO: u32 = 0x000A_0705;

std::thread_local! {
    /// Los eventos que deja el procedimiento de ventana hasta que la app los recoge.
    /// Una cola por hilo: la ventana es de su hilo, como exige Win32.
    static COLA: RefCell<Vec<Evento>> = const { RefCell::new(Vec::new()) };
}

/// Recibe los mensajes de Windows: los traduce a eventos y deja el resto a Windows.
extern "system" fn procedimiento(ventana: isize, mensaje: u32, w: usize, l: isize) -> isize {
    match mensaje {
        // Pinta Vulkan: sin borrar el fondo, no hay parpadeo al redimensionar.
        win32::WM_ERASEBKGND => return 1,
        win32::WM_DPICHANGED => {
            let sugerido = core::ptr::with_exposed_provenance::<Rect>(l.cast_unsigned());
            // SAFETY: con WM_DPICHANGED, lParam apunta a un RECT válido mientras dura
            // la llamada (documentación de WM_DPICHANGED); solo se lee.
            unsafe {
                let r = *sugerido;
                SetWindowPos(
                    ventana,
                    0,
                    r.izquierda,
                    r.arriba,
                    r.derecha - r.izquierda,
                    r.abajo - r.arriba,
                    win32::SWP_NOZORDER | win32::SWP_NOACTIVATE,
                );
            }
            return 0;
        }
        _ => {}
    }
    if let Some(evento) = traducir(mensaje, w, l) {
        COLA.with(|cola| {
            if let Ok(mut cola) = cola.try_borrow_mut() {
                cola.push(evento);
            }
        });
        if mensaje == WM_CLOSE {
            return 0; // la ventana no se destruye sola: decide la app
        }
    }
    // SAFETY: se pasan a Windows los mismos argumentos que nos dio, para el
    // comportamiento por defecto de la ventana que es nuestra.
    unsafe { DefWindowProcW(ventana, mensaje, w, l) }
}

/// Crea la ventana centrada en la pantalla, con `ancho × alto` píxeles lógicos de
/// área de dibujo. Devuelve `(instancia, ventana)`.
pub(crate) fn crear(titulo: &str, ancho: u32, alto: u32, visible: bool) -> Result<(isize, isize), String> {
    let clase = utf16("TELAR3D.Ventana");
    let titulo = utf16(titulo);

    // SAFETY: llamadas sin punteros (o con null, que Win32 admite: el módulo propio
    // y la flecha del sistema). SetProcessDpiAwarenessContext falla si ya se fijó
    // (por un manifiesto o una ventana anterior), y eso no es un error.
    let (instancia, cursor, fondo, dpi, pantalla) = unsafe {
        SetProcessDpiAwarenessContext(win32::DPI_POR_MONITOR_V2);
        let instancia = GetModuleHandleW(core::ptr::null());
        let cursor = LoadCursorW(0, core::ptr::without_provenance(win32::IDC_ARROW));
        let pantalla = (GetSystemMetrics(win32::SM_CXSCREEN), GetSystemMetrics(win32::SM_CYSCREEN));
        (instancia, cursor, CreateSolidBrush(FONDO), GetDpiForSystem().max(96), pantalla)
    };

    let datos = ClaseVentana {
        tamano: u32::try_from(size_of::<ClaseVentana>()).unwrap_or(0),
        estilo: win32::CS_HREDRAW | win32::CS_VREDRAW,
        procedimiento,
        extra_clase: 0,
        extra_ventana: 0,
        instancia,
        icono: 0,
        cursor,
        fondo,
        menu: core::ptr::null(),
        nombre: clase.as_ptr(),
        icono_pequeno: 0,
    };
    // SAFETY: `datos` y el nombre de la clase viven durante la llamada (Windows
    // copia lo que necesita). Registrar dos veces la misma clase no es un error.
    let registrada = unsafe { RegisterClassExW(&raw const datos) != 0 || GetLastError() == win32::ERROR_CLASE_EXISTE };
    if !registrada {
        return Err("no se pudo registrar la clase de la ventana".to_owned());
    }

    // Tamaño físico del área de dibujo y, con la barra y los bordes, de la ventana.
    let escala = |v: u32| i32::try_from(u64::from(v) * u64::from(dpi) / 96).unwrap_or(i32::MAX);
    let mut marco = Rect { izquierda: 0, arriba: 0, derecha: escala(ancho), abajo: escala(alto) };
    // SAFETY: `marco` es un RECT válido y nuestro durante la llamada.
    unsafe { AdjustWindowRectEx(&raw mut marco, win32::WS_OVERLAPPEDWINDOW, 0, 0) };
    let (total_x, total_y) = (marco.derecha - marco.izquierda, marco.abajo - marco.arriba);
    let (pos_x, pos_y) = (((pantalla.0 - total_x) / 2).max(0), ((pantalla.1 - total_y) / 2).max(0));

    // SAFETY: la clase está registrada y el título vive durante la llamada; el
    // procedimiento puede ejecutarse aquí dentro (WM_CREATE, WM_SIZE…) y solo
    // toca la cola de este hilo.
    let ventana = unsafe {
        CreateWindowExW(
            0,
            clase.as_ptr(),
            titulo.as_ptr(),
            win32::WS_OVERLAPPEDWINDOW,
            pos_x,
            pos_y,
            total_x,
            total_y,
            0,
            0,
            instancia,
            core::ptr::null_mut(),
        )
    };
    if ventana == 0 {
        return Err("no se pudo crear la ventana".to_owned());
    }

    let oscura: i32 = 1;
    // SAFETY: la ventana existe; `oscura` vive durante la llamada y mide 4 bytes,
    // como dice el último argumento. Si el sistema no lo admite, se ignora.
    unsafe {
        DwmSetWindowAttribute(ventana, win32::DWMWA_MODO_OSCURO, (&raw const oscura).cast::<c_void>(), 4);
        if visible {
            ShowWindow(ventana, win32::SW_SHOW);
        }
    }
    Ok((instancia, ventana))
}

/// Atiende los mensajes pendientes del hilo y devuelve los eventos acumulados.
pub(crate) fn bombear() -> Vec<Evento> {
    let mut mensaje = Mensaje::default();
    // SAFETY: `mensaje` es nuestro y vive durante todo el bucle; PeekMessageW lo
    // rellena y las otras dos solo lo leen.
    unsafe {
        while PeekMessageW(&raw mut mensaje, 0, 0, 0, win32::PM_REMOVE) != 0 {
            TranslateMessage(&raw const mensaje);
            DispatchMessageW(&raw const mensaje);
        }
    }
    COLA.with(|cola| cola.try_borrow_mut().map(|mut c| core::mem::take(&mut *c)).unwrap_or_default())
}

/// Tamaño del área de dibujo, en píxeles físicos.
pub(crate) fn tamano(ventana: isize) -> (u32, u32) {
    let mut r = Rect::default();
    // SAFETY: `r` es nuestro; si la ventana ya no existe, GetClientRect falla y
    // `r` se queda a cero.
    unsafe { GetClientRect(ventana, &raw mut r) };
    (u32::try_from(r.derecha).unwrap_or(0), u32::try_from(r.abajo).unwrap_or(0))
}

/// Cambia el texto de la barra de título.
pub(crate) fn poner_titulo(ventana: isize, titulo: &str) {
    let texto = utf16(titulo);
    // SAFETY: el texto UTF-16 terminado en cero vive durante la llamada.
    unsafe { SetWindowTextW(ventana, texto.as_ptr()) };
}

/// Destruye la ventana (desde su hilo).
pub(crate) fn destruir(ventana: isize) {
    // SAFETY: la ventana es nuestra y se destruye una sola vez (desde Drop).
    unsafe { DestroyWindow(ventana) };
}

/// Cambia el área de dibujo a `ancho × alto` píxeles físicos, sin moverla. Windows
/// manda `WM_SIZE` a este hilo, y llega como `Evento::Tamano`.
pub(crate) fn cambiar_tamano(ventana: isize, ancho: u32, alto: u32) {
    let a_i32 = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    let mut marco = Rect { izquierda: 0, arriba: 0, derecha: a_i32(ancho), abajo: a_i32(alto) };
    let banderas = win32::SWP_NOMOVE | win32::SWP_NOZORDER | win32::SWP_NOACTIVATE;
    // SAFETY: `marco` es nuestro durante las dos llamadas y la ventana es de este
    // hilo; el WM_SIZE que provoca lo atiende `procedimiento`, que solo toca la cola.
    unsafe {
        AdjustWindowRectEx(&raw mut marco, win32::WS_OVERLAPPEDWINDOW, 0, 0);
        SetWindowPos(ventana, 0, 0, 0, marco.derecha - marco.izquierda, marco.abajo - marco.arriba, banderas);
    }
}
