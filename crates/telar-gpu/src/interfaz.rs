//! API segura para la interfaz 2D; los eventos y el diseño pertenecen a telar-ui.
use crate::{Constantes, Gpu, Procedural, Sombreadores, ffi};

/// Overlay para capturas diagnósticas de la escena completa.
#[derive(Clone, Copy)]
pub struct CapaInterfaz<'a> {
    /// Shaders 2D, independientes de los de la escena.
    pub shaders: Sombreadores<'a>,
    /// Triángulos de posición NDC y color sRGB.
    pub vertices: &'a [[f32; 6]],
    /// Área del visor en píxeles, dentro de la captura.
    pub area: [u32; 4],
}

impl Procedural<'_> {
    /// Captura el resultado GPU con la interfaz, sin ventanas ni descargas en vivo.
    ///
    /// # Errors
    /// Si no se ejecutó un paso, hay tamaños inválidos o Vulkan falla.
    pub fn capturar_estudio(
        &self,
        shaders: Sombreadores,
        tamano: (u32, u32),
        fondo: [f32; 4],
        constantes: &Constantes,
        interfaz: CapaInterfaz,
    ) -> Result<Vec<u8>, String> {
        self.interior.capturar_estudio(&self.calculo.contexto, shaders, tamano, fondo, constantes, interfaz)
    }
}

impl Gpu {
    /// Prepara el pipeline 2D y dos buffers mapeados, separados de los de compute.
    ///
    /// # Errors
    /// Si no hay formato de presentación o falla Vulkan.
    pub fn preparar_interfaz(&mut self, shaders: Sombreadores) -> Result<(), String> {
        let formato = self.presentacion.formato;
        if formato == 0 {
            return Err("no hay formato de presentación para la interfaz".to_owned());
        }
        self.interfaz = Some(ffi::InterfazGpu::nueva(&self.contexto, shaders, formato)?);
        Ok(())
    }

    /// Reemplaza triángulos UI y área del visor. Se llama al cambiar controles o
    /// tamaño, sin subir geometría procedural. Los colores son sRGB y las posiciones NDC.
    ///
    /// # Errors
    /// Si falta la interfaz, hay más de 120 000 vértices, valores no finitos o área inválida.
    pub fn actualizar_interfaz(&mut self, vertices: &[[f32; 6]], area: [u32; 4]) -> Result<(), String> {
        let (_, (ancho, alto)) = self.imagenes();
        if area[0].checked_add(area[2]).is_none_or(|fin| fin > ancho)
            || area[1].checked_add(area[3]).is_none_or(|fin| fin > alto)
        {
            return Err("el área del visor excede la imagen".to_owned());
        }
        self.interfaz.as_mut().ok_or("falta preparar la interfaz")?.actualizar(vertices, area)
    }
}
