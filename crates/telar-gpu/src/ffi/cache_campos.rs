//! Dos secundarios compute reutilizables; cada juego se modifica solo tras su valla.
use super::{
    contexto::{Contexto, ok},
    funciones::FnDispositivo,
    tipos::{
        Comandos, Disp, InfoInicioComandos, InfoReservaComandos, InfoSimple, Objeto, POOL_REINICIABLE,
        ST_COMMAND_BUFFER_ALLOCATE_INFO, ST_COMMAND_BUFFER_BEGIN_INFO, ST_COMMAND_POOL_CREATE_INFO,
    },
};
use core::ffi::c_void;
/// `VkCommandBufferInheritanceInfo`, sin pase de render ni queries heredadas.
#[repr(C)]
struct Herencia {
    s_type: u32,
    p_next: *const c_void,
    render_pass: Objeto,
    subpass: u32,
    framebuffer: Objeto,
    occlusion: u32,
    query_flags: u32,
    estadisticas: u32,
}
pub(super) struct Cache {
    fd: FnDispositivo,
    d: Disp,
    pool: Objeto,
    comandos: [Comandos; 2],
    claves: [Option<([u32; 32], u64)>; 2],
    pub(super) activa: bool,
    pub(super) mascara: u32,
    pub(super) grabaciones: u64,
    pub(super) reusos: u64,
    pub(super) directas: u64,
}
impl Cache {
    pub(super) fn nueva(ctx: &Contexto) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("cache compute sin dispositivo")?;
        let mut cache = Self {
            fd,
            d: ctx.dispositivo,
            pool: 0,
            comandos: [core::ptr::null_mut(); 2],
            claves: [None; 2],
            activa: true,
            mascara: u32::MAX,
            grabaciones: 0,
            reusos: 0,
            directas: 0,
        };
        let info = InfoSimple {
            s_type: ST_COMMAND_POOL_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: POOL_REINICIABLE,
            familia: ctx.familia,
        };
        // SAFETY: dispositivo/familia válidos; pool propio liberado por Drop incluso ante error parcial.
        let resultado = unsafe { (fd.crear_pool)(cache.d, &raw const info, core::ptr::null(), &raw mut cache.pool) };
        ok(resultado, "crear pool de cache compute")?;
        let reserva = InfoReservaComandos {
            s_type: ST_COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: core::ptr::null(),
            pool: cache.pool,
            nivel: 1,
            cantidad: 2,
        };
        // SAFETY: reserva dos secundarios; capacidad del array y pool coinciden con la solicitud.
        let resultado = unsafe { (fd.reservar_comandos)(cache.d, &raw const reserva, cache.comandos.as_mut_ptr()) };
        ok(resultado, "reservar secundarios compute")?;
        Ok(cache)
    }
    pub(super) fn configurar(&mut self, activa: bool, mascara: u32) {
        if self.mascara != mascara {
            self.claves = [None; 2];
        }
        self.activa = activa;
        self.mascara = mascara;
    }
    /// El presentador ha completado la valla del juego; el callback solo graba compute.
    pub(super) fn preparar(
        &mut self,
        juego: usize,
        version: u64,
        bloque: &[f32; 32],
        grabar: impl FnOnce(Comandos, &[f32; 32]),
    ) -> Result<Option<Comandos>, String> {
        if !self.activa {
            self.directas += 1;
            return Ok(None);
        }
        let clave = core::array::from_fn(|i| if self.mascara & (1u32 << i) != 0 { bloque[i].to_bits() } else { 0 });
        if self.claves[juego] == Some((clave, version)) {
            self.reusos += 1;
            return Ok(Some(self.comandos[juego]));
        }
        let cmd = self.comandos[juego];
        self.claves[juego] = None;
        let herencia = Herencia {
            s_type: 41,
            p_next: core::ptr::null(),
            render_pass: 0,
            subpass: 0,
            framebuffer: 0,
            occlusion: 0,
            query_flags: 0,
            estadisticas: 0,
        };
        // flags=0: reutilizable, sin SIMULTANEOUS_USE; un secundario por juego/valla.
        let inicio = InfoInicioComandos {
            s_type: ST_COMMAND_BUFFER_BEGIN_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            herencia: (&raw const herencia).cast(),
        };
        // SAFETY: la valla del juego terminó; ningún otro juego referencia este secundario.
        unsafe {
            ok((self.fd.reiniciar_comandos)(cmd, 0), "reiniciar secundario compute")?;
            ok((self.fd.empezar)(cmd, &raw const inicio), "empezar secundario compute")?;
        }
        let constantes = clave.map(f32::from_bits);
        grabar(cmd, &constantes);
        // SAFETY: callback terminó de grabar fuera de rendering; recursos siguen vivos.
        ok(unsafe { (self.fd.terminar)(cmd) }, "terminar secundario compute")?;
        self.claves[juego] = Some((clave, version));
        self.grabaciones += 1;
        Ok(Some(cmd))
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        // SAFETY: completa usos antes de liberar el pool y sus dos secundarios.
        unsafe {
            (self.fd.esperar)(self.d);
            if self.pool != 0 {
                (self.fd.destruir_pool)(self.d, self.pool, core::ptr::null());
            }
        }
    }
}
