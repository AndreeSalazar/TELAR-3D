//! El pipeline de gráficos del visor: dos shaders (vértices y fragmentos),
//! 128 bytes de constantes empujadas, un vértice = posición + normal (24 bytes),
//! caras traseras descartadas y profundidad invertida (`GREATER`, se limpia a 0).
//! Dibuja con renderizado dinámico: no hay render pass, solo los formatos.

use core::ffi::c_void;

use super::contexto::{Contexto, ok};
use super::funciones::FnDispositivo;
use super::tipos::Disp;
use super::tipos::Objeto;
use super::tipos_calculo::{
    InfoDisposicionPipeline, InfoEtapa, InfoModulo, ST_PIPELINE_LAYOUT_CREATE_INFO,
    ST_PIPELINE_SHADER_STAGE_CREATE_INFO, ST_SHADER_MODULE_CREATE_INFO,
};
use super::tipos_grafico::{
    AtributoVertice, COMPARAR_MAYOR, DESCARTAR_TRASERAS, DINAMICO_TIJERA, DINAMICO_VIEWPORT, ESCRIBIR_RGBA,
    ETAPA_FRAGMENTOS, ETAPA_VERTICES, EnlaceVertices, EstadoDinamico, EstadoEnsamblado, EstadoMezcla, EstadoMuestreo,
    EstadoProfundidad, EstadoRasterizado, EstadoVertices, EstadoViewport, FORMATO_PROFUNDIDAD, FORMATO_RGB32F,
    FRENTE_ANTIHORARIO, InfoPipelineGrafico, InfoRenderizadoPipeline, MUESTRAS_1, MezclaAdjunto, POR_VERTICE, RELLENAR,
    RangoConstantes, ST_COLOR_BLEND, ST_DEPTH_STENCIL, ST_DYNAMIC_STATE, ST_GRAPHICS_PIPELINE, ST_INPUT_ASSEMBLY,
    ST_MULTISAMPLE, ST_PIPELINE_RENDERING, ST_RASTERIZATION, ST_VERTEX_INPUT, ST_VIEWPORT_STATE, TRIANGULOS,
};

/// Bytes de constantes empujadas: 8 `vec4` (la matriz, la luz y tres colores).
pub(crate) const TAMANO_CONSTANTES: u32 = 128;
/// Bytes por vértice: posición y normal, tres `f32` cada una.
const PASO_VERTICE: u32 = 24;
const NULO: *const c_void = core::ptr::null();

/// El pipeline y su disposición. Se suelta en Drop.
pub(crate) struct PipelineGrafico {
    fd: FnDispositivo,
    d: Disp,
    pub(crate) disposicion: Objeto,
    pub(crate) pipeline: Objeto,
    /// El formato de color para el que se creó.
    pub(crate) formato: u32,
    interfaz: bool,
}

impl PipelineGrafico {
    /// `vertices` y `fragmentos`: SPIR-V con la entrada `main`.
    pub(crate) fn nuevo(ctx: &Contexto, vertices: &[u32], fragmentos: &[u32], formato: u32) -> Result<Self, String> {
        Self::crear_modo(ctx, vertices, fragmentos, formato, false, 0)
    }

    pub(crate) fn interfaz(
        ctx: &Contexto,
        vertices: &[u32],
        fragmentos: &[u32],
        formato: u32,
        conjunto: Objeto,
    ) -> Result<Self, String> {
        Self::crear_modo(ctx, vertices, fragmentos, formato, true, conjunto)
    }

