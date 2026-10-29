//! El visor: los shaders, el pipeline (que se hace para el formato de la
//! ventana, y se rehace si cambia) y la malla subida. Y la captura: lo mismo,
//! pero en una imagen fuera de pantalla que vuelve a la CPU, para los tests.

use super::buffer::{Buffer, una_vez};
use super::contexto::{Contexto, ok};
use super::dibujo::{Destino, Escena, grabar};
use super::funciones::FnDispositivo;
use super::imagen::Imagen;
use super::malla_gpu::MallaGpu;
use super::pipeline_grafico::PipelineGrafico;
use super::residente::Residente;
use super::tipos::{ACCESO_TRANSFER_WRITE, ASPECTO_COLOR, Disp, ETAPA_TRANSFER, USO_COLOR_ATTACHMENT};
use super::tipos_calculo::{
    ACCESO_HOST_READ, ACCESO_TRANSFER_READ, BUFFER_TRANSFER_DST, BarreraMemoria, ETAPA_HOST, ST_MEMORY_BARRIER,
};
use super::tipos_grafico::{CopiaImagen, Extension3D, FORMATO_RGBA8, LAYOUT_TRANSFER_SRC, USO_TRANSFER_SRC};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_CON_CACHE, MEMORIA_VISIBLE};
use crate::{ActividadProcedural, ParametrosNodos, PlanProcedural};

/// SPIR-V de vértices y de fragmentos.
pub(crate) type Sombreadores<'a> = (&'a [u32], &'a [u32]);

/// Al soltarse espera a la GPU: su pipeline y su malla pueden estar en un
/// fotograma en vuelo.
pub(crate) struct Visor {
    fd: FnDispositivo,
    d: Disp,
    vertices: Vec<u32>,
    fragmentos: Vec<u32>,
    pipeline: Option<PipelineGrafico>,
    malla: Option<MallaGpu>,
    procedural: Option<Residente>,
    historial: Option<super::historial::Historial>,
    campos: Option<super::campos::Campos>,
}

