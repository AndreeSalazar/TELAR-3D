//! MSAA nativo: capacidades consultadas por formato y adjuntos persistentes.
use super::{
    contexto::{Contexto, ok},
    imagen::Imagen,
    tipos::USO_COLOR_ATTACHMENT,
    tipos_grafico::{ASPECTO_PROFUNDIDAD, Extension3D, FORMATO_PROFUNDIDAD, USO_PROFUNDIDAD},
};

/// `VkImageFormatProperties`, 32 bytes, alineación de ocho; sampleCounts en byte 20.
#[repr(C)]
pub(crate) struct PropiedadesImagen {
    extension: Extension3D,
    mips: u32,
    capas: u32,
    muestras: u32,
    tamano: u64,
}

const _: () = assert!(core::mem::size_of::<PropiedadesImagen>() == 32);
const _: () = assert!(core::mem::offset_of!(PropiedadesImagen, muestras) == 20);

impl Contexto {
    pub(crate) fn muestras(&self, formato: u32) -> Result<u32, String> {
        if !self.antialias || self.muestras_solicitadas == 1 {
            return Ok(1);
        }
        let existente = self.formatos_muestras.borrow().iter().find(|(f, _)| *f == formato).map(|(_, m)| *m);
        let disponibles = if let Some(mascara) = existente {
            mascara
        } else {
            let mascara = self.consultar_muestras(formato)?;
            self.formatos_muestras.borrow_mut().push((formato, mascara));
            mascara
        };
        Ok([4, 2, 1].into_iter().find(|m| *m <= self.muestras_solicitadas && disponibles & m != 0).unwrap_or(1))
    }

    fn consultar_muestras(&self, formato: u32) -> Result<u32, String> {
        let consultar = |formato, uso| {
            let mut propiedades = PropiedadesImagen {
                extension: Extension3D { ancho: 0, alto: 0, fondo: 0 },
                mips: 0,
                capas: 0,
                muestras: 0,
                tamano: 0,
            };
            let resultado = {
                // SAFETY: físico válido, imagen 2D óptima, estructura C de salida completa.
                unsafe { (self.fi.propiedades_imagen)(self.fisico, formato, 1, 0, uso, 0, &raw mut propiedades) }
            };
            ok(resultado, "consultar muestras de imagen")?;
            Ok::<_, String>(propiedades.muestras)
        };
        Ok(consultar(formato, USO_COLOR_ATTACHMENT)? & consultar(FORMATO_PROFUNDIDAD, USO_PROFUNDIDAD)?)
    }
}

pub(crate) struct Adjuntos {
    pub(crate) profundidad: Imagen,
    pub(crate) color: Option<Imagen>,
    pub(crate) muestras: u32,
}

impl Adjuntos {
    pub(crate) fn nuevos(ctx: &Contexto, formato: u32, tamano: (u32, u32)) -> Result<Self, String> {
        let muestras = ctx.muestras(formato)?;
        let profundidad =
            Imagen::con_muestras(ctx, FORMATO_PROFUNDIDAD, tamano, USO_PROFUNDIDAD, ASPECTO_PROFUNDIDAD, muestras)?;
        let color = if muestras > 1 {
            Some(Imagen::con_muestras(
                ctx,
                formato,
                tamano,
                USO_COLOR_ATTACHMENT,
                super::tipos::ASPECTO_COLOR,
                muestras,
            )?)
        } else {
            None
        };
        Ok(Self { profundidad, color, muestras })
    }

    pub(crate) fn color(&self) -> Option<(super::tipos::Objeto, super::tipos::Objeto)> {
        self.color.as_ref().map(|color| (color.objeto, color.vista))
    }
}
