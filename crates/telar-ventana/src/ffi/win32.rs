//! Win32 a mano: los tipos, las constantes y las funciones que usa la ventana,
//! con las firmas de la documentación de Microsoft (winuser.h, libloaderapi.h,
//! errhandlingapi.h, wingdi.h y dwmapi.h). Los manejadores son `isize`, como
//! `HANDLE` en 64 bits.

use core::ffi::c_void;

/// El procedimiento de ventana (`WNDPROC`).
pub(crate) type ProcVentana = unsafe extern "system" fn(isize, u32, usize, isize) -> isize;

/// `WNDCLASSEXW`.
#[repr(C)]
pub(crate) struct ClaseVentana {
    pub(crate) tamano: u32,
    pub(crate) estilo: u32,
    pub(crate) procedimiento: ProcVentana,
    pub(crate) extra_clase: i32,
    pub(crate) extra_ventana: i32,
    pub(crate) instancia: isize,
    pub(crate) icono: isize,
    pub(crate) cursor: isize,
    pub(crate) fondo: isize,
    pub(crate) menu: *const u16,
    pub(crate) nombre: *const u16,
    pub(crate) icono_pequeno: isize,
}

/// `MSG` (con `lPrivate`, que cabe en el relleno final de la versión antigua).
#[repr(C)]
#[derive(Default)]
pub(crate) struct Mensaje {
    pub(crate) ventana: isize,
    pub(crate) codigo: u32,
    pub(crate) w: usize,
    pub(crate) l: isize,
    pub(crate) tiempo: u32,
    pub(crate) punto: [i32; 2],
    pub(crate) privado: u32,
}

/// `RECT`.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub(crate) struct Rect {
    pub(crate) izquierda: i32,
    pub(crate) arriba: i32,
    pub(crate) derecha: i32,
    pub(crate) abajo: i32,
}

pub(crate) const CS_HREDRAW: u32 = 0x0002;
pub(crate) const CS_VREDRAW: u32 = 0x0001;
pub(crate) const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
pub(crate) const SW_SHOW: i32 = 5;
pub(crate) const PM_REMOVE: u32 = 0x0001;
pub(crate) const WM_ERASEBKGND: u32 = 0x0014;
pub(crate) const WM_DPICHANGED: u32 = 0x02E0;
pub(crate) const SWP_NOMOVE: u32 = 0x0002;
pub(crate) const SWP_NOZORDER: u32 = 0x0004;
pub(crate) const SWP_NOACTIVATE: u32 = 0x0010;
pub(crate) const SM_CXSCREEN: i32 = 0;
pub(crate) const SM_CYSCREEN: i32 = 1;
/// `IDC_ARROW` (un recurso por número, `MAKEINTRESOURCE(32512)`).
pub(crate) const IDC_ARROW: usize = 32512;
/// `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`: nítida en cada monitor.
pub(crate) const DPI_POR_MONITOR_V2: isize = -4;
/// `DWMWA_USE_IMMERSIVE_DARK_MODE`: barra de título oscura.
pub(crate) const DWMWA_MODO_OSCURO: u32 = 20;
/// `ERROR_CLASS_ALREADY_EXISTS`: la clase ya está registrada (otra ventana del proceso).
pub(crate) const ERROR_CLASE_EXISTE: u32 = 1410;

// SAFETY: firmas idénticas a las de winuser.h (user32.dll), con `isize` para los
// manejadores y `*const u16` para el texto UTF-16 terminado en cero.
#[link(name = "user32")]
unsafe extern "system" {
    pub(crate) fn RegisterClassExW(clase: *const ClaseVentana) -> u16;
    pub(crate) fn CreateWindowExW(
        estilo_ext: u32,
        clase: *const u16,
        titulo: *const u16,
        estilo: u32,
        x: i32,
        y: i32,
        ancho: i32,
        alto: i32,
        padre: isize,
        menu: isize,
        instancia: isize,
        parametro: *mut c_void,
    ) -> isize;
    pub(crate) fn DefWindowProcW(ventana: isize, mensaje: u32, w: usize, l: isize) -> isize;
    pub(crate) fn DestroyWindow(ventana: isize) -> i32;
    pub(crate) fn ShowWindow(ventana: isize, comando: i32) -> i32;
    pub(crate) fn PeekMessageW(mensaje: *mut Mensaje, ventana: isize, min: u32, max: u32, quitar: u32) -> i32;
    pub(crate) fn TranslateMessage(mensaje: *const Mensaje) -> i32;
    pub(crate) fn DispatchMessageW(mensaje: *const Mensaje) -> isize;
    pub(crate) fn LoadCursorW(instancia: isize, nombre: *const u16) -> isize;
    pub(crate) fn GetClientRect(ventana: isize, rect: *mut Rect) -> i32;
    pub(crate) fn AdjustWindowRectEx(rect: *mut Rect, estilo: u32, menu: i32, estilo_ext: u32) -> i32;
    pub(crate) fn GetSystemMetrics(indice: i32) -> i32;
    pub(crate) fn SetProcessDpiAwarenessContext(contexto: isize) -> i32;
    pub(crate) fn GetDpiForSystem() -> u32;
    pub(crate) fn SetWindowPos(
        ventana: isize,
        despues: isize,
        x: i32,
        y: i32,
        ancho: i32,
        alto: i32,
        banderas: u32,
    ) -> i32;
    pub(crate) fn SetWindowTextW(ventana: isize, texto: *const u16) -> i32;
}

// SAFETY: firmas de libloaderapi.h y errhandlingapi.h (kernel32.dll).
#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetModuleHandleW(nombre: *const u16) -> isize;
    pub(crate) fn GetLastError() -> u32;
}

// SAFETY: firma de wingdi.h (gdi32.dll); el color es un COLORREF (0x00BBGGRR).
#[link(name = "gdi32")]
unsafe extern "system" {
    pub(crate) fn CreateSolidBrush(color: u32) -> isize;
}

// SAFETY: firma de dwmapi.h (dwmapi.dll); `valor` apunta a `tamano` bytes.
#[link(name = "dwmapi")]
unsafe extern "system" {
    pub(crate) fn DwmSetWindowAttribute(ventana: isize, atributo: u32, valor: *const c_void, tamano: u32) -> i32;
}