impl Visor {
    pub(crate) fn carga_campos(&self) -> crate::CargaCampos {
        self.campos.as_ref().map_or_else(crate::CargaCampos::default, super::campos::Campos::carga)
    }
    pub(crate) fn validar_control(&self, offset: u32) -> Result<(), String> {
        self.campos.as_ref().and_then(|c| c.control(offset)).map(|_| ()).ok_or("rango de telemetría inválido".into())
    }
    pub(crate) fn nuevo(ctx: &Contexto, (vertices, fragmentos): Sombreadores) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
        let (vertices, fragmentos) = (vertices.to_vec(), fragmentos.to_vec());
        Ok(Self {
            fd,
            d: ctx.dispositivo,
            vertices,
            fragmentos,
            pipeline: None,
            malla: None,
            procedural: None,
            historial: None,
            campos: None,
        })
    }

    /// Otros shaders: el pipeline se rehará en el próximo dibujo.
    pub(crate) fn cambiar(&mut self, ctx: &Contexto, (vertices, fragmentos): Sombreadores) -> Result<(), String> {
        esperar(ctx)?;
        self.historial = None;
        self.campos = None;
        self.pipeline = None;
        (self.vertices, self.fragmentos) = (vertices.to_vec(), fragmentos.to_vec());
        Ok(())
    }

    /// Sube una malla nueva en lugar de la que había.
    pub(crate) fn subir(&mut self, ctx: &Contexto, vertices: &[[f32; 6]], indices: &[u32]) -> Result<(), String> {
        let nueva = MallaGpu::nueva(ctx, vertices, indices)?;
        esperar(ctx)?; // la vieja puede estar en un fotograma en vuelo
        self.malla = Some(nueva);
        Ok(())
    }

    /// Lo que hay que dibujar para una imagen de `formato` (`None` si aún no hay malla).
    pub(crate) fn escena<'a>(
        &'a mut self,
        ctx: &Contexto,
        formato: u32,
        constantes: &'a [f32; 32],
    ) -> Result<Option<Escena<'a>>, String> {
        if let Some(historial) = &self.historial {
            historial.validar_bloque(constantes)?;
        }
        let muestras = ctx.muestras(formato)?;
        if self.pipeline.as_ref().is_none_or(|p| p.formato != formato || p.muestras != muestras) {
            esperar(ctx)?;
            self.pipeline = None;
            self.pipeline = Some(if let Some(historial) = &self.historial {
                PipelineGrafico::residente(
                    ctx,
                    &self.vertices,
                    &self.fragmentos,
                    formato,
                    historial.recursos.disposicion,
                )?
            } else if let Some(campos) = &self.campos {
                PipelineGrafico::residente(
                    ctx,
                    &self.vertices,
                    &self.fragmentos,
                    formato,
                    campos.programas[0].disposicion,
                )?
            } else if let Some(residente) = &self.procedural {
                let disposicion = residente.ranuras[0].etapas[0].recursos.disposicion;
                PipelineGrafico::residente(ctx, &self.vertices, &self.fragmentos, formato, disposicion)?
            } else {
                PipelineGrafico::nuevo(ctx, &self.vertices, &self.fragmentos, formato)?
            });
        }
        Ok(match (&self.pipeline, &self.malla) {
            (Some(pipeline), Some(malla)) => Some(Escena {
                pipeline,
                malla,
                constantes,
                indirecto: None,
                conjunto: self
                    .historial
                    .as_ref()
                    .map(|h| h.recursos.conjunto)
                    .or_else(|| self.campos.as_ref().map(|c| c.programas[0].conjunto)),
                historial: self.historial.as_ref(),
                campos: self.campos.as_ref(),
                calculo: None,
            }),
            _ => None,
        })
    }

    pub(crate) fn preparar_procedural(
        &mut self,
        ctx: &Contexto,
        capacidad: u32,
        plan: PlanProcedural,
    ) -> Result<(), String> {
        esperar(ctx)?;
        self.pipeline = None;
        self.historial = None;
        self.campos = None;
        self.procedural = Some(Residente::nuevo(ctx, capacidad, plan, 2)?);
        self.malla = None;
        Ok(())
    }

    pub(crate) fn escena_procedural<'a>(
        &'a mut self,
        ctx: &Contexto,
        formato: u32,
        constantes: &'a [f32; 32],
        cuadro: usize,
        parametros: ParametrosNodos,
    ) -> Result<Escena<'a>, String> {
        // Valida antes de adquirir imagen o tocar sincronización de presentación.
        parametros.palabras(2)?;
        let _ = self.escena(ctx, formato, constantes)?;
        let pipeline = self.pipeline.as_ref().ok_or("falta el pipeline gráfico")?;
        let ranura =
            self.procedural.as_ref().and_then(|p| p.ranuras.get(cuadro)).ok_or("falta la ranura procedural")?;
        Ok(Escena {
            pipeline,
            malla: &ranura.malla,
            constantes,
            indirecto: Some(ranura.control.objeto),
            conjunto: Some(ranura.etapas[0].recursos.conjunto),
            historial: None,
            campos: None,
            calculo: Some((ranura, parametros)),
        })
    }

    pub(crate) fn preparar_historial(
        &mut self,
        ctx: &Contexto,
        shaders: Sombreadores,
        codigo: &[u32],
        tamano: [u32; 2],
    ) -> Result<(), String> {
        let nuevo = super::historial::Historial::nuevo(ctx, codigo, tamano)?;
        esperar(ctx)?;
        self.pipeline = None;
        self.procedural = None;
        self.vertices = shaders.0.to_vec();
        self.fragmentos = shaders.1.to_vec();
        self.historial = Some(nuevo);
        self.campos = None;
        Ok(())
    }
    pub(crate) fn inspeccionar_historial(&self, ctx: &Contexto) -> Result<Vec<[u32; 4]>, String> {
        self.historial.as_ref().ok_or("falta historial")?.inspeccionar(ctx)
    }
    pub(crate) fn capturar_historial(
        &mut self,
        ctx: &Contexto,
        tamano: (u32, u32),
        fondo: [f32; 4],
        bloque: &[f32; 32],
        capa: crate::CapaInterfaz,
    ) -> Result<crate::CapturaEstudio, String> {
        if self.historial.is_none() {
            return Err("falta historial".into());
        }
        self.capturar_residente(ctx, tamano, fondo, bloque, capa)
    }
    pub(crate) fn capturar_campos(
        &mut self,
        ctx: &Contexto,
        tamano: (u32, u32),
        fondo: [f32; 4],
        bloque: &[f32; 32],
        capa: crate::CapaInterfaz,
    ) -> Result<crate::CapturaEstudio, String> {
        if self.campos.is_none() {
            return Err("faltan campos".into());
        }
        self.capturar_residente(ctx, tamano, fondo, bloque, capa)
    }
    fn capturar_residente(
        &mut self,
        ctx: &Contexto,
        tamano: (u32, u32),
        fondo: [f32; 4],
        bloque: &[f32; 32],
        capa: crate::CapaInterfaz,
    ) -> Result<crate::CapturaEstudio, String> {
        if capa.area[0].checked_add(capa.area[2]).is_none_or(|v| v > tamano.0)
            || capa.area[1].checked_add(capa.area[3]).is_none_or(|v| v > tamano.1)
        {
            return Err("área fuera de captura".into());
        }
        let escena = self.escena(ctx, super::tipos_grafico::FORMATO_RGBA8, bloque)?.ok_or("falta malla de pantalla")?;
        let mut ui = super::interfaz::InterfazGpu::nueva(ctx, capa.shaders, super::tipos_grafico::FORMATO_RGBA8)?;
        if let Some(atlas) = capa.atlas {
            ui.fuente(ctx, atlas.ancho, atlas.alto, atlas.cobertura)?;
        }
        ui.actualizar(capa.vertices, capa.area)?;
        let dibujo = ui.juego(0)?;
        super::captura_interfaz::capturar(ctx, Some(&escena), &dibujo, tamano, fondo)
    }
    pub(crate) fn preparar_campos(
        &mut self,
        ctx: &Contexto,
        shaders: Sombreadores,
        plan: crate::PlanCampos,
    ) -> Result<(), String> {
        let nuevo = super::campos::Campos::nuevo(ctx, plan)?;
        esperar(ctx)?;
        self.pipeline = None;
        self.procedural = None;
        self.historial = None;
        self.vertices = shaders.0.to_vec();
        self.fragmentos = shaders.1.to_vec();
        self.campos = Some(nuevo);
        Ok(())
    }
    pub(crate) fn secuencia_campos(&mut self, nueva: &[usize]) -> Result<(), String> {
        self.campos.as_mut().ok_or("faltan campos")?.secuencia(nueva)
    }
    pub(crate) fn limpiar_campos(&self, ctx: &Contexto) -> Result<(), String> {
        self.campos.as_ref().ok_or("faltan campos")?.limpiar(ctx)
    }
    pub(crate) fn activar_campos(&mut self, activo: bool) -> Result<(), String> {
        self.campos.as_mut().ok_or("faltan campos")?.activo = activo;
        Ok(())
    }
    pub(crate) fn leer_campos(&self, ctx: &Contexto, desde: u32, cantidad: u32) -> Result<Vec<u32>, String> {
        self.campos.as_ref().ok_or("faltan campos")?.leer_rango(ctx, desde, cantidad)
    }
    pub(crate) fn ejecutar_campos(&self, ctx: &Contexto, bloque: &[f32; 32]) -> Result<(), String> {
        self.campos.as_ref().ok_or("faltan campos")?.ejecutar(ctx, bloque)
    }
    pub(crate) fn escribir_campos(&self, ctx: &Contexto, desde: u32, datos: &[u32]) -> Result<(), String> {
        self.campos.as_ref().ok_or("faltan campos")?.escribir_rango(ctx, desde, datos)
    }
    pub(crate) fn transferir_campos(&self, ctx: &Contexto, datos: Option<&[u32]>) -> Result<Vec<u32>, String> {
        self.campos.as_ref().ok_or("faltan campos")?.transferir(ctx, datos)
    }
    pub(crate) fn contar_paso(&mut self) {
        if let Some(procedural) = &mut self.procedural {
            procedural.contar_paso();
        }
    }

    pub(crate) fn actividad(&self) -> ActividadProcedural {
        self.procedural.as_ref().map_or(ActividadProcedural::default(), Residente::actividad)
    }
}

