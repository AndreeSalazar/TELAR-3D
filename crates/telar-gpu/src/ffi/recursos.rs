//! Los recursos de una ejecución de cálculo: buffers con su memoria, el
//! pipeline (módulo, disposiciones y pipeline de cálculo), los descriptores y
//! los objetos de los comandos. Todo vive en `Recursos`, cuyo Drop lo suelta
//! también si algo falla a medias.

use core::ffi::c_void;

use super::contexto::{Contexto, ok};
use super::funciones::FnDispositivo;
use super::tipos::{
    COMPARTIR_EXCLUSIVO, Comandos, Disp, InfoReservaComandos, InfoSimple, NIVEL_PRIMARIO, Objeto,
    ST_COMMAND_BUFFER_ALLOCATE_INFO, ST_COMMAND_POOL_CREATE_INFO, ST_FENCE_CREATE_INFO,
};
use super::tipos_calculo::{
    CONSULTA_TIEMPO, DESCRIPTOR_ALMACEN, ETAPA_SHADER_CALCULO, Enlace, EscrituraDescriptor, InfoBuffer,
    InfoBufferDescriptor, InfoConsultas, InfoDisposicion, InfoDisposicionPipeline, InfoEtapa, InfoMemoria, InfoModulo,
    InfoPipelineCalculo, InfoPoolDescriptores, InfoReservaConjunto, Requisitos, ST_BUFFER_CREATE_INFO,
    ST_COMPUTE_PIPELINE_CREATE_INFO, ST_DESCRIPTOR_POOL_CREATE_INFO, ST_DESCRIPTOR_SET_ALLOCATE_INFO,
    ST_DESCRIPTOR_SET_LAYOUT_CREATE_INFO, ST_MEMORY_ALLOCATE_INFO, ST_PIPELINE_LAYOUT_CREATE_INFO,
    ST_PIPELINE_SHADER_STAGE_CREATE_INFO, ST_QUERY_POOL_CREATE_INFO, ST_SHADER_MODULE_CREATE_INFO,
    ST_WRITE_DESCRIPTOR_SET, TamanoPool,
};
use crate::eleccion::elegir_memoria;

/// Todo lo que se crea para una ejecución. Se suelta en Drop, también si algo
/// falla a medias (0 = no creado).
pub(super) struct Recursos {
    pub(super) fd: FnDispositivo,
    pub(super) d: Disp,
    pub(super) buffers: [Objeto; 2],
    pub(super) memorias: [Objeto; 2],
    pub(super) modulo: Objeto,
    pub(super) disposicion: Objeto,
    pub(super) disposicion_pipeline: Objeto,
    pub(super) pipeline: Objeto,
    pub(super) pool_descriptores: Objeto,
    pub(super) conjunto: Objeto,
    pub(super) pool_comandos: Objeto,
    pub(super) consultas: Objeto,
    pub(super) valla: Objeto,
}

impl Recursos {
    pub(super) fn nuevo(fd: &FnDispositivo, d: Disp) -> Self {
        Self {
            fd: *fd,
            d,
            buffers: [0; 2],
            memorias: [0; 2],
            modulo: 0,
            disposicion: 0,
            disposicion_pipeline: 0,
            pipeline: 0,
            pool_descriptores: 0,
            conjunto: 0,
            pool_comandos: 0,
            consultas: 0,
            valla: 0,
        }
    }

