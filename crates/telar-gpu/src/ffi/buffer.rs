//! Buffers con su memoria, y los envíos «de una vez» (grabar unos comandos,
//! enviarlos y esperar): para subir mallas a la GPU o leer una captura.

use core::ffi::c_void;

use super::contexto::{Contexto, ok};
use super::funciones::FnDispositivo;
use super::tipos::{
    COMPARTIR_EXCLUSIVO, Comandos, Disp, InfoEnvio, InfoInicioComandos, InfoReservaComandos, InfoSimple,
    NIVEL_PRIMARIO, Objeto, ST_COMMAND_BUFFER_ALLOCATE_INFO, ST_COMMAND_BUFFER_BEGIN_INFO, ST_COMMAND_POOL_CREATE_INFO,
    ST_FENCE_CREATE_INFO, ST_SUBMIT_INFO, USO_UNA_VEZ,
};
use super::tipos_calculo::{InfoBuffer, InfoMemoria, Requisitos, ST_BUFFER_CREATE_INFO, ST_MEMORY_ALLOCATE_INFO};
use crate::eleccion::elegir_memoria;

/// Los bytes de un slice de números (`f32`, `u32`, `[f32; 6]`…).
pub(crate) fn como_bytes<T: Copy>(datos: &[T]) -> &[u8] {
    // SAFETY: T es un número o un array de números (Copy, sin relleno ni punteros):
    // sus bytes son memoria inicializada que vive lo mismo que `datos`.
    unsafe { core::slice::from_raw_parts(datos.as_ptr().cast::<u8>(), core::mem::size_of_val(datos)) }
}

/// Un buffer con su memoria; se suelta en Drop.
pub(crate) struct Buffer {
    fd: FnDispositivo,
    d: Disp,
    pub(crate) objeto: Objeto,
    memoria: Objeto,
    pub(crate) tamano: u64,
}

impl Buffer {
    /// `memoria`: `(necesarias, preferidas)`, las propiedades del tipo de memoria.
    pub(crate) fn nuevo(ctx: &Contexto, tamano: u64, uso: u32, memoria: (u32, u32)) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let mut b = Self { fd, d: ctx.dispositivo, objeto: 0, memoria: 0, tamano: tamano.max(4) };
        let info = InfoBuffer {
            s_type: ST_BUFFER_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            tamano: b.tamano,
            uso,
            compartir: COMPARTIR_EXCLUSIVO,
            n_familias: 0,
            familias: core::ptr::null(),
        };
        let mut req = Requisitos::default();
        let fc = fd.calculo;
        // SAFETY: dispositivo válido; `info` vive durante la llamada; el buffer queda en `b`.
        unsafe {
            ok((fc.crear_buffer)(b.d, &raw const info, core::ptr::null(), &raw mut b.objeto), "crear un buffer")?;
            (fc.requisitos)(b.d, b.objeto, &raw mut req);
        }
        let tipo = elegir_memoria(&ctx.tipos_memoria, req.tipos_permitidos, memoria.0, memoria.1)
            .ok_or("la GPU no tiene memoria del tipo que necesita el buffer")?;
        let reserva =
            InfoMemoria { s_type: ST_MEMORY_ALLOCATE_INFO, p_next: core::ptr::null(), tamano: req.tamano, tipo };
        // SAFETY: el tipo de memoria es de los permitidos para el buffer; la memoria queda en `b`.
        unsafe {
            let r = (fc.reservar_memoria)(b.d, &raw const reserva, core::ptr::null(), &raw mut b.memoria);
            ok(r, "reservar memoria")?;
            ok((fc.unir_memoria)(b.d, b.objeto, b.memoria, 0), "unir la memoria")?;
        }
        Ok(b)
    }

    /// Copia `datos` en el buffer a partir del byte `desde` (su memoria tiene que
    /// ser visible para la CPU).
    pub(crate) fn escribir(&self, desde: usize, datos: &[u8]) -> Result<(), String> {
        if (desde + datos.len()) as u64 > self.tamano {
            return Err(format!("{} bytes no caben en un buffer de {} desde el {desde}", datos.len(), self.tamano));
        }
        let fc = self.fd.calculo;
        let mut puntero: *mut c_void = core::ptr::null_mut();
        // SAFETY: la memoria es visible y coherente, nadie más la usa ahora, y se
        // copian `datos.len()` bytes en `desde..desde + datos.len()` ⊆ `0..tamano`
        // del mapeo; se desmapea al terminar.
        unsafe {
            ok((fc.mapear)(self.d, self.memoria, 0, self.tamano, 0, &raw mut puntero), "mapear la memoria")?;
            core::ptr::copy_nonoverlapping(datos.as_ptr(), puntero.cast::<u8>().add(desde), datos.len());
            (fc.desmapear)(self.d, self.memoria);
        }
        Ok(())
    }

    /// Los primeros `n` bytes del buffer (su memoria tiene que ser visible para la CPU).
    pub(crate) fn leer(&self, n: usize) -> Result<Vec<u8>, String> {
        let n = n.min(usize::try_from(self.tamano).unwrap_or(usize::MAX));
        let fc = self.fd.calculo;
        let mut puntero: *mut c_void = core::ptr::null_mut();
        // SAFETY: la GPU ya terminó (lo garantiza quien llama, tras esperar su valla);
        // se leen `n` ≤ `tamano` bytes del mapeo y se desmapea antes de devolver.
        unsafe {
            ok((fc.mapear)(self.d, self.memoria, 0, self.tamano, 0, &raw mut puntero), "mapear la memoria")?;
            let datos = core::slice::from_raw_parts(puntero.cast::<u8>(), n).to_vec();
            (fc.desmapear)(self.d, self.memoria);
            Ok(datos)
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let fc = self.fd.calculo;
        // SAFETY: quien lo creó ya no lo usa en la GPU (cada uso espera su valla); se
        // destruye una vez y solo si existe.
        unsafe {
            if self.objeto != 0 {
                (fc.destruir_buffer)(self.d, self.objeto, core::ptr::null());
            }
            if self.memoria != 0 {
                (fc.liberar_memoria)(self.d, self.memoria, core::ptr::null());
            }
        }
    }
}

