//! Overlay 2D con dos buffers visibles y mapeados. Solo se reescriben al cambiar
//! la interfaz; nunca contienen geometría procedural ni resultados de compute.
use super::{
    buffer::{Buffer, BufferMapeado, como_bytes},
    contexto::{Contexto, ok},
    funciones::FnDispositivo,
    pipeline_grafico::PipelineGrafico,
    tipos::Disp,
    tipos_grafico::BUFFER_VERTICES,
};
use crate::{
    Sombreadores,
    eleccion::{MEMORIA_COHERENTE, MEMORIA_VISIBLE, color_para},
};

const CAPACIDAD: usize = 120_000;

pub(crate) struct InterfazGpu {
    fd: FnDispositivo,
    dispositivo: Disp,
    pipeline: PipelineGrafico,
    fuente: super::fuente_gpu::FuenteGpu,
    shaders: [Vec<u32>; 2],
    juegos: Vec<BufferMapeado>,
    versiones: [Option<u64>; 2],
    revision: u64,
    vertices: Vec<[f32; 8]>,
    area: [u32; 4],
}

pub(crate) struct DibujoInterfaz<'a> {
    pub(crate) pipeline: &'a PipelineGrafico,
    pub(crate) buffer: &'a BufferMapeado,
    pub(crate) cantidad: u32,
    pub(crate) area: [u32; 4],
    pub(crate) conjunto: super::tipos::Objeto,
}

impl InterfazGpu {
    pub(crate) fn nueva(ctx: &Contexto, shaders: Sombreadores, formato: u32) -> Result<Self, String> {
        let fd = ctx.fd.ok_or("falta el dispositivo")?;
        let fuente = super::fuente_gpu::FuenteGpu::nueva(ctx)?;
        let pipeline =
            PipelineGrafico::interfaz(ctx, shaders.vertices, shaders.fragmentos, formato, fuente.disposicion())?;
        let mut juegos = Vec::new();
        for _ in 0..2 {
            juegos.push(
                Buffer::nuevo(ctx, (CAPACIDAD * 32) as u64, BUFFER_VERTICES, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?
                    .mapear_persistente()?,
            );
        }
        Ok(Self {
            fd,
            dispositivo: ctx.dispositivo,
            pipeline,
            fuente,
            shaders: [shaders.vertices.to_vec(), shaders.fragmentos.to_vec()],
            juegos,
            versiones: [None; 2],
            revision: 0,
            vertices: Vec::new(),
            area: [0, 0, 1, 1],
        })
    }

    pub(crate) fn formato(&mut self, ctx: &Contexto, formato: u32) -> Result<(), String> {
        if self.pipeline.formato != formato {
            // SAFETY: dispositivo propio; se espera antes de reemplazar el pipeline en vuelo.
            ok(unsafe { (self.fd.esperar)(self.dispositivo) }, "esperar interfaz")?;
            self.pipeline =
                PipelineGrafico::interfaz(ctx, &self.shaders[0], &self.shaders[1], formato, self.fuente.disposicion())?;
            self.versiones = [None; 2];
        }
        Ok(())
    }

    pub(crate) fn actualizar(&mut self, vertices: &[[f32; 8]], area: [u32; 4]) -> Result<(), String> {
        if vertices.len() > CAPACIDAD
            || !vertices.len().is_multiple_of(3)
            || !vertices.iter().flatten().all(|v| v.is_finite())
            || area[2] == 0
            || area[3] == 0
        {
            return Err("triángulos o área de interfaz inválidos".to_owned());
        }
        self.vertices.clear();
        self.vertices.extend_from_slice(vertices);
        self.area = area;
        self.revision = self.revision.wrapping_add(1);
        self.versiones = [None; 2];
        Ok(())
    }

    pub(crate) fn fuente(&mut self, ctx: &Contexto, ancho: u32, alto: u32, cobertura: &[u8]) -> Result<(), String> {
        self.fuente.actualizar(ctx, ancho, alto, cobertura)
    }

    /// Se llama tras la valla del juego, antes del envío que publica la escritura host.
    pub(crate) fn juego(&mut self, indice: usize) -> Result<DibujoInterfaz<'_>, String> {
        if self.versiones[indice] != Some(self.revision) {
            let mut vertices = self.vertices.clone();
            let mut anterior = None;
            let mut lineal = [0.0; 4];
            for vertice in &mut vertices {
                let bits = [vertice[4].to_bits(), vertice[5].to_bits(), vertice[6].to_bits()];
                if anterior != Some(bits) {
                    lineal = color_para(self.pipeline.formato, [vertice[4], vertice[5], vertice[6], 1.0]);
                    anterior = Some(bits);
                }
                vertice[4..7].copy_from_slice(&lineal[..3]);
            }
            self.juegos[indice].escribir(como_bytes(&vertices))?;
            self.versiones[indice] = Some(self.revision);
        }
        Ok(DibujoInterfaz {
            pipeline: &self.pipeline,
            buffer: &self.juegos[indice],
            cantidad: u32::try_from(self.vertices.len()).map_err(|_| "demasiados vértices UI")?,
            area: self.area,
            conjunto: self.fuente.recursos.conjunto,
        })
    }
}

impl Drop for InterfazGpu {
    fn drop(&mut self) {
        // SAFETY: el contexto vive más; tras esperar, se pueden desmapear y liberar
        // los dos buffers y el pipeline usados en los fotogramas en vuelo.
        unsafe {
            (self.fd.esperar)(self.dispositivo);
        }
    }
}