    /// Un buffer y su memoria: `(necesarias, preferidas)` son las propiedades de la memoria.
    pub(super) fn buffer(
        &mut self,
        ctx: &Contexto,
        i: usize,
        tamano: u64,
        uso: u32,
        memoria: (u32, u32),
    ) -> Result<(), String> {
        let fc = self.fd.calculo;
        let info = InfoBuffer {
            s_type: ST_BUFFER_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            tamano,
            uso,
            compartir: COMPARTIR_EXCLUSIVO,
            n_familias: 0,
            familias: core::ptr::null(),
        };
        let mut req = Requisitos::default();
        // SAFETY: dispositivo válido; `info` vive durante la llamada; el buffer queda
        // en `self` para soltarlo en Drop.
        unsafe {
            ok(
                (fc.crear_buffer)(self.d, &raw const info, core::ptr::null(), &raw mut self.buffers[i]),
                "crear un buffer",
            )?;
            (fc.requisitos)(self.d, self.buffers[i], &raw mut req);
        }
        let tipo = elegir_memoria(&ctx.tipos_memoria, req.tipos_permitidos, memoria.0, memoria.1)
            .ok_or("la GPU no tiene memoria del tipo que necesita el buffer")?;
        let reserva =
            InfoMemoria { s_type: ST_MEMORY_ALLOCATE_INFO, p_next: core::ptr::null(), tamano: req.tamano, tipo };
        // SAFETY: el tipo de memoria está entre los permitidos para este buffer; la
        // memoria queda en `self` (Drop la libera después del buffer).
        unsafe {
            ok(
                (fc.reservar_memoria)(self.d, &raw const reserva, core::ptr::null(), &raw mut self.memorias[i]),
                "reservar memoria",
            )?;
            ok((fc.unir_memoria)(self.d, self.buffers[i], self.memorias[i], 0), "unir la memoria al buffer")
        }
    }

    /// Módulo, disposición del conjunto (un buffer de almacenamiento) y pipeline de cálculo.
    pub(super) fn pipeline(&mut self, spirv: &[u32]) -> Result<(), String> {
        self.pipeline_enlaces(spirv, 1)
    }

    /// Pipeline de cálculo con `cantidad` buffers de almacenamiento consecutivos.
    pub(super) fn pipeline_enlaces(&mut self, spirv: &[u32], cantidad: u32) -> Result<(), String> {
        self.pipeline_opciones(spirv, cantidad, 0, false)
    }

    pub(super) fn pipeline_base(&mut self, spirv: &[u32]) -> Result<(), String> {
        self.pipeline_opciones(spirv, 1, 0x10, false) // VK_PIPELINE_CREATE_DISPATCH_BASE_BIT.
    }
    pub(super) fn pipeline_campos(&mut self, spirv: &[u32]) -> Result<(), String> {
        self.pipeline_opciones(spirv, 2, 0, true)
    }
    fn pipeline_opciones(&mut self, spirv: &[u32], cantidad: u32, flags: u32, parametros: bool) -> Result<(), String> {
        let fc = self.fd.calculo;
        let modulo = InfoModulo {
            s_type: ST_SHADER_MODULE_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            tamano: core::mem::size_of_val(spirv),
            codigo: spirv.as_ptr(),
        };
        let enlaces: Vec<Enlace> = (0..cantidad)
            .map(|numero| Enlace {
                numero,
                tipo: DESCRIPTOR_ALMACEN,
                cantidad: 1,
                etapas: ETAPA_SHADER_CALCULO
                    | if matches!(numero, 1 | 3) { super::tipos_grafico::ETAPA_FRAGMENTOS } else { 0 },
                muestreadores: core::ptr::null(),
            })
            .collect();
        let disposicion = InfoDisposicion {
            s_type: ST_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            n_enlaces: cantidad,
            enlaces: enlaces.as_ptr(),
        };
        let nulo = core::ptr::null();
        // SAFETY: el SPIR-V vive durante la llamada (Vulkan lo copia); las demás
        // `info` también; todo lo creado queda en `self`.
        unsafe {
            ok((fc.crear_modulo)(self.d, &raw const modulo, nulo, &raw mut self.modulo), "crear el módulo del shader")?;
            ok(
                (fc.crear_disposicion)(self.d, &raw const disposicion, nulo, &raw mut self.disposicion),
                "crear la disposición",
            )?;
        }
        let rango =
            super::tipos_grafico::RangoConstantes { etapas: ETAPA_SHADER_CALCULO, desplazamiento: 0, tamano: 128 };
        let pipeline_disp = InfoDisposicionPipeline {
            s_type: ST_PIPELINE_LAYOUT_CREATE_INFO,
            p_next: nulo,
            flags: 0,
            n_conjuntos: 1,
            conjuntos: &raw const self.disposicion,
            n_constantes: u32::from(parametros),
            constantes: if parametros { (&raw const rango).cast() } else { nulo },
        };
        // SAFETY: la disposición del conjunto existe; `pipeline_disp` vive durante la llamada.
        let r = unsafe {
            (fc.crear_disposicion_pipeline)(self.d, &raw const pipeline_disp, nulo, &raw mut self.disposicion_pipeline)
        };
        ok(r, "crear la disposición del pipeline")?;
        let info = InfoPipelineCalculo {
            s_type: ST_COMPUTE_PIPELINE_CREATE_INFO,
            p_next: nulo,
            flags,
            etapa: InfoEtapa {
                s_type: ST_PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: nulo,
                flags: 0,
                etapa: ETAPA_SHADER_CALCULO,
                modulo: self.modulo,
                nombre: c"main".as_ptr().cast::<u8>(),
                especializacion: nulo,
            },
            disposicion: self.disposicion_pipeline,
            base: 0,
            indice_base: -1,
        };
        // SAFETY: módulo y disposición existen; `info` vive durante la llamada; sin caché (0).
        let r = unsafe { (fc.crear_pipelines)(self.d, 0, 1, &raw const info, nulo, &raw mut self.pipeline) };
        ok(r, "crear el pipeline de cálculo")
    }

