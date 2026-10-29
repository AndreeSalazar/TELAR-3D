//! La presentación: la sincronización por fotograma y el ciclo de cada uno
//! (esperar, adquirir, grabar, enviar, presentar). Qué se graba lo decide
//! `dibujo`; la swapchain y su profundidad, `swapchain`.
//!
//! Sincronización, a prueba de las capas de validación:
//! - un semáforo de «imagen adquirida» y una valla **por fotograma en vuelo** (2);
//! - un semáforo de «listo para presentar» **por imagen** de la swapchain: la
//!   presentación lo usa hasta que esa imagen vuelve a adquirirse, así que
//!   reutilizarlo por fotograma provocaría avisos (y carreras de verdad).

use super::contexto::{Contexto, ok};
use super::dibujo::{Destino, Escena};
use super::funciones::{FnDispositivo, FnSwapchain};
use super::swapchain::Swapchain;
use super::tipos::{
    Cola, Comandos, Disp, ETAPA_BOTTOM_OF_PIPE, InfoEnvio, InfoInicioComandos, InfoPresentacion, InfoReservaComandos,
    InfoSimple, LAYOUT_PRESENT_SRC, NIVEL_PRIMARIO, Objeto, POOL_REINICIABLE, ST_COMMAND_BUFFER_ALLOCATE_INFO,
    ST_COMMAND_BUFFER_BEGIN_INFO, ST_COMMAND_POOL_CREATE_INFO, ST_FENCE_CREATE_INFO, ST_PRESENT_INFO_KHR,
    ST_SEMAPHORE_CREATE_INFO, ST_SUBMIT_INFO, USO_UNA_VEZ, VALLA_SENALADA, VK_ERROR_OUT_OF_DATE_KHR, VK_SUBOPTIMAL_KHR,
    VK_SUCCESS,
};
use super::tipos_grafico::ETAPA_SALIDA_COLOR;

/// Fotogramas en vuelo: la CPU prepara uno mientras la GPU pinta el otro.
const EN_VUELO: usize = 2;
const EN_VUELO_U32: u32 = 2;

pub(crate) struct Presentacion {
    fd: FnDispositivo,
    fs: FnSwapchain,
    dispositivo: Disp,
    cola: Cola,
    swapchain: Option<Swapchain>,
    pub(crate) formato: u32,
    pub(crate) extension: (u32, u32),
    adquiridas: [Objeto; EN_VUELO],
    vallas: [Objeto; EN_VUELO],
    pool: Objeto,
    comandos: [Comandos; EN_VUELO],
    cuadro: usize,
    ventana: (u32, u32),
    rehacer: bool,
    /// Cuántas veces se ha creado la swapchain (la primera incluida).
    pub(crate) creaciones: u32,
    reloj: Option<super::reloj_vivo::RelojVivo>,
    host: Option<crate::TelemetriaHost>,
}

impl Presentacion {
    pub(crate) fn medicion_host_activa(&self) -> bool {
        self.reloj.as_ref().is_some_and(|r| r.activa)
    }
    pub(crate) fn observacion_host(&self) -> Option<crate::TelemetriaHost> {
        self.host
    }
    pub(crate) fn preparacion_host(&mut self, ms: Option<f64>) {
        if let Some(host) = &mut self.host {
            host.preparacion_ms = ms;
        }
    }
    pub(crate) fn telemetria(&mut self, ctx: &Contexto, activa: bool, control: Option<u32>) -> Result<(), String> {
        if activa && self.reloj.is_none() {
            self.reloj = Some(super::reloj_vivo::RelojVivo::nuevo(ctx)?);
        }
        if let Some(reloj) = &mut self.reloj {
            reloj.activa = activa;
            reloj.control = control;
        }
        Ok(())
    }
    pub(crate) fn observacion(&self) -> crate::TelemetriaGpu {
        self.reloj.as_ref().map_or_else(crate::TelemetriaGpu::default, |r| r.ultima)
    }
    pub(crate) fn cuadro(&self) -> usize {
        self.cuadro
    }

