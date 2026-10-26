//! # telar-ventana
//!
//! La ventana de TELAR 3D, con Win32 a mano y sin crates. **Aislada por
//! completo:** no sabe que existe Vulkan ni ningún otro crate de TELAR. Da sus
//! manejadores nativos como números ([`Manejadores`]); quien los une con la GPU
//! es `telar-app`.
//!
//! ```no_run
//! use telar_ventana::{Config, Evento, Ventana};
//!
//! let mut ventana = Ventana::nueva(&Config::default())?;
//! 'principal: loop {
//!     for evento in ventana.eventos() {
//!         if evento == Evento::Cerrar {
//!             break 'principal;
//!         }
//!     }
//! }
//! # Ok::<(), String>(())
//! ```

pub mod evento;
mod ffi;

use core::marker::PhantomData;

pub use evento::{Evento, Tecla};

/// Cómo se abre la ventana.
#[derive(Debug, Clone)]
pub struct Config {
    /// El texto de la barra de título.
    pub titulo: String,
    /// Ancho del área de dibujo, en píxeles lógicos (se escala con el DPI).
    pub ancho: u32,
    /// Alto del área de dibujo, en píxeles lógicos.
    pub alto: u32,
    /// `false` para crearla oculta (los tests).
    pub visible: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { titulo: "TELAR 3D".to_owned(), ancho: 1280, alto: 720, visible: true }
    }
}

/// Los manejadores nativos, como números: lo único que la GPU necesita saber
/// de la ventana para crear su superficie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Manejadores {
    /// `HINSTANCE` del módulo.
    pub instancia: isize,
    /// `HWND` de la ventana.
    pub ventana: isize,
}

/// Una ventana de Windows. Es del hilo que la crea (como exige Win32), así que
/// no se puede mandar a otro hilo.
pub struct Ventana {
    manejadores: Manejadores,
    _solo_este_hilo: PhantomData<*const ()>,
}

impl Ventana {
    /// Abre la ventana, centrada, con la barra de título oscura y nítida en
    /// cada monitor.
    ///
    /// # Errors
    ///
    /// Si Windows no deja registrar la clase o crear la ventana.
    pub fn nueva(config: &Config) -> Result<Self, String> {
        let (instancia, ventana) = ffi::crear(&config.titulo, config.ancho, config.alto, config.visible)?;
        Ok(Self { manejadores: Manejadores { instancia, ventana }, _solo_este_hilo: PhantomData })
    }

    /// Atiende los mensajes pendientes y devuelve los eventos desde la última llamada.
    pub fn eventos(&mut self) -> Vec<Evento> {
        ffi::bombear()
    }

    /// Tamaño actual del área de dibujo, en píxeles físicos.
    #[must_use]
    pub fn tamano(&self) -> (u32, u32) {
        ffi::tamano(self.manejadores.ventana)
    }

    /// Los manejadores nativos, para crear la superficie de la GPU.
    #[must_use]
    pub fn manejadores(&self) -> Manejadores {
        self.manejadores
    }

    /// Cambia el tamaño del área de dibujo (píxeles físicos), sin moverla. Llega
    /// como [`Evento::Tamano`] en la próxima llamada a [`Ventana::eventos`].
    pub fn cambiar_tamano(&self, ancho: u32, alto: u32) {
        ffi::cambiar_tamano(self.manejadores.ventana, ancho, alto);
    }

    /// Cambia el título (por ejemplo, para enseñar los fps).
    pub fn poner_titulo(&self, titulo: &str) {
        ffi::poner_titulo(self.manejadores.ventana, titulo);
    }
}

impl Drop for Ventana {
    fn drop(&mut self) {
        ffi::destruir(self.manejadores.ventana);
    }
}