    /// El conjunto de descriptores: el buffer 0, entero, en el enlace 0.
    pub(super) fn descriptores(&mut self, tamano: u64) -> Result<(), String> {
        self.descriptores_buffers(&[(self.buffers[0], tamano)])
    }

    /// Vincula buffers externos residentes; su propietario mantiene su vida útil.
    pub(super) fn descriptores_buffers(&mut self, buffers: &[(Objeto, u64)]) -> Result<(), String> {
        let fc = self.fd.calculo;
        let nulo = core::ptr::null();
        let cantidad = u32::try_from(buffers.len()).map_err(|_| "demasiados enlaces")?;
        let tamanos = TamanoPool { tipo: DESCRIPTOR_ALMACEN, cantidad };
        let pool = InfoPoolDescriptores {
            s_type: ST_DESCRIPTOR_POOL_CREATE_INFO,
            p_next: nulo,
            flags: 0,
            max_conjuntos: 1,
            n_tamanos: 1,
            tamanos: &raw const tamanos,
        };
        // SAFETY: dispositivo válido; `pool` vive durante la llamada.
        let r = unsafe { (fc.crear_pool_descriptores)(self.d, &raw const pool, nulo, &raw mut self.pool_descriptores) };
        ok(r, "crear el pool de descriptores")?;
        let reserva = InfoReservaConjunto {
            s_type: ST_DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: nulo,
            pool: self.pool_descriptores,
            n_conjuntos: 1,
            disposiciones: &raw const self.disposicion,
        };
        // SAFETY: pool y disposición existen; el conjunto se escribe antes de usarse
        // y `buffer` vive durante la escritura.
        unsafe {
            ok((fc.reservar_conjuntos)(self.d, &raw const reserva, &raw mut self.conjunto), "reservar el conjunto")?;
            for (enlace, &(objeto, rango)) in (0u32..).zip(buffers) {
                let buffer = InfoBufferDescriptor { buffer: objeto, desplazamiento: 0, rango };
                let escritura = EscrituraDescriptor {
                    s_type: ST_WRITE_DESCRIPTOR_SET,
                    p_next: nulo,
                    conjunto: self.conjunto,
                    enlace,
                    elemento: 0,
                    cantidad: 1,
                    tipo: DESCRIPTOR_ALMACEN,
                    imagenes: nulo,
                    buffers: &raw const buffer,
                    vistas: nulo,
                };
                (fc.escribir_descriptores)(self.d, 1, &raw const escritura, 0, nulo);
            }
        }
        Ok(())
    }