    fn crear_modo(
        ctx: &Contexto,
        vertices: &[u32],
        fragmentos: &[u32],
        formato: u32,
        interfaz: bool,
        conjunto: Objeto,
    ) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let mut p = Self { fd, d: ctx.dispositivo, disposicion: 0, pipeline: 0, formato, interfaz };
        let rango =
            RangoConstantes { etapas: ETAPA_VERTICES | ETAPA_FRAGMENTOS, desplazamiento: 0, tamano: TAMANO_CONSTANTES };
        let info = InfoDisposicionPipeline {
            s_type: ST_PIPELINE_LAYOUT_CREATE_INFO,
            p_next: NULO,
            flags: 0,
            n_conjuntos: u32::from(interfaz),
            conjuntos: if interfaz { &raw const conjunto } else { core::ptr::null() },
            n_constantes: 1,
            constantes: (&raw const rango).cast(),
        };
        let fc = fd.calculo;
        // SAFETY: dispositivo válido; `info` y `rango` viven durante la llamada.
        let r = unsafe { (fc.crear_disposicion_pipeline)(p.d, &raw const info, NULO, &raw mut p.disposicion) };
        ok(r, "crear la disposición del pipeline")?;
        // Los módulos solo hacen falta para crear el pipeline: se sueltan pase lo que pase.
        let de_vertices = p.modulo(vertices)?;
        let creado = p.modulo(fragmentos).and_then(|de_fragmentos| {
            let r = p.crear([de_vertices, de_fragmentos]);
            p.soltar_modulo(de_fragmentos);
            r
        });
        p.soltar_modulo(de_vertices);
        p.pipeline = creado?;
        Ok(p)
    }

    fn modulo(&self, spirv: &[u32]) -> Result<Objeto, String> {
        let info = InfoModulo {
            s_type: ST_SHADER_MODULE_CREATE_INFO,
            p_next: NULO,
            flags: 0,
            tamano: core::mem::size_of_val(spirv),
            codigo: spirv.as_ptr(),
        };
        let mut modulo = 0;
        // SAFETY: `spirv` vive durante la llamada y `tamano` son sus bytes.
        let r = unsafe { (self.fd.calculo.crear_modulo)(self.d, &raw const info, NULO, &raw mut modulo) };
        ok(r, "crear un módulo de shader (¿SPIR-V inválido?)")?;
        Ok(modulo)
    }

    fn soltar_modulo(&self, modulo: Objeto) {
        if modulo != 0 {
            // SAFETY: los módulos solo hacen falta mientras se crea el pipeline.
            unsafe { (self.fd.calculo.destruir_modulo)(self.d, modulo, NULO) };
        }
    }

    fn crear(&self, modulos: [Objeto; 2]) -> Result<Objeto, String> {
        let etapa = |etapa, modulo| InfoEtapa {
            s_type: ST_PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: NULO,
            flags: 0,
            etapa,
            modulo,
            nombre: c"main".as_ptr().cast(),
            especializacion: NULO,
        };
        let etapas = [etapa(ETAPA_VERTICES, modulos[0]), etapa(ETAPA_FRAGMENTOS, modulos[1])];
        let enlace =
            EnlaceVertices { numero: 0, paso: if self.interfaz { 32 } else { PASO_VERTICE }, ritmo: POR_VERTICE };
        let atributo =
            |ubicacion, desplazamiento, formato| AtributoVertice { ubicacion, enlace: 0, formato, desplazamiento };
        let atributos = if self.interfaz {
            vec![atributo(0, 0, 103), atributo(1, 8, 103), atributo(2, 16, 109)]
        } else {
            vec![atributo(0, 0, FORMATO_RGB32F), atributo(1, 12, FORMATO_RGB32F)]
        };
        let vertices = EstadoVertices {
            s_type: ST_VERTEX_INPUT,
            p_next: NULO,
            flags: 0,
            n_enlaces: 1,
            enlaces: &raw const enlace,
            n_atributos: if self.interfaz { 3 } else { 2 },
            atributos: atributos.as_ptr(),
        };
        let (ensamblado, viewport) = (ensamblado(), viewport());
        let (mut rasterizado, muestreo, mut profundidad) = (rasterizado(), muestreo(), profundidad());
        if self.interfaz {
            rasterizado.descartar_caras = 0;
            profundidad.prueba = 0;
            profundidad.escritura = 0;
        }
        let adjunto = if self.interfaz {
            MezclaAdjunto { activa: 1, factores_color: [6, 7, 0], factores_alfa: [1, 7, 0], mascara: ESCRIBIR_RGBA }
        } else {
            MezclaAdjunto { activa: 0, factores_color: [0; 3], factores_alfa: [0; 3], mascara: ESCRIBIR_RGBA }
        };
        let mezcla = EstadoMezcla {
            s_type: ST_COLOR_BLEND,
            p_next: NULO,
            flags: 0,
            operacion_logica: 0,
            logica: 0,
            n_adjuntos: 1,
            adjuntos: &raw const adjunto,
            constantes: [0.0; 4],
        };
        let estados = [DINAMICO_VIEWPORT, DINAMICO_TIJERA];
        let dinamico = EstadoDinamico {
            s_type: ST_DYNAMIC_STATE,
            p_next: NULO,
            flags: 0,
            n_estados: 2,
            estados: estados.as_ptr(),
        };
        let renderizado = InfoRenderizadoPipeline {
            s_type: ST_PIPELINE_RENDERING,
            p_next: NULO,
            mascara_vista: 0,
            n_colores: 1,
            formatos_color: &raw const self.formato,
            formato_profundidad: FORMATO_PROFUNDIDAD,
            formato_plantilla: 0,
        };
        let info = InfoPipelineGrafico {
            s_type: ST_GRAPHICS_PIPELINE,
            p_next: (&raw const renderizado).cast(),
            flags: 0,
            n_etapas: 2,
            etapas: etapas.as_ptr().cast(),
            vertices: &raw const vertices,
            ensamblado: &raw const ensamblado,
            teselado: NULO,
            viewport: &raw const viewport,
            rasterizado: &raw const rasterizado,
            muestreo: &raw const muestreo,
            profundidad: &raw const profundidad,
            mezcla: &raw const mezcla,
            dinamico: &raw const dinamico,
            disposicion: self.disposicion,
            pase: 0,
            subpase: 0,
            base: 0,
            indice_base: -1,
        };
        let mut pipeline = 0;
        // SAFETY: todos los estados viven hasta el final de la función y los punteros
        // de `info` apuntan a ellos; los módulos y la disposición existen.
        let r = unsafe { (self.fd.grafico.crear_pipelines)(self.d, 0, 1, &raw const info, NULO, &raw mut pipeline) };
        ok(r, "crear el pipeline de gráficos")?;
        Ok(pipeline)
    }
}

