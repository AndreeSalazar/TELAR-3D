//! Imágenes de la GPU con su memoria y su vista: la de profundidad del visor y
//! la de color de las capturas. Y las vistas sueltas, para las imágenes de la
//! swapchain (que son del sistema).

use super::contexto::{Contexto, ok};
use super::funciones::FnDispositivo;
use super::tipos::{COMPARTIR_EXCLUSIVO, Disp, LAYOUT_UNDEFINED, Objeto, Rango};
use super::tipos_calculo::{InfoMemoria, Requisitos, ST_MEMORY_ALLOCATE_INFO};
use super::tipos_grafico::{
    Extension3D, IMAGEN_2D, InfoImagen, InfoVista, MUESTRAS_1, ST_IMAGE_CREATE_INFO, ST_IMAGE_VIEW_CREATE_INFO,
    TESELADO_OPTIMO, VISTA_2D,
};
use crate::eleccion::{MEMORIA_GPU, elegir_memoria};

/// Una vista 2D de `imagen`, de un solo nivel y una sola capa, con el `aspecto` dado.
pub(crate) fn crear_vista(
    fd: &FnDispositivo,
    d: Disp,
    imagen: Objeto,
    formato: u32,
    aspecto: u32,
) -> Result<Objeto, String> {
    let info = InfoVista {
        s_type: ST_IMAGE_VIEW_CREATE_INFO,
        p_next: core::ptr::null(),
        flags: 0,
        imagen,
        tipo: VISTA_2D,
        formato,
        componentes: [0; 4], // identidad
        rango: Rango { aspecto, mip_base: 0, n_mips: 1, capa_base: 0, n_capas: 1 },
    };
    let mut vista = 0;
    // SAFETY: la imagen existe y `info` vive durante la llamada.
    let r = unsafe { (fd.grafico.crear_vista)(d, &raw const info, core::ptr::null(), &raw mut vista) };
    ok(r, "crear una vista de imagen")?;
    Ok(vista)
}

/// Destruye una vista (si existe).
pub(crate) fn destruir_vista(fd: &FnDispositivo, d: Disp, vista: Objeto) {
    if vista != 0 {
        // SAFETY: la GPU ya no usa la vista (quien llama esperó a que acabara).
        unsafe { (fd.grafico.destruir_vista)(d, vista, core::ptr::null()) };
    }
}

/// Una imagen 2D en memoria de la GPU, con su vista. Se suelta en Drop.
pub(crate) struct Imagen {
    fd: FnDispositivo,
    d: Disp,
    pub(crate) objeto: Objeto,
    memoria: Objeto,
    pub(crate) vista: Objeto,
}

impl Imagen {
    pub(crate) fn nueva(
        ctx: &Contexto,
        formato: u32,
        (ancho, alto): (u32, u32),
        uso: u32,
        aspecto: u32,
    ) -> Result<Self, String> {
        Self::con_muestras(ctx, formato, (ancho, alto), uso, aspecto, MUESTRAS_1)
    }

    pub(crate) fn con_muestras(
        ctx: &Contexto,
        formato: u32,
        (ancho, alto): (u32, u32),
        uso: u32,
        aspecto: u32,
        muestras: u32,
    ) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let mut i = Self { fd, d: ctx.dispositivo, objeto: 0, memoria: 0, vista: 0 };
        let info = InfoImagen {
            s_type: ST_IMAGE_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            tipo: IMAGEN_2D,
            formato,
            extension: Extension3D { ancho, alto, fondo: 1 },
            mips: 1,
            capas: 1,
            muestras,
            teselado: TESELADO_OPTIMO,
            uso,
            compartir: COMPARTIR_EXCLUSIVO,
            n_familias: 0,
            familias: core::ptr::null(),
            layout_inicial: LAYOUT_UNDEFINED,
        };
        let (fg, fc) = (fd.grafico, fd.calculo);
        let mut req = Requisitos::default();
        // SAFETY: dispositivo válido; `info` vive durante la llamada; la imagen queda en `i`.
        unsafe {
            ok((fg.crear_imagen)(i.d, &raw const info, core::ptr::null(), &raw mut i.objeto), "crear una imagen")?;
            (fg.requisitos_imagen)(i.d, i.objeto, &raw mut req);
        }
        let tipo = elegir_memoria(&ctx.tipos_memoria, req.tipos_permitidos, MEMORIA_GPU, 0)
            .ok_or("la GPU no tiene memoria propia para la imagen")?;
        let reserva =
            InfoMemoria { s_type: ST_MEMORY_ALLOCATE_INFO, p_next: core::ptr::null(), tamano: req.tamano, tipo };
        // SAFETY: el tipo de memoria es de los permitidos para la imagen; queda en `i`.
        unsafe {
            let r = (fc.reservar_memoria)(i.d, &raw const reserva, core::ptr::null(), &raw mut i.memoria);
            ok(r, "reservar memoria para la imagen")?;
            ok((fg.unir_memoria_imagen)(i.d, i.objeto, i.memoria, 0), "unir la memoria a la imagen")?;
        }
        i.vista = crear_vista(&fd, i.d, i.objeto, formato, aspecto)?;
        Ok(i)
    }
}

impl Drop for Imagen {
    fn drop(&mut self) {
        destruir_vista(&self.fd, self.d, self.vista);
        // SAFETY: la GPU ya no la usa (quien la suelta esperó a que acabara); se destruye
        // una vez, la imagen antes que su memoria.
        unsafe {
            if self.objeto != 0 {
                (self.fd.grafico.destruir_imagen)(self.d, self.objeto, core::ptr::null());
            }
            if self.memoria != 0 {
                (self.fd.calculo.liberar_memoria)(self.d, self.memoria, core::ptr::null());
            }
        }
    }
}
