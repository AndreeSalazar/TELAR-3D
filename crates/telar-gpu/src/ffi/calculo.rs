//! Calcular en la GPU: un kernel SPIR-V sobre un buffer de `u32`.
//!
//! Los datos viven en memoria **de la GPU** (la rápida). Solo al final se copian
//! a un buffer que la CPU puede leer, y los temporizadores de la GPU miden **solo
//! el kernel**: ni la copia por el bus (`PCIe`) ni el envío cuentan.
//!
//! ```text
//! [tiempo 0] despachar kernel [tiempo 1] ─barrera─► copiar a lectura ─barrera─► CPU lee
//! ```

use core::ffi::c_void;

use super::contexto::{Contexto, ok};
use super::recursos::Recursos;
use super::tipos::{
    ACCESO_TRANSFER_WRITE, ETAPA_TRANSFER, InfoEnvio, InfoInicioComandos, ST_COMMAND_BUFFER_BEGIN_INFO, ST_SUBMIT_INFO,
    USO_UNA_VEZ,
};
use super::tipos_calculo::{
    ACCESO_HOST_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_ALMACEN, BUFFER_TRANSFER_DST,
    BUFFER_TRANSFER_SRC, BarreraMemoria, Copia, ETAPA_CALCULO, ETAPA_HOST, ETAPA_TOP_OF_PIPE, PUNTO_CALCULO,
    RESULTADO_64, RESULTADO_ESPERAR, ST_MEMORY_BARRIER,
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_CON_CACHE, MEMORIA_GPU, MEMORIA_VISIBLE, grupos, nanosegundos};

/// Lo que devuelve una ejecución.
pub(crate) struct Ejecucion {
    pub(crate) datos: Vec<u32>,
    /// Lo que tardó el kernel según la GPU (sin temporizador en la cola, `None`).
    pub(crate) nanosegundos: Option<f64>,
}

/// Diez segundos: si un kernel tarda más, algo va mal.
const ESPERA_MAXIMA_NS: u64 = 10_000_000_000;

/// Ejecuta `spirv` (entrada `main`, buffer en el conjunto 0 y enlace 0) sobre
/// `elementos` valores, en grupos de `grupo` hilos.
pub(crate) fn ejecutar(ctx: &Contexto, spirv: &[u32], elementos: u32, grupo: u32) -> Result<Ejecucion, String> {
    let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
    let n_grupos = grupos(elementos, grupo);
    if n_grupos > ctx.grupos_max_x {
        return Err(format!("{n_grupos} grupos: la GPU admite {} en X; usa grupos más grandes", ctx.grupos_max_x));
    }
    let tamano = u64::from(elementos.max(1)) * 4;
    let mut r = Recursos::nuevo(&fd, ctx.dispositivo);
    r.buffer(ctx, 0, tamano, BUFFER_ALMACEN | BUFFER_TRANSFER_SRC, (MEMORIA_GPU, 0))?;
    r.buffer(ctx, 1, tamano, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, MEMORIA_CON_CACHE))?;
    r.pipeline(spirv)?;
    r.descriptores(tamano)?;
    let medir = ctx.bits_tiempo > 0;
    r.grabar_y_enviar(ctx, n_grupos, tamano, medir)?;
    let nanosegundos = if medir { Some(r.tiempo(ctx)?) } else { None };
    Ok(Ejecucion { datos: r.leer(tamano, elementos)?, nanosegundos })
}