    pub(crate) fn nueva(ctx: &Contexto, ventana: (u32, u32)) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let fs = fd.swapchain.ok_or("el dispositivo no tiene swapchain (se creó sin ventana)")?;
        let mut p = Self {
            fd,
            fs,
            dispositivo: ctx.dispositivo,
            cola: ctx.cola,
            swapchain: None,
            formato: 0,
            extension: (0, 0),
            adquiridas: [0; EN_VUELO],
            vallas: [0; EN_VUELO],
            pool: 0,
            comandos: [core::ptr::null_mut(); EN_VUELO],
            cuadro: 0,
            ventana,
            rehacer: false,
            creaciones: 0,
            reloj: None,
            host: None,
        };
        let nulo = core::ptr::null();
        let pool = InfoSimple {
            s_type: ST_COMMAND_POOL_CREATE_INFO,
            p_next: nulo,
            flags: POOL_REINICIABLE,
            familia: ctx.familia,
        };
        let semaforo = InfoSimple { s_type: ST_SEMAPHORE_CREATE_INFO, p_next: nulo, flags: 0, familia: 0 };
        let valla = InfoSimple { s_type: ST_FENCE_CREATE_INFO, p_next: nulo, flags: VALLA_SENALADA, familia: 0 };
        let d = p.dispositivo;
        // SAFETY: dispositivo válido; cada `info` vive durante su llamada; lo creado
        // queda en `p` y lo suelta su Drop aunque algo falle a medias.
        unsafe {
            ok((fd.crear_pool)(d, &raw const pool, nulo, &raw mut p.pool), "crear el pool de comandos")?;
            let reserva = InfoReservaComandos {
                s_type: ST_COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: nulo,
                pool: p.pool,
                nivel: NIVEL_PRIMARIO,
                cantidad: EN_VUELO_U32,
            };
            ok((fd.reservar_comandos)(d, &raw const reserva, p.comandos.as_mut_ptr()), "reservar los comandos")?;
            for i in 0..EN_VUELO {
                let r = (fd.crear_semaforo)(d, &raw const semaforo, nulo, &raw mut p.adquiridas[i]);
                ok(r, "crear un semáforo")?;
                ok((fd.crear_valla)(d, &raw const valla, nulo, &raw mut p.vallas[i]), "crear una valla")?;
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

    pub(crate) fn actualizar_muestreo(&mut self, ctx: &Contexto) -> Result<(), String> {
        let Some(sc) = self.swapchain.as_mut() else {
            return Ok(());
        };
        // SAFETY: dispositivo propio; los adjuntos antiguos solo se reemplazan tras completar todos sus usos.
        ok(unsafe { (self.fd.esperar)(self.dispositivo) }, "esperar cambio de calidad")?;
        let nuevos = super::multimuestreo::Adjuntos::nuevos(ctx, sc.formato, sc.extension)?;
        sc.adjuntos = nuevos;
        Ok(())
    }

    /// (Re)crea la swapchain para el tamaño actual de la ventana. Si está
    /// minimizada se queda la que había y se reintenta en el próximo dibujo.
    fn crear_swapchain(&mut self, ctx: &Contexto) -> Result<(), String> {
        // SAFETY: dispositivo válido; tras esperar, la GPU ya no usa la swapchain vieja.
        ok(unsafe { (self.fd.esperar)(self.dispositivo) }, "esperar a la GPU")?;
        let anterior = self.swapchain.as_ref().map_or(0, |s| s.objeto);
        let Some(nueva) = Swapchain::nueva(ctx, self.ventana, anterior)? else {
            self.rehacer = true;
            return Ok(());
        };
        (self.formato, self.extension, self.rehacer) = (nueva.formato, nueva.extension, false);
        self.swapchain = Some(nueva); // la vieja se suelta aquí, ya retirada
        self.creaciones += 1;
        Ok(())
    }

    /// Dibuja el siguiente fotograma: limpia a `limpiar` (ya en el espacio del
    /// formato), dibuja `escena` si hay, y lo presenta. `Ok(false)` si no se
    /// dibujó (ventana minimizada o swapchain rehecha).
    pub(crate) fn dibujar(
        &mut self,
        ctx: &Contexto,
        limpiar: [f32; 4],
        escena: Option<&Escena>,
        interfaz: Option<&mut super::interfaz::InterfazGpu>,
    ) -> Result<bool, String> {
        if self.rehacer {
            self.crear_swapchain(ctx)?;
            if self.rehacer {
                return Ok(false);
            }
        }
        let Some(sc) = self.swapchain.as_ref() else { return Ok(false) };
        let (f, mut indice) = (self.cuadro, 0u32);
        let mut tiempo = super::host_vivo::Cronometro::nuevo(self.medicion_host_activa());
        // SAFETY: la valla y el semáforo son de este fotograma; tras esperar la valla,
        // la GPU ya no usa sus comandos ni su semáforo de adquisición.
        let r = unsafe {
            let r = (self.fd.esperar_vallas)(self.dispositivo, 1, &raw const self.vallas[f], 1, u64::MAX);
            ok(r, "esperar la valla")?;
            tiempo.marcar(0);
            if let Some(reloj) = &mut self.reloj {
                reloj.recoger(f)?;
            }
            tiempo.marcar(1);
            (self.fs.adquirir)(self.dispositivo, sc.objeto, u64::MAX, self.adquiridas[f], 0, &raw mut indice)
        };
        tiempo.marcar(2);
        if r == VK_ERROR_OUT_OF_DATE_KHR {
            self.rehacer = true;
            return Ok(false);
        }
        if r != VK_SUCCESS && r != VK_SUBOPTIMAL_KHR {
            ok(r, "adquirir una imagen")?;
        }
        let interfaz = interfaz.map(|interfaz| interfaz.juego(f)).transpose()?;
        let secundario = preparar_calculo(escena, &self.fd, f)?;
        let i = indice as usize;
        let destino = Destino {
            imagen: sc.imagenes[i],
            vista: sc.vistas[i],
            profundidad: (sc.adjuntos.profundidad.objeto, sc.adjuntos.profundidad.vista),
            multimuestreo: sc.adjuntos.color(),
            extension: sc.extension,
            despues: (LAYOUT_PRESENT_SRC, ETAPA_BOTTOM_OF_PIPE, 0),
        };
        let (cmd, listo, nulo) = (self.comandos[f], sc.listos[i], core::ptr::null());
        let mut comandos = [cmd; 2];
        let n_comandos = if let Some((ranura, parametros)) = escena.and_then(|e| e.calculo) {
            // La valla del juego f ya terminó; ahora se pueden actualizar sus 32 bytes.
            ranura.actualizar(parametros)?;
            comandos = [ranura.comando, cmd];
            2
        } else {
            1
        };
        let inicio = InfoInicioComandos {
            s_type: ST_COMMAND_BUFFER_BEGIN_INFO,
            p_next: nulo,
            flags: USO_UNA_VEZ,
            herencia: nulo,
        };
        let envio = info_envio(&self.adquiridas[f], &listo, &comandos, n_comandos);
        let presentacion = InfoPresentacion {
            s_type: ST_PRESENT_INFO_KHR,
            p_next: nulo,
            n_esperas: 1,
            esperas: &raw const listo,
            n_swapchains: 1,
            swapchains: &raw const sc.objeto,
            indices: &raw const indice,
            resultados: core::ptr::null_mut(),
        };
        // SAFETY: el búfer de comandos es de este fotograma y ya no está en uso (valla
        // esperada); la imagen, su vista y la profundidad son de la swapchain actual,
        // que no se suelta sin esperar a la GPU; todo lo que se pasa por puntero vive
        // hasta el final del bloque, y la valla se reinicia justo antes del envío que
        // la vuelve a señalar.
        let r = unsafe {
            ok((self.fd.reiniciar_comandos)(cmd, 0), "reiniciar los comandos")?;
            ok((self.fd.empezar)(cmd, &raw const inicio), "empezar los comandos")?;
            super::dibujo::grabar_medido(
                &self.fd,
                cmd,
                &destino,
                limpiar,
                escena,
                interfaz.as_ref(),
                self.reloj.as_ref().filter(|r| r.activa).map(|r| (r, f)),
                secundario,
            );
            ok((self.fd.terminar)(cmd), "terminar los comandos")?;
            ok((self.fd.reiniciar_vallas)(self.dispositivo, 1, &raw const self.vallas[f]), "reiniciar la valla")?;
            tiempo.marcar(3);
            ok((self.fd.enviar)(self.cola, 1, &raw const envio, self.vallas[f]), "enviar a la GPU")?;
            tiempo.marcar(4);
            if let Some(reloj) = self.reloj.as_mut().filter(|r| r.activa) {
                let copiado =
                    escena.and_then(|e| e.campos).and_then(|c| reloj.control.and_then(|o| c.control(o))).is_some();
                reloj.enviado(f, copiado);
            }
            (self.fs.presentar)(self.cola, &raw const presentacion)
        };
        tiempo.marcar(5);
        if let Some(host) = tiempo.resultado(self.host.map_or(1, |h| h.cuadros + 1)) {
            self.host = Some(host);
        }
        self.cuadro = (f + 1) % EN_VUELO;
        if r == VK_ERROR_OUT_OF_DATE_KHR || r == VK_SUBOPTIMAL_KHR {
            self.rehacer = true;
        } else {
            ok(r, "presentar")?;
        }
        Ok(true)
    }
}

impl Drop for Presentacion {
    fn drop(&mut self) {
        // SAFETY: tras esperar a la GPU nada está en uso; cada objeto se destruye una
        // vez y solo si se creó (0 = no creado). La swapchain se suelta después, sola.
        unsafe {
            (self.fd.esperar)(self.dispositivo);
            for &s in self.adquiridas.iter().filter(|&&s| s != 0) {
                (self.fd.destruir_semaforo)(self.dispositivo, s, core::ptr::null());
            }
            for &v in self.vallas.iter().filter(|&&v| v != 0) {
                (self.fd.destruir_valla)(self.dispositivo, v, core::ptr::null());
            }
            if self.pool != 0 {
                (self.fd.destruir_pool)(self.dispositivo, self.pool, core::ptr::null());
            }
        }
    }
}

// Los objetos referenciados viven hasta terminar vkQueueSubmit; etapa es una constante promovida.
fn info_envio(espera: &Objeto, listo: &Objeto, comandos: &[Comandos; 2], cantidad: u32) -> InfoEnvio {
    InfoEnvio {
        s_type: ST_SUBMIT_INFO,
        p_next: core::ptr::null(),
        n_esperas: 1,
        esperas: espera,
        etapas_espera: &ETAPA_SALIDA_COLOR,
        n_comandos: cantidad,
        comandos: comandos.as_ptr(),
        n_senales: 1,
        senales: listo,
    }
}

// Solo se llama tras la valla del juego; prepara un secundario con su propia clave y versión.
fn preparar_calculo(escena: Option<&Escena>, fd: &FnDispositivo, juego: usize) -> Result<Option<Comandos>, String> {
    match escena.and_then(|e| e.campos.map(|c| (c, e.constantes))) {
        Some((campos, constantes)) => campos.preparar_cache(fd, juego, constantes),
        None => Ok(None),
    }
}
