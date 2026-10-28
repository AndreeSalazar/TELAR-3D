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

mod calidad;
pub mod eleccion;
mod escena;
mod ffi;
mod historial;
mod interfaz;
mod lotes;
pub use lotes::{PlanLotes, ProgresoLotes};
mod perfil;
mod plan;
mod procedural;

use core::sync::atomic::Ordering;

pub use escena::{Constantes, Sombreadores};
pub use interfaz::{AtlasInterfaz, CapaInterfaz, CapturaEstudio};
pub use perfil::PerfilGpu;
pub use plan::{DominioProcedural, EtapaProcedural, PlanProcedural};
pub use procedural::{
    ActividadProcedural, DiagnosticoProcedural, KernelsProcedurales, PALABRAS_CONTROL_PROCEDURAL, ParametrosNodos,
    ParametrosProcedurales, Procedural,
};

/// Bytes de [`Constantes`] en la GPU (8 `vec4`).
pub const TAMANO_CONSTANTES: u32 = ffi::TAMANO_CONSTANTES;
/// El formato de las capturas: `VK_FORMAT_R8G8B8A8_UNORM` (sin sRGB).
pub const FORMATO_CAPTURA: u32 = 37;

/// Cómo se crea la GPU.
#[derive(Debug, Clone)]
pub struct Opciones {
    /// Nombre de la aplicación (lo ven las herramientas de depuración de Vulkan).
    pub nombre: String,
    /// Activar las capas de validación si están instaladas (vienen con el SDK).
    pub validacion: bool,
    /// MSAA según soporte real y máximo solicitado; false conserva 1x.
    pub antialias: bool,
    /// Máximo solicitado: potencia de dos entre 1 y 64; antialias=false fuerza 1x.
    pub muestras: u32,
}