impl Recursos {
    /// Graba los comandos, los envía y espera a que la GPU termine.
    fn grabar_y_enviar(&mut self, ctx: &Contexto, n_grupos: u32, tamano: u64, medir: bool) -> Result<(), String> {
        let cmd = self.preparar(ctx, medir)?;
        let (fd, fc, nulo) = (self.fd, self.fd.calculo, core::ptr::null());
        let inicio = InfoInicioComandos {
            s_type: ST_COMMAND_BUFFER_BEGIN_INFO,
            p_next: nulo,
            flags: USO_UNA_VEZ,
            herencia: nulo,
        };
        let a_copia = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: nulo,
            acceso_origen: ACCESO_SHADER_WRITE,
            acceso_destino: ACCESO_TRANSFER_READ,
        };
        let a_cpu = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: nulo,
            acceso_origen: ACCESO_TRANSFER_WRITE,
            acceso_destino: ACCESO_HOST_READ,
        };
        let copia = Copia { origen: 0, destino: 0, tamano };
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
        let barrera = |origen: u32, destino: u32, b: &BarreraMemoria| {
            // SAFETY: `cmd` está grabándose; la barrera de memoria vive durante la llamada.
            unsafe {
                (fd.barrera)(
                    cmd,
                    origen,
                    destino,
                    0,
                    1,
                    core::ptr::from_ref(b).cast::<c_void>(),
                    0,
                    nulo,
                    0,
                    nulo.cast(),
                );
            }
        };
        // SAFETY: `cmd` es nuestro y nuevo; pipeline, conjunto, buffers y consultas
        // existen; las barreras ordenan kernel → copia → CPU; se espera la valla
        // antes de devolver, así que nada de lo que se pasa por puntero muere antes.
        unsafe {
            ok((fd.empezar)(cmd, &raw const inicio), "empezar los comandos")?;
            if medir {
                (fc.reiniciar_consultas)(cmd, self.consultas, 0, 2);
                (fc.marcar_tiempo)(cmd, ETAPA_TOP_OF_PIPE, self.consultas, 0);
            }
            (fc.usar_pipeline)(cmd, PUNTO_CALCULO, self.pipeline);
            (fc.usar_conjuntos)(
                cmd,
                PUNTO_CALCULO,
                self.disposicion_pipeline,
                0,
                1,
                &raw const self.conjunto,
                0,
                nulo.cast(),
            );
            (fc.despachar)(cmd, n_grupos, 1, 1);
            if medir {
                (fc.marcar_tiempo)(cmd, ETAPA_CALCULO, self.consultas, 1);
            }
        }
        barrera(ETAPA_CALCULO, ETAPA_TRANSFER, &a_copia);
        // SAFETY: como el bloque anterior; los dos buffers miden `tamano`.
        unsafe { (fc.copiar_buffer)(cmd, self.buffers[0], self.buffers[1], 1, &raw const copia) };
        barrera(ETAPA_TRANSFER, ETAPA_HOST, &a_cpu);
        // SAFETY: la grabación termina aquí; el envío usa la valla nuestra, y se
        // espera a que la GPU acabe (con un límite) antes de seguir.
        unsafe {
            ok((fd.terminar)(cmd), "terminar los comandos")?;
            ok((fd.enviar)(ctx.cola, 1, &raw const envio, self.valla), "enviar a la GPU")?;
            ok((fd.esperar_vallas)(self.d, 1, &raw const self.valla, 1, ESPERA_MAXIMA_NS), "esperar a la GPU")
        }
    }

    /// Nanosegundos entre los dos tiempos.
    fn tiempo(&self, ctx: &Contexto) -> Result<f64, String> {
        let mut tics = [0u64; 2];
        // SAFETY: la GPU ya terminó (valla esperada); `tics` mide 16 bytes, 2 consultas de 8.
        let r = unsafe {
            (self.fd.calculo.leer_consultas)(
                self.d,
                self.consultas,
                0,
                2,
                16,
                tics.as_mut_ptr().cast::<c_void>(),
                8,
                RESULTADO_64 | RESULTADO_ESPERAR,
            )
        };
        ok(r, "leer los tiempos")?;
        Ok(nanosegundos(tics[0], tics[1], ctx.bits_tiempo, ctx.periodo_ns))
    }

    /// Copia los `elementos` del buffer de lectura a un `Vec`.
    pub(super) fn leer(&self, tamano: u64, elementos: u32) -> Result<Vec<u32>, String> {
        let fc = self.fd.calculo;
        let mut puntero: *mut c_void = core::ptr::null_mut();
        // SAFETY: la memoria es visible y coherente para la CPU, la GPU ya terminó, y
        // se leen `elementos` u32 de un mapeo de `tamano` ≥ 4 × elementos bytes
        // (alineado a minMemoryMapAlignment ≥ 64); se desmapea antes de devolver.
        unsafe {
            ok((fc.mapear)(self.d, self.memorias[1], 0, tamano, 0, &raw mut puntero), "mapear la memoria")?;
            let datos = core::slice::from_raw_parts(puntero.cast::<u32>(), elementos as usize).to_vec();
            (fc.desmapear)(self.d, self.memorias[1]);
            Ok(datos)
        }
    }
}