impl Drop for Visor {
    fn drop(&mut self) {
        // SAFETY: el dispositivo es válido (el contexto se suelta después); tras
        // esperar, el pipeline y la malla ya no están en uso y se sueltan solos.
        unsafe { (self.fd.esperar)(self.d) };
    }
}

fn esperar(ctx: &Contexto) -> Result<(), String> {
    let fd = ctx.fd.ok_or("el contexto no tiene dispositivo")?;
    // SAFETY: el dispositivo es válido; esperar no toca nada más.
    ok(unsafe { (fd.esperar)(ctx.dispositivo) }, "esperar a la GPU")
}

/// Dibuja la malla en una imagen RGBA8 (lineal, sin sRGB) de `tamano` y devuelve
/// sus píxeles, fila a fila desde arriba, 4 bytes cada uno.
pub(crate) fn capturar(
    ctx: &Contexto,
    sombreadores: Sombreadores,
    (vertices, indices): (&[[f32; 6]], &[u32]),
    (ancho, alto): (u32, u32),
    limpiar: [f32; 4],
    constantes: &[f32; 32],
) -> Result<Vec<u8>, String> {
    if ancho == 0 || alto == 0 || ancho > 4096 || alto > 4096 {
        return Err(format!("una captura de {ancho}×{alto} no vale (de 1 a 4096 por lado)"));
    }
    let pipeline = PipelineGrafico::nuevo(ctx, sombreadores.0, sombreadores.1, FORMATO_RGBA8)?;
    let malla = MallaGpu::nueva(ctx, vertices, indices)?;
    capturar_buffers(ctx, &pipeline, &malla, None, (ancho, alto), limpiar, constantes)
}

