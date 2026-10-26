//! La presentación: swapchain, sincronización por fotograma y el primer
//! dibujo (limpiar la imagen a un color).
//!
//! Sincronización, a prueba de las capas de validación:
//! - un semáforo de «imagen adquirida» y una valla **por fotograma en vuelo** (2);
//! - un semáforo de «listo para presentar» **por imagen** de la swapchain: la
//!   presentación lo usa hasta que esa imagen vuelve a adquirirse, así que
//!   reutilizarlo por fotograma provocaría avisos (y carreras de verdad).

use super::contexto::{Contexto, ok};
use super::funciones::{FnDispositivo, FnSwapchain};
use super::tipos::{
    ACCESO_TRANSFER_WRITE, ALFA_OPACO, ASPECTO_COLOR, BarreraImagen, COMPARTIR_EXCLUSIVO, Capacidades, Cola, Comandos,
    Disp, ETAPA_BOTTOM_OF_PIPE, ETAPA_TRANSFER, Extension2D, FAMILIA_IGNORADA, FormatoSuperficie, InfoEnvio,
    InfoInicioComandos, InfoPresentacion, InfoReservaComandos, InfoSimple, InfoSwapchain, LAYOUT_PRESENT_SRC,
    LAYOUT_TRANSFER_DST, LAYOUT_UNDEFINED, NIVEL_PRIMARIO, Objeto, POOL_REINICIABLE, PRESENT_MODE_FIFO, Rango,
    ST_COMMAND_BUFFER_ALLOCATE_INFO, ST_COMMAND_BUFFER_BEGIN_INFO, ST_COMMAND_POOL_CREATE_INFO, ST_FENCE_CREATE_INFO,
    ST_IMAGE_MEMORY_BARRIER, ST_PRESENT_INFO_KHR, ST_SEMAPHORE_CREATE_INFO, ST_SUBMIT_INFO,
    ST_SWAPCHAIN_CREATE_INFO_KHR, USO_COLOR_ATTACHMENT, USO_TRANSFER_DST, USO_UNA_VEZ, VALLA_SENALADA,
    VK_ERROR_OUT_OF_DATE_KHR, VK_SUBOPTIMAL_KHR, VK_SUCCESS,
};
use crate::eleccion::{elegir_extension, elegir_formato, elegir_imagenes};

/// Fotogramas en vuelo: la CPU prepara uno mientras la GPU pinta el otro.
const EN_VUELO: usize = 2;
const EN_VUELO_U32: u32 = 2;

pub(crate) struct Presentacion {
    fd: FnDispositivo,
    fs: FnSwapchain,
    dispositivo: Disp,
    cola: Cola,
    swapchain: Objeto,
    pub(crate) formato: u32,
    pub(crate) extension: (u32, u32),
    imagenes: Vec<Objeto>,
    listos: Vec<Objeto>,
    adquiridas: [Objeto; EN_VUELO],
    vallas: [Objeto; EN_VUELO],
    pool: Objeto,
    comandos: [Comandos; EN_VUELO],
    cuadro: usize,
    ventana: (u32, u32),
    rehacer: bool,
    /// Cuántas veces se ha creado la swapchain (la primera incluida).
    pub(crate) creaciones: u32,
}

