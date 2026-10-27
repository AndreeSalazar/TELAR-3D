//! Contrato público de ejecución residente, sin dependencia del compilador ni de NUDO.

use crate::{Calculo, Constantes, DominioProcedural, EtapaProcedural, Gpu, PlanProcedural, Sombreadores, ffi};

/// Tres programas con el ABI documentado en hilo-spirv: productor, generador y deformador.
#[derive(Clone, Copy)]
pub struct KernelsProcedurales<'a> {
    /// Escribe argumentos de despacho y dibujo en el buffer de control.
    pub preparar: &'a [u32],
    /// Escribe vértices e índices.
    pub generar: &'a [u32],
    /// Lee y modifica los vértices residentes.
    pub deformar: &'a [u32],
}

impl<'a> KernelsProcedurales<'a> {
    /// Traduce la demostración original de tres kernels al plan extensible.
    #[must_use]
    pub const fn etapas(self) -> [EtapaProcedural<'a>; 3] {
        [
            EtapaProcedural { nombre: "preparar", codigo: self.preparar, dominio: DominioProcedural::Preparar },
            EtapaProcedural { nombre: "generar", codigo: self.generar, dominio: DominioProcedural::Generar },
            EtapaProcedural { nombre: "deformar", codigo: self.deformar, dominio: DominioProcedural::Puntos },
        ]
    }
}

/// Parámetros pequeños; ningún vértice ni índice cruza desde el host.
#[derive(Clone, Copy)]
pub struct ParametrosProcedurales {
    /// Puntos por lado. 0 y 1 producen una superficie vacía.
    pub lado: u32,
    /// Fase de la onda, en radianes.
    pub fase: f32,
    /// Altura máxima de la onda.
    pub amplitud: f32,
}

impl ParametrosProcedurales {
    pub(crate) fn palabras(self, capacidad: u32) -> Result<[u32; 4], String> {
        if !self.fase.is_finite()
            || !self.amplitud.is_finite()
            || self.fase.abs() > 1_000_000.0
            || self.amplitud.abs() > 1_000.0
        {
            return Err("fase o amplitud fuera del intervalo finito admitido".to_owned());
        }
        Ok([self.lado, self.fase.to_bits(), self.amplitud.to_bits(), capacidad])
    }
}

/// Contadores de esta ruta: las capturas y descargas de diagnóstico son explícitas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActividadProcedural {
    /// Buffers residentes creados al preparar los juegos en vuelo.
    pub buffers: u64,
    /// Pipelines de compute creados al preparar.
    pub pipelines: u64,
    /// Grabaciones de compute, reutilizadas posteriormente.
    pub grabaciones: u64,
    /// Mapeos persistentes de parámetros, creados una vez por juego residente.
    pub mapeos: u64,
    /// Lotes enviados que contienen el plan completo.
    pub pasos: u64,
    /// Bytes de parámetros escritos por el host.
    pub parametros: u64,
    /// Bytes de geometría subidos en esta ruta.
    pub geometria_subida: u64,
    /// Bytes descargados por la inspección explícita.
    pub descargados: u64,
}

/// Copia diagnóstica del estado GPU; nunca interviene en las decisiones del paso.
pub struct DiagnosticoProcedural {
    /// Dispatch, draw y estado, según el ABI del buffer de control.
    pub control: [u32; 16],
    /// Vértices activos, posición y normal.
    pub vertices: Vec<[f32; 6]>,
    /// Índices activos.
    pub indices: Vec<u32>,
    /// Nombres de las operaciones, en el mismo orden que sus tiempos.
    pub etapas: Vec<String>,
    /// Tiempos GPU por operación, en nanosegundos.
    pub nanosegundos: Option<Vec<f64>>,
}

/// Ejecutor sin ventana; el préstamo mantiene vivo el dispositivo.
pub struct Procedural<'a> {
    pub(crate) interior: ffi::Residente,
    pub(crate) calculo: &'a Calculo,
}

