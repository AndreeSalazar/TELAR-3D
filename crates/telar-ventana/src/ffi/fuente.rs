//! GDI aislado: Segoe UI con cobertura antialias, sin `ClearType` de colores.
use crate::fuente::FuenteSistema;
use core::ffi::c_void;

#[repr(C)]
struct Cabecera {
    tamano: u32,
    ancho: i32,
    alto: i32,
    planos: u16,
    bits: u16,
    compresion: u32,
    bytes: u32,
    resolucion_x: i32,
    resolucion_y: i32,
    colores: u32,
    importantes: u32,
}
#[repr(C)]
struct InfoBitmap {
    cabecera: Cabecera,
    paleta: [u32; 1],
}
#[repr(C)]
#[derive(Default)]
struct Extension {
    ancho: i32,
    alto: i32,
}

// SAFETY: firmas de wingdi.h; los HDC/HFONT/HBITMAP se representan como isize.
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: isize) -> isize;
    fn DeleteDC(dc: isize) -> i32;
    fn CreateFontW(
        alto: i32,
        ancho: i32,
        giro: i32,
        orientacion: i32,
        peso: i32,
        cursiva: u32,
        subrayado: u32,
        tachado: u32,
        charset: u32,
        salida: u32,
        clip: u32,
        calidad: u32,
        familia: u32,
        nombre: *const u16,
    ) -> isize;
    fn CreateDIBSection(
        dc: isize,
        info: *const InfoBitmap,
        uso: u32,
        pixels: *mut *mut c_void,
        seccion: isize,
        desplazamiento: u32,
    ) -> isize;
    fn SelectObject(dc: isize, objeto: isize) -> isize;
    fn DeleteObject(objeto: isize) -> i32;
    fn SetTextColor(dc: isize, color: u32) -> u32;
    fn SetBkColor(dc: isize, color: u32) -> u32;
    fn TextOutW(dc: isize, x: i32, y: i32, texto: *const u16, cantidad: i32) -> i32;
    fn GetTextExtentPoint32W(dc: isize, texto: *const u16, cantidad: i32, extension: *mut Extension) -> i32;
    fn GdiFlush() -> i32;
}

#[derive(Default)]
struct Recursos {
    dc: isize,
    bitmap: isize,
    fuente: isize,
    anterior_bitmap: isize,
    anterior_fuente: isize,
}

impl Drop for Recursos {
    fn drop(&mut self) {
        // SAFETY: solo se restauran selecciones válidas de este DC y se destruyen
        // objetos propios, una vez y después de retirarlos del DC.
        unsafe {
            for anterior in [self.anterior_fuente, self.anterior_bitmap] {
                if anterior != 0 && anterior != -1 {
                    SelectObject(self.dc, anterior);
                }
            }
            if self.fuente != 0 {
                DeleteObject(self.fuente);
            }
            if self.bitmap != 0 {
                DeleteObject(self.bitmap);
            }
            if self.dc != 0 {
                DeleteDC(self.dc);
            }
        }
    }
}

#[allow(clippy::cast_precision_loss, reason = "métricas de glifos de menos de 40 píxeles")]
pub(crate) fn crear() -> Result<FuenteSistema, String> {
    let (ancho, alto, celda) = (640u32, 560u32, 40u32);
    let info = InfoBitmap {
        cabecera: Cabecera {
            tamano: 40,
            ancho: 640,
            alto: -560,
            planos: 1,
            bits: 32,
            compresion: 0,
            bytes: 0,
            resolucion_x: 0,
            resolucion_y: 0,
            colores: 0,
            importantes: 0,
        },
        paleta: [0],
    };
    let nombre: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    let mut recursos = Recursos::default();
    let mut pixeles = core::ptr::null_mut();
    // SAFETY: estructuras wingdi con layout C, fuente UTF-16 terminada en cero;
    // el DIB es 32 bits, top-down, y cada objeto creado queda protegido por Drop.
    unsafe {
        recursos.dc = CreateCompatibleDC(0);
        if recursos.dc == 0 {
            return Err("no se pudo crear el contexto tipográfico".to_owned());
        }
        recursos.bitmap = CreateDIBSection(recursos.dc, &raw const info, 0, &raw mut pixeles, 0, 0);
        recursos.fuente = CreateFontW(-24, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 4, 0, nombre.as_ptr());
        if recursos.bitmap == 0 || recursos.fuente == 0 || pixeles.is_null() {
            return Err("no se pudo preparar la fuente del sistema".to_owned());
        }
        recursos.anterior_bitmap = SelectObject(recursos.dc, recursos.bitmap);
        recursos.anterior_fuente = SelectObject(recursos.dc, recursos.fuente);
        if [recursos.anterior_bitmap, recursos.anterior_fuente].iter().any(|v| *v == 0 || *v == -1) {
            return Err("no se pudo seleccionar la fuente o su bitmap".to_owned());
        }
        core::ptr::write_bytes(pixeles.cast::<u8>(), 0, ancho as usize * alto as usize * 4);
        SetTextColor(recursos.dc, 0x00ff_ffff);
        SetBkColor(recursos.dc, 0);
    }
    let mut avances = Vec::with_capacity(224);
    for codigo in 32..256u16 {
        let indice = u32::from(codigo - 32);
        let mut extension = Extension::default();
        // SAFETY: un carácter UTF-16 vive durante las llamadas y su celda 40x40
        // cabe en el DIB. El DC y la fuente existen hasta terminar el atlas.
        unsafe {
            if GetTextExtentPoint32W(recursos.dc, &raw const codigo, 1, &raw mut extension) == 0
                || TextOutW(
                    recursos.dc,
                    i32::try_from(indice % 16 * celda + 2).map_err(|_| "celda inválida")?,
                    i32::try_from(indice / 16 * celda + 2).map_err(|_| "celda inválida")?,
                    &raw const codigo,
                    1,
                ) == 0
            {
                return Err("no se pudo rasterizar un glifo".to_owned());
            }
        }
        avances.push(extension.ancho as f32);
    }
    // SAFETY: GdiFlush publica las escrituras del DIB antes de leer su memoria;
    // se leen exactamente ancho*alto píxeles inicializados antes de liberar GDI.
    let mut cobertura = unsafe {
        if GdiFlush() == 0 {
            return Err("falló la rasterización tipográfica".to_owned());
        }
        core::slice::from_raw_parts(pixeles.cast::<u8>(), ancho as usize * alto as usize * 4)
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0])
            .collect::<Vec<_>>()
    };
    cobertura[0] = 255;
    Ok(FuenteSistema { ancho, alto, celda, altura: 24.0, primero: 32, avances, cobertura })
}