impl Drop for PipelineGrafico {
    fn drop(&mut self) {
        let fc = self.fd.calculo;
        // SAFETY: quien lo suelta esperó a que la GPU acabara; cada objeto se destruye
        // una vez y solo si se creó.
        unsafe {
            if self.pipeline != 0 {
                (fc.destruir_pipeline)(self.d, self.pipeline, NULO);
            }
            if self.disposicion != 0 {
                (fc.destruir_disposicion_pipeline)(self.d, self.disposicion, NULO);
            }
        }
    }
}

// Los estados fijos, sin punteros a nada: se construyen aparte.

fn ensamblado() -> EstadoEnsamblado {
    EstadoEnsamblado { s_type: ST_INPUT_ASSEMBLY, p_next: NULO, flags: 0, topologia: TRIANGULOS, reinicio: 0 }
}

/// Viewport y tijera dinámicos: solo se dice cuántos.
fn viewport() -> EstadoViewport {
    EstadoViewport {
        s_type: ST_VIEWPORT_STATE,
        p_next: NULO,
        flags: 0,
        n_viewports: 1,
        viewports: NULO,
        n_tijeras: 1,
        tijeras: NULO,
    }
}

fn rasterizado() -> EstadoRasterizado {
    EstadoRasterizado {
        s_type: ST_RASTERIZATION,
        p_next: NULO,
        flags: 0,
        limitar_profundidad: 0,
        descartar: 0,
        modo: RELLENAR,
        descartar_caras: DESCARTAR_TRASERAS,
        frente: FRENTE_ANTIHORARIO,
        sesgo: 0,
        sesgo_constante: 0.0,
        sesgo_limite: 0.0,
        sesgo_pendiente: 0.0,
        grosor_linea: 1.0,
    }
}

fn muestreo() -> EstadoMuestreo {
    EstadoMuestreo {
        s_type: ST_MULTISAMPLE,
        p_next: NULO,
        flags: 0,
        muestras: MUESTRAS_1,
        sombreado_por_muestra: 0,
        minimo: 0.0,
        mascara: core::ptr::null(),
        alfa_a_cobertura: 0,
        alfa_a_uno: 0,
    }
}

fn profundidad() -> EstadoProfundidad {
    EstadoProfundidad {
        s_type: ST_DEPTH_STENCIL,
        p_next: NULO,
        flags: 0,
        prueba: 1,
        escritura: 1,
        comparacion: COMPARAR_MAYOR, // profundidad invertida: lo cercano vale más
        limites: 0,
        plantilla: 0,
        plantilla_frente: [0; 7],
        plantilla_detras: [0; 7],
        minimo: 0.0,
        maximo: 1.0,
    }
}
