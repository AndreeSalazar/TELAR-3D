//! La swapchain y lo que va con ella: sus imágenes (son del sistema), una vista
//! y un semáforo «listo para presentar» por imagen, y la imagen de profundidad
//! del mismo tamaño. Se rehace entera cuando cambia el tamaño de la ventana.

use super::contexto::{Contexto, ok};
use super::funciones::{FnDispositivo, FnSwapchain};
use super::imagen::{crear_vista, destruir_vista};
use super::multimuestreo::Adjuntos;
use super::tipos::{
    ALFA_OPACO, ASPECTO_COLOR, COMPARTIR_EXCLUSIVO, Capacidades, Disp, Extension2D, FormatoSuperficie, InfoSimple,
    InfoSwapchain, Objeto, PRESENT_MODE_FIFO, ST_SEMAPHORE_CREATE_INFO, ST_SWAPCHAIN_CREATE_INFO_KHR,
    USO_COLOR_ATTACHMENT,
};
use crate::eleccion::{elegir_extension, elegir_formato, elegir_imagenes};

pub(crate) struct Swapchain {
    fd: FnDispositivo,
    fs: FnSwapchain,
    d: Disp,
    pub(crate) objeto: Objeto,
    pub(crate) formato: u32,
    pub(crate) extension: (u32, u32),
    pub(crate) imagenes: Vec<Objeto>,
    pub(crate) vistas: Vec<Objeto>,
    /// Uno por imagen: la presentación lo usa hasta que esa imagen vuelve a adquirirse.
    pub(crate) listos: Vec<Objeto>,
    pub(crate) adjuntos: Adjuntos,
}

impl Swapchain {
    /// Una swapchain para el tamaño de `ventana`, que sustituye a `anterior` (0 si
    /// no hay). `Ok(None)` si la ventana está minimizada. Quien llama tiene que
    /// haber esperado a que la GPU acabe con la anterior.
    pub(crate) fn nueva(ctx: &Contexto, ventana: (u32, u32), anterior: Objeto) -> Result<Option<Self>, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let fs = fd.swapchain.ok_or("el dispositivo no tiene swapchain (se creó sin ventana)")?;
        let mut cap = Capacidades::default();
        let mut n = 0u32;
        // SAFETY: físico y superficie válidos; los datos se piden en dos pasos.
        let formatos = unsafe {
            ok((ctx.fi.capacidades)(ctx.fisico, ctx.superficie, &raw mut cap), "leer la superficie")?;
            let r = (ctx.fi.formatos)(ctx.fisico, ctx.superficie, &raw mut n, core::ptr::null_mut());
            ok(r, "contar los formatos")?;
            let mut v = vec![FormatoSuperficie::default(); n as usize];
            ok((ctx.fi.formatos)(ctx.fisico, ctx.superficie, &raw mut n, v.as_mut_ptr()), "listar los formatos")?;
            v
        };
        let pares: Vec<(u32, u32)> = formatos.iter().map(|f| (f.formato, f.espacio_color)).collect();
        let (formato, espacio) = elegir_formato(&pares).ok_or("la superficie no ofrece ningún formato")?;
        let par = |e: Extension2D| (e.ancho, e.alto);
        let (ancho, alto) = elegir_extension(par(cap.actual), par(cap.minima), par(cap.maxima), ventana);
        if ancho == 0 || alto == 0 {
            return Ok(None); // minimizada: se rehará al volver
        }
        if cap.usos & USO_COLOR_ATTACHMENT == 0 {
            return Err("la swapchain no admite dibujar en sus imágenes (COLOR_ATTACHMENT)".to_owned());
        }
        let adjuntos = Adjuntos::nuevos(ctx, formato, (ancho, alto))?;
        let info = InfoSwapchain {
            s_type: ST_SWAPCHAIN_CREATE_INFO_KHR,
            p_next: core::ptr::null(),
            flags: 0,
            superficie: ctx.superficie,
            min_imagenes: elegir_imagenes(cap.min_imagenes, cap.max_imagenes),
            formato,
            espacio_color: espacio,
            extension: Extension2D { ancho, alto },
            capas: 1,
            uso: USO_COLOR_ATTACHMENT,
            compartir: COMPARTIR_EXCLUSIVO,
            n_familias: 0,
            familias: core::ptr::null(),
            transformacion: cap.transformacion_actual,
            alfa: if cap.alfa_compuesto & ALFA_OPACO != 0 {
                ALFA_OPACO
            } else {
                cap.alfa_compuesto & cap.alfa_compuesto.wrapping_neg()
            },
            modo_presentacion: PRESENT_MODE_FIFO, // sincronizado con el monitor; siempre disponible
            recortada: 1,
            anterior,
        };
        let mut objeto = 0;
        // SAFETY: dispositivo y superficie válidos; `info` vive durante la llamada.
        let r = unsafe { (fs.crear)(ctx.dispositivo, &raw const info, core::ptr::null(), &raw mut objeto) };
        ok(r, "crear la swapchain")?;
        let mut s = Self {
            fd,
            fs,
            d: ctx.dispositivo,
            objeto,
            formato,
            extension: (ancho, alto),
            imagenes: Vec::new(),
            vistas: Vec::new(),
            listos: Vec::new(),
            adjuntos,
        };
        // SAFETY: la swapchain es nueva; las imágenes se piden en dos pasos.
        unsafe {
            let r = (fs.imagenes)(s.d, s.objeto, &raw mut n, core::ptr::null_mut());
            ok(r, "contar las imágenes")?;
            s.imagenes = vec![0; n as usize];
            ok((fs.imagenes)(s.d, s.objeto, &raw mut n, s.imagenes.as_mut_ptr()), "listar las imágenes")?;
        }
        let semaforo = InfoSimple { s_type: ST_SEMAPHORE_CREATE_INFO, p_next: core::ptr::null(), flags: 0, familia: 0 };
        for i in 0..s.imagenes.len() {
            s.vistas.push(crear_vista(&fd, s.d, s.imagenes[i], formato, ASPECTO_COLOR)?);
            let mut listo = 0;
            // SAFETY: dispositivo válido; `semaforo` vive durante la llamada.
            let r = unsafe { (fd.crear_semaforo)(s.d, &raw const semaforo, core::ptr::null(), &raw mut listo) };
            ok(r, "crear un semáforo")?;
            s.listos.push(listo);
        }
        Ok(Some(s))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        for &v in &self.vistas {
            destruir_vista(&self.fd, self.d, v);
        }
        // SAFETY: quien la suelta esperó a que la GPU acabara; cada objeto se destruye
        // una vez (las imágenes son de la swapchain y se van con ella).
        unsafe {
            for &s in &self.listos {
                (self.fd.destruir_semaforo)(self.d, s, core::ptr::null());
            }
            (self.fs.destruir)(self.d, self.objeto, core::ptr::null());
        }
    }
}