/// Captura de buffers ya residentes, con dibujo directo o indirecto.
pub(super) fn capturar_buffers(
    ctx: &Contexto,
    pipeline: &PipelineGrafico,
    malla: &MallaGpu,
    indirecto: Option<super::tipos::Objeto>,
    (ancho, alto): (u32, u32),
    limpiar: [f32; 4],
    constantes: &[f32; 32],
) -> Result<Vec<u8>, String> {
    if ancho == 0 || alto == 0 || ancho > 4096 || alto > 4096 {
        return Err("captura fuera de 1..=4096 por lado".to_owned());
    }
    let color =
        Imagen::nueva(ctx, FORMATO_RGBA8, (ancho, alto), USO_COLOR_ATTACHMENT | USO_TRANSFER_SRC, ASPECTO_COLOR)?;
    let adjuntos = super::multimuestreo::Adjuntos::nuevos(ctx, FORMATO_RGBA8, (ancho, alto))?;
    let bytes = ancho as usize * alto as usize * 4;
    let memoria = (MEMORIA_VISIBLE | MEMORIA_COHERENTE, MEMORIA_CON_CACHE);
    let lectura = Buffer::nuevo(ctx, bytes as u64, BUFFER_TRANSFER_DST, memoria)?;
    let destino = Destino {
        imagen: color.objeto,
        vista: color.vista,
        profundidad: (adjuntos.profundidad.objeto, adjuntos.profundidad.vista),
        multimuestreo: adjuntos.color(),
        extension: (ancho, alto),
        despues: (LAYOUT_TRANSFER_SRC, ETAPA_TRANSFER, ACCESO_TRANSFER_READ),
    };
    let escena =
        Escena { pipeline, malla, constantes, indirecto, conjunto: None, historial: None, campos: None, calculo: None };
    let copia = CopiaImagen {
        desplazamiento: 0,
        largo_fila: 0, // filas seguidas, sin relleno
        alto_imagen: 0,
        aspecto: ASPECTO_COLOR,
        mip: 0,
        capa_base: 0,
        n_capas: 1,
        origen: [0; 3],
        extension: Extension3D { ancho, alto, fondo: 1 },
    };
    // La copia tiene que verse desde la CPU al mapear el buffer.
    let barrera = BarreraMemoria {
        s_type: ST_MEMORY_BARRIER,
        p_next: core::ptr::null(),
        acceso_origen: ACCESO_TRANSFER_WRITE,
        acceso_destino: ACCESO_HOST_READ,
    };
    una_vez(ctx, |fd, cmd| {
        let nulo = core::ptr::null();
        // SAFETY: `cmd` está grabando; las imágenes, el pipeline, la malla y el buffer
        // viven hasta después de esperar a la GPU; el pipeline es RGBA8, como la imagen,
        // y el buffer mide justo una imagen.
        unsafe {
            grabar(fd, cmd, &destino, limpiar, Some(&escena), None);
            (fd.grafico.copiar_imagen)(cmd, color.objeto, LAYOUT_TRANSFER_SRC, lectura.objeto, 1, &raw const copia);
            let b = (&raw const barrera).cast();
            (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, b, 0, nulo, 0, nulo.cast());
        }
    })?;
    lectura.leer(bytes)
}
