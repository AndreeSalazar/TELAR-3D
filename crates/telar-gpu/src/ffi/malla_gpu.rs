//! Una malla en la memoria de la GPU: vértices (posición + normal) e índices de
//! triángulos, en buffers de memoria propia (la rápida). Se suben a través de un
//! buffer intermedio visible para la CPU, con una copia en la GPU.

use super::buffer::{Buffer, como_bytes, una_vez};
use super::contexto::Contexto;
use super::tipos::{ACCESO_TRANSFER_WRITE, ETAPA_TRANSFER};
use super::tipos_calculo::{BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia, ST_MEMORY_BARRIER};
use super::tipos_grafico::{
    ACCESO_LEER_ATRIBUTOS, ACCESO_LEER_INDICES, BUFFER_INDICES, BUFFER_VERTICES, ETAPA_ENTRADA_VERTICES,
};
use crate::eleccion::{MEMORIA_COHERENTE, MEMORIA_GPU, MEMORIA_VISIBLE};

/// Los dos buffers de una malla y cuántos índices dibujar.
pub(crate) struct MallaGpu {
    pub(crate) vertices: Buffer,
    pub(crate) indices: Buffer,
    pub(crate) n_indices: u32,
}

impl MallaGpu {
    /// Comprueba la malla (triángulos completos, índices dentro) y la sube.
    pub(crate) fn nueva(ctx: &Contexto, vertices: &[[f32; 6]], indices: &[u32]) -> Result<Self, String> {
        let n_indices = u32::try_from(indices.len()).map_err(|_| "la malla tiene demasiados índices")?;
        if indices.is_empty() || !indices.len().is_multiple_of(3) {
            return Err(format!("la malla tiene {} índices: hacen falta triángulos completos", indices.len()));
        }
        if let Some(&i) = indices.iter().find(|&&i| i as usize >= vertices.len()) {
            return Err(format!("el índice {i} no tiene vértice (la malla tiene {})", vertices.len()));
        }
        let (bv, bi) = (como_bytes(vertices), como_bytes(indices));
        let (nv, ni) = (bv.len() as u64, bi.len() as u64);
        let subida = Buffer::nuevo(ctx, nv + ni, BUFFER_TRANSFER_SRC, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, 0))?;
        subida.escribir(0, bv)?;
        subida.escribir(bv.len(), bi)?;
        let gpu = |tamano, uso| Buffer::nuevo(ctx, tamano, uso | BUFFER_TRANSFER_DST, (MEMORIA_GPU, 0));
        let malla = Self { vertices: gpu(nv, BUFFER_VERTICES)?, indices: gpu(ni, BUFFER_INDICES)?, n_indices };
        let copias = [Copia { origen: 0, destino: 0, tamano: nv }, Copia { origen: nv, destino: 0, tamano: ni }];
        // La copia tiene que verse al leer vértices e índices en los dibujos que vengan.
        let barrera = BarreraMemoria {
            s_type: ST_MEMORY_BARRIER,
            p_next: core::ptr::null(),
            acceso_origen: ACCESO_TRANSFER_WRITE,
            acceso_destino: ACCESO_LEER_ATRIBUTOS | ACCESO_LEER_INDICES,
        };
        una_vez(ctx, |fd, cmd| {
            let nulo = core::ptr::null();
            // SAFETY: `cmd` está grabando; los tres buffers existen hasta después de
            // esperar a la GPU, y cada copia cabe en su origen y su destino.
            unsafe {
                (fd.calculo.copiar_buffer)(cmd, subida.objeto, malla.vertices.objeto, 1, &raw const copias[0]);
                (fd.calculo.copiar_buffer)(cmd, subida.objeto, malla.indices.objeto, 1, &raw const copias[1]);
                let b = (&raw const barrera).cast();
                (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_ENTRADA_VERTICES, 0, 1, b, 0, nulo, 0, nulo.cast());
            }
        })?;
        Ok(malla)
    }
}
