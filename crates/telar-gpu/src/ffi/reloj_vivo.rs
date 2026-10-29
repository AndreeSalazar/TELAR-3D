//! Dos juegos de consultas y snapshots; se recogen después de las vallas ya existentes.
use super::{
    buffer::{Buffer, BufferMapeado},
    contexto::{Contexto, ok},
    funciones::FnDispositivo,
    tipos::{ACCESO_TRANSFER_WRITE, ETAPA_BOTTOM_OF_PIPE, ETAPA_TRANSFER},
    tipos::{Comandos, Disp, Objeto},
    tipos_calculo::ETAPA_TOP_OF_PIPE,
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_TRANSFER_DST,
        CONSULTA_TIEMPO, Copia, ETAPA_CALCULO, ETAPA_HOST, InfoConsultas, ST_QUERY_POOL_CREATE_INFO,
    },
};
use crate::{
    TelemetriaGpu,
    eleccion::{MEMORIA_COHERENTE, MEMORIA_VISIBLE},
};
pub(crate) struct RelojVivo {
    fd: FnDispositivo,
    d: Disp,
    pool: Objeto,
    lecturas: Vec<BufferMapeado>,
    pendientes: [bool; 2],
    copiados: [bool; 2],
    pub(crate) control: Option<u32>,
    pub(crate) activa: bool,
    pub(crate) ultima: TelemetriaGpu,
    bits: u32,
    periodo: f32,
}
impl RelojVivo {
    pub(crate) fn nuevo(ctx: &Contexto) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("falta dispositivo de telemetría")?;
        let mut reloj = Self {
            fd,
            d: ctx.dispositivo,
            pool: 0,
            lecturas: Vec::new(),
            pendientes: [false; 2],
            copiados: [false; 2],
            control: None,
            activa: false,
            ultima: TelemetriaGpu::default(),
            bits: ctx.bits_tiempo,
            periodo: ctx.periodo_ns,
        };
        for _ in 0..2 {
            reloj.lecturas.push(
                Buffer::nuevo(ctx, 64, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?
                    .mapear_persistente()?,
            );
        }
        if ctx.bits_tiempo > 0 {
            let info = InfoConsultas {
                s_type: ST_QUERY_POOL_CREATE_INFO,
                p_next: core::ptr::null(),
                flags: 0,
                tipo: CONSULTA_TIEMPO,
                cantidad: 8,
                estadisticas: 0,
            };
            ok(
                // SAFETY: dispositivo propio y VkQueryPoolCreateInfo válida; pool destruido en Drop.
                unsafe {
                    (fd.calculo.crear_consultas)(
                        ctx.dispositivo,
                        &raw const info,
                        core::ptr::null(),
                        &raw mut reloj.pool,
                    )
                },
                "crear telemetría temporal",
            )?;
        }
        Ok(reloj)
    }
    pub(crate) fn recoger(&mut self, juego: usize) -> Result<(), String> {
        if !self.pendientes[juego] {
            return Ok(());
        }
        self.pendientes[juego] = false;
        self.ultima.muestras += 1;
        self.ultima.control = if self.copiados[juego] { Some(self.lecturas[juego].leer_control()?) } else { None };
        self.ultima.milisegundos = None;
        if self.pool != 0 {
            let mut datos = [0u64; 8];
            // SAFETY: la valla del juego acaba de completarse, resultados de cuatro consultas con disponibilidad, sin WAIT.
            let r = unsafe {
                (self.fd.calculo.leer_consultas)(
                    self.d,
                    self.pool,
                    u32::try_from(juego * 4).unwrap_or(0),
                    4,
                    64,
                    datos.as_mut_ptr().cast(),
                    16,
                    1 | 4,
                )
            };
            if r != 1 {
                ok(r, "leer tiempos vivos")?;
            } // NOT_READY se mantiene como N/D.
            if r == 0 && [1, 3, 5, 7].iter().all(|i| datos[*i] != 0) {
                self.ultima.milisegundos =
                    crate::telemetria::intervalos([datos[0], datos[2], datos[4], datos[6]], self.bits, self.periodo);
            }
        }
        Ok(())
    }
    pub(crate) fn enviado(&mut self, juego: usize, copiado: bool) {
        self.pendientes[juego] = true;
        self.copiados[juego] = copiado;
    }
    /// # Safety
    /// El comando se graba y el juego ya completó su valla antes de reutilizar consultas.
    pub(crate) unsafe fn marcar(&self, cmd: Comandos, juego: usize, marca: u32) {
        if self.pool == 0 {
            return;
        }
        let base = u32::try_from(juego * 4).unwrap_or(0);
        // SAFETY: contrato del llamador; las cuatro consultas del juego no están en vuelo.
        unsafe {
            if marca == 0 {
                (self.fd.calculo.reiniciar_consultas)(cmd, self.pool, base, 4);
            }
            (self.fd.calculo.marcar_tiempo)(
                cmd,
                if marca == 0 { ETAPA_TOP_OF_PIPE } else { ETAPA_BOTTOM_OF_PIPE },
                self.pool,
                base + marca,
            );
        }
    }
    /// # Safety
    /// Buffer fuente válido con `TRANSFER_SRC`, rango comprobado por Campos, juego fuera de uso.
    pub(crate) unsafe fn copiar(&self, cmd: Comandos, juego: usize, fuente: Option<(Objeto, u64)>) {
        let Some((fuente, offset)) = fuente else {
            return;
        };
        let copia = Copia { origen: offset, destino: 0, tamano: 64 };
        // SAFETY: contrato del llamador y buffers persistentes hasta la valla.
        unsafe {
            // GPU: ordena escrituras de campos y lecturas previas de transferencia antes de copiar control.
            super::campos::barrera(
                &self.fd,
                cmd,
                ETAPA_CALCULO | ETAPA_TRANSFER,
                ETAPA_TRANSFER,
                ACCESO_SHADER_WRITE | ACCESO_TRANSFER_READ,
                ACCESO_TRANSFER_READ,
            );
            (self.fd.calculo.copiar_buffer)(cmd, fuente, self.lecturas[juego].buffer.objeto, 1, &raw const copia);
            // GPU: la instantánea queda visible al host tras la valla; próximos shaders esperan la copia del estado.
            super::campos::barrera(
                &self.fd,
                cmd,
                ETAPA_TRANSFER,
                ETAPA_HOST | ETAPA_CALCULO,
                ACCESO_TRANSFER_WRITE | ACCESO_TRANSFER_READ,
                ACCESO_HOST_READ | ACCESO_SHADER_READ | ACCESO_SHADER_WRITE,
            );
        }
    }
}
impl Drop for RelojVivo {
    fn drop(&mut self) {
        // SAFETY: espera solo al destruir, antes de liberar consultas y mapeos usados por comandos en vuelo.
        unsafe {
            (self.fd.esperar)(self.d);
            if self.pool != 0 {
                (self.fd.calculo.destruir_consultas)(self.d, self.pool, core::ptr::null());
            }
        }
    }
}