    /// El pool de comandos, la valla, las consultas de tiempo (si se mide) y el búfer de comandos.
    pub(super) fn preparar(&mut self, ctx: &Contexto, medir: bool) -> Result<Comandos, String> {
        self.preparar_consultas(ctx, if medir { 2 } else { 0 })
    }

    /// Un comando primario reutilizable y el número solicitado de consultas.
    pub(super) fn preparar_consultas(&mut self, ctx: &Contexto, cantidad: u32) -> Result<Comandos, String> {
        self.preparar_opciones(ctx, cantidad, 0)
    }
    pub(super) fn preparar_reutilizable(&mut self, ctx: &Contexto) -> Result<Comandos, String> {
        self.preparar_opciones(ctx, 0, 2) // VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT.
    }
    fn preparar_opciones(&mut self, ctx: &Contexto, cantidad: u32, flags: u32) -> Result<Comandos, String> {
        let (fd, fc, nulo) = (self.fd, self.fd.calculo, core::ptr::null());
        let pool = InfoSimple { s_type: ST_COMMAND_POOL_CREATE_INFO, p_next: nulo, flags, familia: ctx.familia };
        let valla = InfoSimple { s_type: ST_FENCE_CREATE_INFO, p_next: nulo, flags: 0, familia: 0 };
        let consultas = InfoConsultas {
            s_type: ST_QUERY_POOL_CREATE_INFO,
            p_next: nulo,
            flags: 0,
            tipo: CONSULTA_TIEMPO,
            cantidad,
            estadisticas: 0,
        };
        let mut cmd: Comandos = core::ptr::null_mut();
        // SAFETY: dispositivo válido; cada `info` vive durante su llamada; lo creado
        // queda en `self`; el búfer de comandos lo libera su pool.
        unsafe {
            ok(
                (fd.crear_pool)(self.d, &raw const pool, nulo, &raw mut self.pool_comandos),
                "crear el pool de comandos",
            )?;
            ok((fd.crear_valla)(self.d, &raw const valla, nulo, &raw mut self.valla), "crear la valla")?;
            if cantidad > 0 {
                ok(
                    (fc.crear_consultas)(self.d, &raw const consultas, nulo, &raw mut self.consultas),
                    "crear las consultas",
                )?;
            }
            let reserva = InfoReservaComandos {
                s_type: ST_COMMAND_BUFFER_ALLOCATE_INFO,
                p_next: nulo,
                pool: self.pool_comandos,
                nivel: NIVEL_PRIMARIO,
                cantidad: 1,
            };
            ok((fd.reservar_comandos)(self.d, &raw const reserva, &raw mut cmd), "reservar los comandos")?;
        }
        Ok(cmd)
    }
}

impl Drop for Recursos {
    fn drop(&mut self) {
        let (fd, fc, nulo) = (self.fd, self.fd.calculo, core::ptr::null());
        // SAFETY: tras esperar a la GPU nada está en uso; cada objeto se destruye una
        // vez, solo si se creó, y antes que aquello de lo que depende.
        unsafe {
            (fd.esperar)(self.d);
            let destruir = |f: unsafe extern "system" fn(Disp, Objeto, *const c_void), o: Objeto| {
                if o != 0 {
                    f(self.d, o, nulo);
                }
            };
            destruir(fd.destruir_valla, self.valla);
            destruir(fd.destruir_pool, self.pool_comandos);
            destruir(fc.destruir_consultas, self.consultas);
            destruir(fc.destruir_pool_descriptores, self.pool_descriptores);
            destruir(fc.destruir_pipeline, self.pipeline);
            destruir(fc.destruir_disposicion_pipeline, self.disposicion_pipeline);
            destruir(fc.destruir_disposicion, self.disposicion);
            destruir(fc.destruir_modulo, self.modulo);
            for i in 0..2 {
                destruir(fc.destruir_buffer, self.buffers[i]);
                destruir(fc.liberar_memoria, self.memorias[i]);
            }
        }
    }
}