impl Presentacion {
    pub(crate) fn nueva(ctx: &Contexto, ventana: (u32, u32)) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let fs = fd.swapchain.ok_or("el dispositivo no tiene swapchain (se creó sin ventana)")?;
        let mut p = Self {
            fd,
            fs,
            dispositivo: ctx.dispositivo,
            cola: ctx.cola,
            swapchain: 0,
            formato: 0,
            extension: (0, 0),
            imagenes: Vec::new(),
            listos: Vec::new(),
            adquiridas: [0; EN_VUELO],
            vallas: [0; EN_VUELO],
            pool: 0,
            comandos: [core::ptr::null_mut(); EN_VUELO],
            cuadro: 0,
            ventana,
            rehacer: false,
            creaciones: 0,
        };
        let pool = InfoSimple {
            s_type: ST_COMMAND_POOL_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: POOL_REINICIABLE,
            familia: ctx.familia,
        };
        let semaforo = InfoSimple { s_type: ST_SEMAPHORE_CREATE_INFO, p_next: core::ptr::null(), flags: 0, familia: 0 };
        let valla =
            InfoSimple { s_type: ST_FENCE_CREATE_INFO, p_next: core::ptr::null(), flags: VALLA_SENALADA, familia: 0 };
        // SAFETY: dispositivo válido; cada `info` vive durante su llamada; lo creado
        // queda en `p` y lo suelta su Drop aunque algo falle a medias.
        unsafe {
            ok(
                (fd.crear_pool)(p.dispositivo, &raw const pool, core::ptr::null(), &raw mut p.pool),
                "crear el pool de comandos",
            )?;
            let reserva = InfoReservaComandos {
                s_type: ST_COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: core::ptr::null(),
                pool: p.pool,
                nivel: NIVEL_PRIMARIO,
                cantidad: EN_VUELO_U32,
            };
            ok(
                (fd.reservar_comandos)(p.dispositivo, &raw const reserva, p.comandos.as_mut_ptr()),
                "reservar los comandos",
            )?;
            for i in 0..EN_VUELO {
                ok(
                    (fd.crear_semaforo)(
                        p.dispositivo,
                        &raw const semaforo,
                        core::ptr::null(),
                        &raw mut p.adquiridas[i],
                    ),
                    "crear un semáforo",
                )?;
                ok(
                    (fd.crear_valla)(p.dispositivo, &raw const valla, core::ptr::null(), &raw mut p.vallas[i]),
                    "crear una valla",
                )?;
            }
        }
        p.crear_swapchain(ctx)?;
        Ok(p)
    }

    /// La ventana cambió de tamaño: la swapchain se rehace antes del próximo dibujo.
    pub(crate) fn redimensionar(&mut self, ventana: (u32, u32)) {
        self.ventana = ventana;
        self.rehacer = true;
    }

    /// (Re)crea la swapchain para el tamaño actual de la ventana.
    fn crear_swapchain(&mut self, ctx: &Contexto) -> Result<(), String> {
        let mut cap = Capacidades::default();
        let mut n = 0u32;
        // SAFETY: físico y superficie válidos; los datos se piden en dos pasos.
        let formatos = unsafe {
            ok((ctx.fi.capacidades)(ctx.fisico, ctx.superficie, &raw mut cap), "leer la superficie")?;
            ok(
                (ctx.fi.formatos)(ctx.fisico, ctx.superficie, &raw mut n, core::ptr::null_mut()),
                "contar los formatos",
            )?;
            let mut v = vec![FormatoSuperficie::default(); n as usize];
            ok((ctx.fi.formatos)(ctx.fisico, ctx.superficie, &raw mut n, v.as_mut_ptr()), "listar los formatos")?;
            v
        };
        let pares: Vec<(u32, u32)> = formatos.iter().map(|f| (f.formato, f.espacio_color)).collect();
        let (formato, espacio) = elegir_formato(&pares).ok_or("la superficie no ofrece ningún formato")?;
        let par = |e: Extension2D| (e.ancho, e.alto);
        let (ancho, alto) = elegir_extension(par(cap.actual), par(cap.minima), par(cap.maxima), self.ventana);
        if ancho == 0 || alto == 0 {
            self.rehacer = true; // minimizada: se rehará al volver
            return Ok(());
        }
        if cap.usos & USO_TRANSFER_DST == 0 {
            return Err("la swapchain no admite escribir en sus imágenes (TRANSFER_DST)".to_owned());
        }
        let anterior = self.swapchain;
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
            uso: USO_TRANSFER_DST | (cap.usos & USO_COLOR_ATTACHMENT),
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
        // SAFETY: se espera a que la GPU termine antes de soltar la swapchain vieja y
        // sus semáforos; `info` vive durante la llamada; las imágenes se piden en dos pasos.
        unsafe {
            ok((self.fd.esperar)(self.dispositivo), "esperar a la GPU")?;
            let mut nueva = 0;
            ok(
                (self.fs.crear)(self.dispositivo, &raw const info, core::ptr::null(), &raw mut nueva),
                "crear la swapchain",
            )?;
            self.swapchain = nueva;
            if anterior != 0 {
                (self.fs.destruir)(self.dispositivo, anterior, core::ptr::null());
            }
            for &s in &self.listos {
                (self.fd.destruir_semaforo)(self.dispositivo, s, core::ptr::null());
            }
            self.listos.clear();
            let mut n = 0u32;
            ok(
                (self.fs.imagenes)(self.dispositivo, self.swapchain, &raw mut n, core::ptr::null_mut()),
                "contar las imágenes",
            )?;
            self.imagenes = vec![0; n as usize];
            ok(
                (self.fs.imagenes)(self.dispositivo, self.swapchain, &raw mut n, self.imagenes.as_mut_ptr()),
                "listar las imágenes",
            )?;
            let semaforo =
                InfoSimple { s_type: ST_SEMAPHORE_CREATE_INFO, p_next: core::ptr::null(), flags: 0, familia: 0 };
            for _ in 0..n {
                let mut s = 0;
                ok(
                    (self.fd.crear_semaforo)(self.dispositivo, &raw const semaforo, core::ptr::null(), &raw mut s),
                    "crear un semáforo",
                )?;
                self.listos.push(s);
            }
        }
        (self.formato, self.extension, self.rehacer) = (formato, (ancho, alto), false);
        self.creaciones += 1;
        Ok(())
    }

    /// Limpia la siguiente imagen a `color` (ya en el espacio del formato) y la
    /// presenta. `Ok(false)` si no se dibujó (ventana minimizada o swapchain rehecha).
    pub(crate) fn dibujar(&mut self, ctx: &Contexto, color: [f32; 4]) -> Result<bool, String> {
        if self.rehacer {
            self.crear_swapchain(ctx)?;
            if self.rehacer || self.swapchain == 0 {
                return Ok(false);
            }
        }
        let f = self.cuadro;
        let mut indice = 0u32;
        // SAFETY: la valla y el semáforo son de este fotograma; tras esperar la valla,
        // la GPU ya no usa sus comandos ni su semáforo de adquisición.
        let r = unsafe {
            ok(
                (self.fd.esperar_vallas)(self.dispositivo, 1, &raw const self.vallas[f], 1, u64::MAX),
                "esperar la valla",
            )?;
            (self.fs.adquirir)(self.dispositivo, self.swapchain, u64::MAX, self.adquiridas[f], 0, &raw mut indice)
        };
        if r == VK_ERROR_OUT_OF_DATE_KHR {
            self.rehacer = true;
            return Ok(false);
        }
        if r != VK_SUCCESS && r != VK_SUBOPTIMAL_KHR {
            ok(r, "adquirir una imagen")?;
        }
        let imagen = self.imagenes[indice as usize];
        let listo = self.listos[indice as usize];
        let cmd = self.comandos[f];
        let rango = Rango { aspecto: ASPECTO_COLOR, mip_base: 0, n_mips: 1, capa_base: 0, n_capas: 1 };
        // De «sin definir» a «destino de copia» (tras esperar al semáforo, en la misma
        // etapa), y de ahí a «presentar».
        let a_limpiar = barrera(imagen, rango, (0, ACCESO_TRANSFER_WRITE), (LAYOUT_UNDEFINED, LAYOUT_TRANSFER_DST));
        let a_presentar = barrera(imagen, rango, (ACCESO_TRANSFER_WRITE, 0), (LAYOUT_TRANSFER_DST, LAYOUT_PRESENT_SRC));
        let inicio = InfoInicioComandos {
            s_type: ST_COMMAND_BUFFER_BEGIN_INFO,
            p_next: core::ptr::null(),
            flags: USO_UNA_VEZ,
            herencia: core::ptr::null(),
        };
        let etapa = ETAPA_TRANSFER;
        let envio = InfoEnvio {
            s_type: ST_SUBMIT_INFO,
            p_next: core::ptr::null(),
            n_esperas: 1,
            esperas: &raw const self.adquiridas[f],
            etapas_espera: &raw const etapa,
            n_comandos: 1,
            comandos: &raw const cmd,
            n_senales: 1,
            senales: &raw const listo,
        };
        let presentacion = InfoPresentacion {
            s_type: ST_PRESENT_INFO_KHR,
            p_next: core::ptr::null(),
            n_esperas: 1,
            esperas: &raw const listo,
            n_swapchains: 1,
            swapchains: &raw const self.swapchain,
            indices: &raw const indice,
            resultados: core::ptr::null_mut(),
        };
        let nulo = core::ptr::null();
        // SAFETY: el búfer de comandos es de este fotograma y ya no está en uso (valla
        // esperada); todo lo que se pasa por puntero vive hasta el final del bloque, y
        // la valla se reinicia justo antes del envío que la vuelve a señalar.
        let r = unsafe {
            ok((self.fd.reiniciar_comandos)(cmd, 0), "reiniciar los comandos")?;
            ok((self.fd.empezar)(cmd, &raw const inicio), "empezar los comandos")?;
            (self.fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_TRANSFER, 0, 0, nulo, 0, nulo, 1, &raw const a_limpiar);
            (self.fd.limpiar_color)(cmd, imagen, LAYOUT_TRANSFER_DST, &raw const color, 1, &raw const rango);
            (self.fd.barrera)(
                cmd,
                ETAPA_TRANSFER,
                ETAPA_BOTTOM_OF_PIPE,
                0,
                0,
                nulo,
                0,
                nulo,
                1,
                &raw const a_presentar,
            );
            ok((self.fd.terminar)(cmd), "terminar los comandos")?;
            ok((self.fd.reiniciar_vallas)(self.dispositivo, 1, &raw const self.vallas[f]), "reiniciar la valla")?;
            ok((self.fd.enviar)(self.cola, 1, &raw const envio, self.vallas[f]), "enviar a la GPU")?;
            (self.fs.presentar)(self.cola, &raw const presentacion)
        };
        self.cuadro = (f + 1) % EN_VUELO;
        if r == VK_ERROR_OUT_OF_DATE_KHR || r == VK_SUBOPTIMAL_KHR {
            self.rehacer = true;
        } else {
            ok(r, "presentar")?;
        }
        Ok(true)
    }
}

