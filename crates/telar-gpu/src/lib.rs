//! # telar-gpu
//!
//! La GPU de TELAR 3D: **Vulkan a mano**, sin crates, sin SDK enlazado
//! (`vulkan-1.dll` se carga al arrancar). **Aislada por completo:** no sabe
//! que existe `telar-ventana`; recibe la superficie como dos números
//! ([`Superficie`]) y quien la une con la ventana es `telar-app`.
//!
//! ```no_run
//! use telar_gpu::{Gpu, Opciones, Superficie};
//!
//! # let (hinstance, hwnd) = (0, 0);
//! let mut gpu = Gpu::nueva(&Opciones::default(), Superficie { instancia: hinstance, ventana: hwnd }, (1280, 720))?;
//! gpu.dibujar(telar_gpu::eleccion::hex(0x35e3ff))?;
//! # Ok::<(), String>(())
//! ```

pub mod eleccion;
mod ffi;

use core::sync::atomic::Ordering;

/// Cómo se crea la GPU.
#[derive(Debug, Clone)]
pub struct Opciones {
    /// Nombre de la aplicación (lo ven las herramientas de depuración de Vulkan).
    pub nombre: String,
    /// Activar las capas de validación si están instaladas (vienen con el SDK).
    pub validacion: bool,
}

impl Default for Opciones {
    fn default() -> Self {
        Self { nombre: "TELAR 3D".to_owned(), validacion: true }
    }
}

/// Los manejadores Win32 de la ventana donde se presenta, como números
/// (`HINSTANCE`, `HWND`). Así la GPU no depende del crate de la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Superficie {
    /// `HINSTANCE`.
    pub instancia: isize,
    /// `HWND`.
    pub ventana: isize,
}

/// La GPU con su ventana: contexto de Vulkan y presentación.
pub struct Gpu {
    // Orden de los campos = orden de destrucción: la presentación antes que el dispositivo.
    presentacion: ffi::Presentacion,
    contexto: ffi::Contexto,
}

impl Gpu {
    /// Crea la instancia (con validación si se pide), la superficie, elige la mejor
    /// GPU (la dedicada primero) y prepara la swapchain para `tamano`.
    ///
    /// # Errors
    ///
    /// Si no hay Vulkan, ninguna GPU puede presentar en la ventana o Vulkan falla.
    pub fn nueva(opciones: &Opciones, superficie: Superficie, tamano: (u32, u32)) -> Result<Self, String> {
        let contexto = ffi::Contexto::nuevo(
            &opciones.nombre,
            opciones.validacion,
            Some((superficie.instancia, superficie.ventana)),
        )?;
        let presentacion = ffi::Presentacion::nueva(&contexto, tamano)?;
        Ok(Self { presentacion, contexto })
    }

    /// El nombre de la GPU elegida.
    #[must_use]
    pub fn dispositivo(&self) -> &str {
        &self.contexto.nombre
    }

    /// ¿Están activas las capas de validación?
    #[must_use]
    pub fn validacion(&self) -> bool {
        self.contexto.validacion
    }

    /// El formato y el tamaño actuales de las imágenes.
    #[must_use]
    pub fn imagenes(&self) -> (u32, (u32, u32)) {
        (self.presentacion.formato, self.presentacion.extension)
    }

    /// Cuántas veces se ha creado la swapchain: 1 al empezar, y una más cada vez
    /// que se rehace (al cambiar de tamaño o al volver de minimizada).
    #[must_use]
    pub fn swapchains_creadas(&self) -> u32 {
        self.presentacion.creaciones
    }

    /// La ventana cambió de tamaño (`0 × 0` si se minimizó).
    pub fn redimensionar(&mut self, ancho: u32, alto: u32) {
        self.presentacion.redimensionar((ancho, alto));
    }

    /// Limpia la pantalla a `color` (sRGB, como en CSS) y la presenta, sincronizada
    /// con el monitor. `Ok(false)` si no tocaba dibujar (minimizada o rehaciendo).
    ///
    /// # Errors
    ///
    /// Si Vulkan falla (no por cambiar de tamaño: eso se rehace solo).
    pub fn dibujar(&mut self, color: [f32; 4]) -> Result<bool, String> {
        let color = eleccion::color_para(self.presentacion.formato, color);
        self.presentacion.dibujar(&self.contexto, color)
    }
}

/// Un contexto sin ventana: instancia y dispositivo, para calcular (la Fase 2)
/// y para los tests.
pub struct Calculo {
    contexto: ffi::Contexto,
}

impl Calculo {
    /// Crea la instancia y el dispositivo, sin superficie.
    ///
    /// # Errors
    ///
    /// Si no hay Vulkan o ninguna GPU tiene una cola de gráficos.
    pub fn nuevo(opciones: &Opciones) -> Result<Self, String> {
        Ok(Self { contexto: ffi::Contexto::nuevo(&opciones.nombre, opciones.validacion, None)? })
    }

    /// El nombre de la GPU elegida.
    #[must_use]
    pub fn dispositivo(&self) -> &str {
        &self.contexto.nombre
    }

    /// ¿Están activas las capas de validación?
    #[must_use]
    pub fn validacion(&self) -> bool {
        self.contexto.validacion
    }
}

/// Los avisos y errores de las capas de validación en todo el proceso. La regla
/// de la Fase 1: tiene que ser **cero**.
#[must_use]
pub fn avisos_de_validacion() -> u32 {
    ffi::AVISOS.load(Ordering::Relaxed)
}