impl Calculo {
    /// Prepara una ruta persistente de prueba, sin generar geometría en CPU.
    ///
    /// # Errors
    /// Rechaza capacidades fuera de 2..=1024 o errores de Vulkan.
    pub fn procedural(&self, capacidad: u32, kernels: KernelsProcedurales) -> Result<Procedural<'_>, String> {
        let etapas = kernels.etapas();
        self.procedural_con_plan(capacidad, PlanProcedural::nuevo(&etapas)?)
    }

    /// Prepara un plan de longitud variable; recursos y comandos quedan residentes.
    ///
    /// # Errors
    /// Si la capacidad no vale, el dispositivo rechaza un shader o Vulkan falla.
    pub fn procedural_con_plan(&self, capacidad: u32, plan: PlanProcedural) -> Result<Procedural<'_>, String> {
        Ok(Procedural { interior: ffi::Residente::nuevo(&self.contexto, capacidad, plan, 1)?, calculo: self })
    }
}

impl Procedural<'_> {
    /// Envía el lote pregrabado y espera su valla para la prueba sin ventana.
    /// La ruta con ventana reutiliza las vallas de presentación.
    ///
    /// # Errors
    /// Si los parámetros no son finitos o Vulkan falla.
    pub fn paso(&mut self, parametros: ParametrosProcedurales) -> Result<(), String> {
        self.interior.paso(&self.calculo.contexto, parametros)
    }

    /// Descarga explícita para comparar contra una referencia CPU.
    ///
    /// # Errors
    /// Si Vulkan falla o los recuentos GPU son inválidos.
    pub fn inspeccionar(&mut self) -> Result<DiagnosticoProcedural, String> {
        self.interior.inspeccionar(&self.calculo.contexto)
    }

    /// Dibuja indirectamente los buffers residentes en una captura diagnóstica.
    ///
    /// # Errors
    /// Si no se ejecutó un paso, el tamaño no vale o Vulkan falla.
    pub fn capturar(
        &self,
        shaders: Sombreadores,
        tamano: (u32, u32),
        fondo: [f32; 4],
        constantes: &Constantes,
    ) -> Result<Vec<u8>, String> {
        self.interior.capturar(&self.calculo.contexto, shaders, tamano, fondo, constantes)
    }

    /// Actividad acumulada del ejecutor.
    #[must_use]
    pub fn actividad(&self) -> ActividadProcedural {
        self.interior.actividad()
    }
}

impl Gpu {
    /// Prepara dos juegos residentes, uno por fotograma en vuelo.
    ///
    /// # Errors
    /// Si falta el visor, la capacidad no vale o Vulkan falla.
    pub fn preparar_procedural(&mut self, capacidad: u32, kernels: KernelsProcedurales) -> Result<(), String> {
        let etapas = kernels.etapas();
        self.preparar_plan_procedural(capacidad, PlanProcedural::nuevo(&etapas)?)
    }

    /// Prepara un plan extensible para los dos fotogramas en vuelo.
    ///
    /// # Errors
    /// Si falta el visor, la capacidad no vale o Vulkan falla.
    pub fn preparar_plan_procedural(&mut self, capacidad: u32, plan: PlanProcedural) -> Result<(), String> {
        let visor = self.visor.as_mut().ok_or("primero hay que preparar el visor")?;
        visor.preparar_procedural(&self.contexto, capacidad, plan)
    }

    /// Actualiza parámetros y envía compute pregrabado junto con el dibujo indirecto.
    ///
    /// # Errors
    /// Si falta el ejecutor, los parámetros no valen o Vulkan falla.
    pub fn dibujar_procedural(
        &mut self,
        fondo: [f32; 4],
        constantes: &Constantes,
        parametros: ParametrosProcedurales,
    ) -> Result<bool, String> {
        let formato = self.presentacion.formato;
        if let Some(interfaz) = &mut self.interfaz {
            interfaz.formato(&self.contexto, formato)?;
        }
        let plana = constantes.plana(formato);
        let visor = self.visor.as_mut().ok_or("falta el visor")?;
        let escena =
            visor.escena_procedural(&self.contexto, formato, &plana, self.presentacion.cuadro(), parametros)?;
        let dibujado = self.presentacion.dibujar(
            &self.contexto,
            crate::eleccion::color_para(formato, fondo),
            Some(&escena),
            self.interfaz.as_mut(),
        )?;
        if dibujado {
            visor.contar_paso();
        }
        Ok(dibujado)
    }

    /// Actividad de la ruta procedural del visor.
    #[must_use]
    pub fn actividad_procedural(&self) -> ActividadProcedural {
        self.visor.as_ref().map_or(ActividadProcedural::default(), ffi::Visor::actividad)
    }
}