/// Graba unos comandos con `grabar`, los envía y espera a que la GPU acabe.
pub(crate) fn una_vez(ctx: &Contexto, grabar: impl FnOnce(&FnDispositivo, Comandos)) -> Result<(), String> {
    let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
    let (d, nulo) = (ctx.dispositivo, core::ptr::null());
    let pool = InfoSimple { s_type: ST_COMMAND_POOL_CREATE_INFO, p_next: nulo, flags: 0, familia: ctx.familia };
    let valla = InfoSimple { s_type: ST_FENCE_CREATE_INFO, p_next: nulo, flags: 0, familia: 0 };
    let (mut p, mut v, mut cmd): (Objeto, Objeto, Comandos) = (0, 0, core::ptr::null_mut());
    let inicio =
        InfoInicioComandos { s_type: ST_COMMAND_BUFFER_BEGIN_INFO, p_next: nulo, flags: USO_UNA_VEZ, herencia: nulo };
    // SAFETY: dispositivo válido; cada `info` vive durante su llamada; el pool y la
    // valla se destruyen al final pase lo que pase (el búfer de comandos va con el pool).
    unsafe {
        let r = (|| {
            ok((fd.crear_pool)(d, &raw const pool, nulo, &raw mut p), "crear el pool de comandos")?;
            ok((fd.crear_valla)(d, &raw const valla, nulo, &raw mut v), "crear la valla")?;
            let reserva = InfoReservaComandos {
                s_type: ST_COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: nulo,
                pool: p,
                nivel: NIVEL_PRIMARIO,
                cantidad: 1,
            };
            ok((fd.reservar_comandos)(d, &raw const reserva, &raw mut cmd), "reservar los comandos")?;
            ok((fd.empezar)(cmd, &raw const inicio), "empezar los comandos")?;
            grabar(&fd, cmd);
            ok((fd.terminar)(cmd), "terminar los comandos")?;
            let envio = InfoEnvio {
                s_type: ST_SUBMIT_INFO,
                p_next: nulo,
                n_esperas: 0,
                esperas: nulo.cast(),
                etapas_espera: nulo.cast(),
                n_comandos: 1,
                comandos: &raw const cmd,
                n_senales: 0,
                senales: nulo.cast(),
            };
            ok((fd.enviar)(ctx.cola, 1, &raw const envio, v), "enviar a la GPU")?;
            ok((fd.esperar_vallas)(d, 1, &raw const v, 1, 10_000_000_000), "esperar a la GPU")
        })();
        if v != 0 {
            (fd.destruir_valla)(d, v, nulo);
        }
        if p != 0 {
            (fd.destruir_pool)(d, p, nulo);
        }
        r
    }
}