impl Default for Opciones {
    fn default() -> Self {
        Self { nombre: "TELAR 3D".to_owned(), validacion: true, antialias: true, muestras: 4 }
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

/// La GPU con su ventana: contexto de Vulkan, presentación y, si se prepara, el visor.
pub struct Gpu {
    // Orden de los campos = orden de destrucción: el visor y la presentación antes
    // que el dispositivo.
    interfaz: Option<ffi::InterfazGpu>,
    visor: Option<ffi::Visor>,
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
        calidad::validar_muestras(opciones.muestras)?;
        let mut contexto = ffi::Contexto::nuevo(
            &opciones.nombre,
            opciones.validacion,
            Some((superficie.instancia, superficie.ventana)),
        )?;
        contexto.antialias = opciones.antialias;
        contexto.muestras_solicitadas = opciones.muestras;
        let presentacion = ffi::Presentacion::nueva(&contexto, tamano)?;
        Ok(Self { interfaz: None, visor: None, presentacion, contexto })
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

    /// Muestras compatibles de color/profundidad para el formato actual.
    /// # Errors
    /// Si falla la consulta de capacidades del dispositivo.
    pub fn muestras(&self) -> Result<u32, String> {
        self.contexto.muestras(self.presentacion.formato)
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
        self.presentacion.dibujar(&self.contexto, color, None, None)
    }

    /// Prepara el visor con estos shaders (o los cambia, si ya estaba). El
    /// pipeline se hace en el primer dibujo, para el formato de la ventana.
    ///
    /// # Errors
    ///
    /// Si Vulkan falla al esperar a la GPU.
    pub fn preparar_visor(&mut self, sombreadores: Sombreadores) -> Result<(), String> {
        let s = (sombreadores.vertices, sombreadores.fragmentos);
        if let Some(visor) = &mut self.visor {
            return visor.cambiar(&self.contexto, s);
        }
        self.visor = Some(ffi::Visor::nuevo(&self.contexto, s)?);
        Ok(())
    }

    /// Sube una malla al visor (en lugar de la que hubiera): `vertices` =
    /// posición + normal, `indices` = triángulos (de tres en tres).
    ///
    /// # Errors
    ///
    /// Si no se preparó el visor, la malla está mal (índices fuera, triángulos
    /// incompletos) o no cabe en la GPU.
    pub fn subir_malla(&mut self, vertices: &[[f32; 6]], indices: &[u32]) -> Result<(), String> {
        let visor = self.visor.as_mut().ok_or("antes de subir una malla hay que preparar el visor")?;
        visor.subir(&self.contexto, vertices, indices)
    }

    /// Limpia a `fondo` (sRGB), dibuja la malla del visor con `constantes` y lo
    /// presenta. Sin visor o sin malla, solo limpia. `Ok(false)` si no tocaba dibujar.
    ///
    /// # Errors
    ///
    /// Si Vulkan falla o rechaza los shaders.
    pub fn dibujar_escena(&mut self, fondo: [f32; 4], constantes: &Constantes) -> Result<bool, String> {
        let formato = self.presentacion.formato;
        let plana = constantes.plana(formato);
        let escena = match self.visor.as_mut() {
            Some(visor) if formato != 0 => visor.escena(&self.contexto, formato, &plana)?,
            _ => None,
        };
        self.presentacion.dibujar(&self.contexto, eleccion::color_para(formato, fondo), escena.as_ref(), None)
    }

    /// Dibuja una malla no procedural con overlay y viewport de interfaz.
    /// El bloque es opaco para Vulkan; no interpreta cámara ni materiales.
    /// # Errors
    /// Rechaza constantes no finitas, visor ausente o errores del backend.
    pub fn dibujar_bloque_interfaz(&mut self, fondo: [f32; 4], bloque: &[f32; 32]) -> Result<bool, String> {
        if !bloque.iter().all(|v| v.is_finite()) {
            return Err("constantes no finitas".into());
        }
        let formato = self.presentacion.formato;
        if let Some(interfaz) = &mut self.interfaz {
            interfaz.formato(&self.contexto, formato)?;
        }
        let visor = self.visor.as_mut().ok_or("falta el visor")?;
        let escena = visor.escena(&self.contexto, formato, bloque)?;
        self.presentacion.dibujar(
            &self.contexto,
            eleccion::color_para(formato, fondo),
            escena.as_ref(),
            self.interfaz.as_mut(),
        )
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
        calidad::validar_muestras(opciones.muestras)?;
        let mut contexto = ffi::Contexto::nuevo(&opciones.nombre, opciones.validacion, None)?;
        contexto.antialias = opciones.antialias;
        contexto.muestras_solicitadas = opciones.muestras;
        Ok(Self { contexto })
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

    /// ¿Puede medir con los temporizadores de la GPU (su cola los tiene)?
    #[must_use]
    pub fn mide_tiempos(&self) -> bool {
        self.contexto.bits_tiempo > 0
    }

    /// Ejecuta un kernel SPIR-V (entrada `main`; un buffer de `u32` en el conjunto 0,
    /// enlace 0) sobre `elementos` valores, en grupos de `grupo` hilos, y devuelve
    /// el buffer y lo que tardó el kernel según la GPU.
    ///
    /// # Errors
    ///
    /// Si la GPU no admite tantos grupos, rechaza el SPIR-V o Vulkan falla.
    pub fn ejecutar(&self, spirv: &[u32], elementos: u32, grupo: u32) -> Result<Resultado, String> {
        let e = ffi::ejecutar(&self.contexto, spirv, elementos, grupo)?;
        Ok(Resultado { datos: e.datos, nanosegundos: e.nanosegundos })
    }

    /// Dibuja una malla como el visor, pero en una imagen fuera de pantalla de
    /// `tamano` ([`FORMATO_CAPTURA`]), y devuelve sus píxeles RGBA, fila a fila
    /// desde arriba. Para comprobar el dibujo sin ventana.
    ///
    /// # Errors
    ///
    /// Si el tamaño no vale (de 1 a 4096 por lado), la malla está mal o Vulkan falla.
    pub fn capturar(
        &self,
        sombreadores: Sombreadores,
        vertices: &[[f32; 6]],
        indices: &[u32],
        tamano: (u32, u32),
        fondo: [f32; 4],
        constantes: &Constantes,
    ) -> Result<Vec<u8>, String> {
        let s = (sombreadores.vertices, sombreadores.fragmentos);
        let plana = constantes.plana(FORMATO_CAPTURA);
        ffi::capturar(&self.contexto, s, (vertices, indices), tamano, fondo, &plana)
    }
}

/// Lo que devuelve un kernel.
#[derive(Debug, Clone)]
pub struct Resultado {
    /// El buffer, ya en la CPU.
    pub datos: Vec<u32>,
    /// Lo que tardó **solo el kernel**, medido por la GPU (sin la copia ni el envío).
    /// `None` si la cola no tiene temporizadores.
    pub nanosegundos: Option<f64>,
}

/// Los avisos y errores de las capas de validación en todo el proceso. La regla
/// de la Fase 1: tiene que ser **cero**.
#[must_use]
pub fn avisos_de_validacion() -> u32 {
    ffi::AVISOS.load(Ordering::Relaxed)
}