/// Una barrera de imagen: `(acceso antes, acceso después)` y `(layout viejo, nuevo)`.
fn barrera(imagen: Objeto, rango: Rango, acceso: (u32, u32), layout: (u32, u32)) -> BarreraImagen {
    BarreraImagen {
        s_type: ST_IMAGE_MEMORY_BARRIER,
        p_next: core::ptr::null(),
        acceso_origen: acceso.0,
        acceso_destino: acceso.1,
        layout_viejo: layout.0,
        layout_nuevo: layout.1,
        familia_origen: FAMILIA_IGNORADA,
        familia_destino: FAMILIA_IGNORADA,
        imagen,
        rango,
    }
}

impl Drop for Presentacion {
    fn drop(&mut self) {
        // SAFETY: tras esperar a la GPU nada está en uso; cada objeto se destruye una
        // vez y solo si se creó (0 = no creado).
        unsafe {
            (self.fd.esperar)(self.dispositivo);
            for &s in self.listos.iter().chain(&self.adquiridas).filter(|&&s| s != 0) {
                (self.fd.destruir_semaforo)(self.dispositivo, s, core::ptr::null());
            }
            for &v in self.vallas.iter().filter(|&&v| v != 0) {
                (self.fd.destruir_valla)(self.dispositivo, v, core::ptr::null());
            }
            if self.pool != 0 {
                (self.fd.destruir_pool)(self.dispositivo, self.pool, core::ptr::null());
            }
            if self.swapchain != 0 {
                (self.fs.destruir)(self.dispositivo, self.swapchain, core::ptr::null());
            }
        }
    }
}
