//! El contexto de Vulkan: instancia (con las capas de validación y un mensajero
//! que cuenta cada aviso), superficie Win32, elección del dispositivo y su cola.

use core::ffi::{CStr, c_void};
use core::sync::atomic::{AtomicU32, Ordering};

use super::carga::{FnInstancia, Global};
use super::funciones::FnDispositivo;
use super::tipos::{
    CAPA_VALIDACION, COLA_CALCULO, COLA_GRAFICOS, Cola, DatosAviso, Disp, EXT_DEBUG_UTILS, EXT_SURFACE, EXT_SWAPCHAIN,
    EXT_WIN32_SURFACE, Fisico, GRAVEDAD_AVISO, GRAVEDAD_ERROR, InfoApp, InfoCola, InfoDispositivo, InfoInstancia,
    InfoMensajero, InfoSuperficieWin32, Instancia, Objeto, PropiedadesCapa, PropiedadesFamilia, PropiedadesFisico,
    ST_APPLICATION_INFO, ST_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT, ST_DEVICE_CREATE_INFO, ST_DEVICE_QUEUE_CREATE_INFO,
    ST_INSTANCE_CREATE_INFO, ST_WIN32_SURFACE_CREATE_INFO_KHR, TIPO_GENERAL, TIPOS_DE_MENSAJE, VK_API_1_3, VK_SUCCESS,
};
use super::tipos_calculo::{Monton, PropiedadesMemoria, TipoMemoria};
use super::tipos_grafico::{Funciones13, RENDERIZADO_DINAMICO, ST_VULKAN_1_3_FEATURES};
use crate::eleccion::puntuar;

/// Avisos y errores de las capas de validación en todo el proceso.
pub(crate) static AVISOS: AtomicU32 = AtomicU32::new(0);

/// Lo llama Vulkan con cada aviso: lo enseña, y lo cuenta si es sobre nuestro uso
/// de Vulkan (validación o rendimiento). Los de tipo GENERAL son del cargador y
/// del sistema (por ejemplo, capas que otros programas instalan en todas las apps
/// Vulkan, como el overlay de GOG Galaxy): se enseñan aparte y no cuentan.
extern "system" fn aviso(gravedad: u32, tipos: u32, datos: *const DatosAviso, _usuario: *mut c_void) -> u32 {
    let del_sistema = tipos == TIPO_GENERAL;
    if gravedad >= GRAVEDAD_AVISO && !del_sistema {
        AVISOS.fetch_add(1, Ordering::Relaxed);
    }
    let texto = if datos.is_null() {
        String::new()
    } else {
        // SAFETY: Vulkan pasa datos válidos durante la llamada, y `mensaje` es una
        // cadena C terminada en cero (o null, que se comprueba).
        unsafe {
            let m = (*datos).mensaje;
            if m.is_null() { String::new() } else { CStr::from_ptr(m).to_string_lossy().into_owned() }
        }
    };
    if del_sistema {
        eprintln!("[vulkan · sistema, no cuenta] {texto}");
    } else {
        eprintln!("[vulkan] {texto}");
    }
    0 // VK_FALSE: la llamada que lo provocó sigue su curso
}

/// Comprueba un `VkResult`.
pub(crate) fn ok(resultado: i32, que: &str) -> Result<(), String> {
    if resultado == VK_SUCCESS { Ok(()) } else { Err(format!("Vulkan: {que} falló (VkResult {resultado})")) }
}

/// Instancia, dispositivo y cola. Se destruye en orden inverso (Drop).
pub(crate) struct Contexto {
    pub(crate) antialias: bool,
    pub(crate) muestras_solicitadas: u32,
    pub(crate) formatos_muestras: std::cell::RefCell<Vec<(u32, u32)>>,
    pub(crate) fi: FnInstancia,
    pub(crate) fd: Option<FnDispositivo>,
    pub(crate) instancia: Instancia,
    pub(crate) mensajero: Objeto,
    pub(crate) superficie: Objeto,
    pub(crate) fisico: Fisico,
    pub(crate) dispositivo: Disp,
    pub(crate) cola: Cola,
    pub(crate) familia: u32,
    pub(crate) nombre: String,
    pub(crate) validacion: bool,
    /// Nanosegundos por tic del temporizador de la GPU (`timestampPeriod`).
    pub(crate) periodo_ns: f32,
    /// Bits válidos de los tiempos en la familia elegida (0 = sin temporizador).
    pub(crate) bits_tiempo: u32,
    /// Grupos que se pueden lanzar en X de una vez.
    pub(crate) grupos_max_x: u32,
    pub(crate) perfil: crate::PerfilGpu,
    /// Las propiedades de cada tipo de memoria (`VkMemoryType::propertyFlags`).
    pub(crate) tipos_memoria: Vec<u32>,
}

/// Lo que se recuerda de la mejor GPU mientras se buscan.
struct Candidata {
    puntos: u32,
    fisico: Fisico,
    familia: u32,
    nombre: String,
    periodo_ns: f32,
    bits_tiempo: u32,
    grupos_max_x: u32,
    perfil: crate::PerfilGpu,
}

impl Contexto {
    /// Crea todo. `ventana` son los manejadores Win32 `(HINSTANCE, HWND)`; sin
    /// ventana, el contexto sirve para calcular (y para los tests).
    pub(crate) fn nuevo(nombre_app: &str, validacion: bool, ventana: Option<(isize, isize)>) -> Result<Self, String> {
        let global = Global::cargar()?;
        let validacion = validacion && hay_validacion(&global);
        let instancia = crear_instancia(&global, nombre_app, validacion)?;
        let fi = FnInstancia::cargar(&global, instancia, validacion)?;
        // A partir de aquí, cualquier error suelta lo creado en Drop.
        let mut ctx = Self {
            antialias: true,
            muestras_solicitadas: 4,
            formatos_muestras: std::cell::RefCell::new(Vec::new()),
            fi,
            fd: None,
            instancia,
            mensajero: 0,
            superficie: 0,
            fisico: core::ptr::null_mut(),
            dispositivo: core::ptr::null_mut(),
            cola: core::ptr::null_mut(),
            familia: 0,
            nombre: String::new(),
            validacion,
            periodo_ns: 0.0,
            bits_tiempo: 0,
            grupos_max_x: 0,
            perfil: crate::PerfilGpu::default(),
            tipos_memoria: Vec::new(),
        };
        if let Some((crear, _)) = fi.mensajero {
            let info = info_mensajero();
            // SAFETY: instancia válida; `info` vive durante la llamada; el mensajero
            // se guarda para destruirlo antes que la instancia.
            let r = unsafe { crear(instancia, &raw const info, core::ptr::null(), &raw mut ctx.mensajero) };
            ok(r, "crear el mensajero")?;
        }
        if let Some((hinstance, hwnd)) = ventana {
            let info = InfoSuperficieWin32 {
                s_type: ST_WIN32_SURFACE_CREATE_INFO_KHR,
                p_next: core::ptr::null(),
                flags: 0,
                hinstance,
                hwnd,
            };
            // SAFETY: los manejadores son de una ventana viva de este proceso (los da
            // telar-ventana); `info` vive durante la llamada.
            let r = unsafe {
                (fi.crear_superficie)(instancia, &raw const info, core::ptr::null(), &raw mut ctx.superficie)
            };
            ok(r, "crear la superficie")?;
        }
        ctx.elegir_fisico()?;
        ctx.crear_dispositivo()?;
        Ok(ctx)
    }

    /// El mejor dispositivo con una cola de gráficos (que, si hay ventana, sepa presentar).
    fn elegir_fisico(&mut self) -> Result<(), String> {
        let mut n = 0u32;
        // SAFETY: primero se pide el número y luego se rellena un Vec de ese tamaño.
        let fisicos = unsafe {
            ok((self.fi.fisicos)(self.instancia, &raw mut n, core::ptr::null_mut()), "contar las GPU")?;
            let mut v = vec![core::ptr::null_mut(); n as usize];
            ok((self.fi.fisicos)(self.instancia, &raw mut n, v.as_mut_ptr()), "listar las GPU")?;
            v.truncate(n as usize);
            v
        };
        let mut mejor: Option<Candidata> = None;
        for fisico in fisicos {
            let mut p = PropiedadesFisico {
                version_api: 0,
                version_driver: 0,
                fabricante: 0,
                id: 0,
                tipo: 0,
                nombre: [0; 256],
                resto: [0; 1024],
            };
            let mut nf = 0u32;
            // SAFETY: `p` tiene sitio de sobra para VkPhysicalDeviceProperties (824 B);
            // las familias se piden en dos pasos, como las GPU.
            let familias = unsafe {
                (self.fi.propiedades)(fisico, &raw mut p);
                (self.fi.familias)(fisico, &raw mut nf, core::ptr::null_mut());
                let mut v = vec![PropiedadesFamilia::default(); nf as usize];
                (self.fi.familias)(fisico, &raw mut nf, v.as_mut_ptr());
                v
            };
            let Some(familia) = (0u32..)
                .zip(&familias)
                .find(|&(i, f)| {
                    f.flags & (COLA_GRAFICOS | COLA_CALCULO) == (COLA_GRAFICOS | COLA_CALCULO)
                        && self.presenta(fisico, i)
                })
                .map(|(i, _)| i)
            else {
                continue;
            };
            let puntos = puntuar(p.tipo);
            if mejor.as_ref().is_none_or(|m| puntos > m.puntos) {
                let fin = p.nombre.iter().position(|&b| b == 0).unwrap_or(p.nombre.len());
                mejor = Some(Candidata {
                    puntos,
                    fisico,
                    familia,
                    nombre: String::from_utf8_lossy(&p.nombre[..fin]).into_owned(),
                    periodo_ns: p.periodo_tiempo(),
                    bits_tiempo: familias[familia as usize].bits_tiempo,
                    grupos_max_x: p.grupos_max_x(),
                    perfil: p.perfil(),
                });
            }
        }
        let c = mejor.ok_or("ninguna GPU tiene una cola de gráficos que pueda presentar en la ventana")?;
        (self.fisico, self.familia, self.nombre) = (c.fisico, c.familia, c.nombre);
        (self.periodo_ns, self.bits_tiempo, self.grupos_max_x) = (c.periodo_ns, c.bits_tiempo, c.grupos_max_x);
        self.perfil = c.perfil;
        let mut memoria = PropiedadesMemoria {
            n_tipos: 0,
            tipos: [TipoMemoria::default(); 32],
            n_montones: 0,
            montones: [Monton::default(); 16],
        };
        // SAFETY: el físico es válido y `memoria` tiene la forma exacta de
        // VkPhysicalDeviceMemoryProperties (32 tipos y 16 montones).
        unsafe { (self.fi.memoria)(self.fisico, &raw mut memoria) };
        let n = (memoria.n_tipos as usize).min(32);
        self.tipos_memoria = memoria.tipos[..n].iter().map(|t| t.propiedades).collect();
        self.perfil.memoria_local = memoria.montones[..(memoria.n_montones as usize).min(16)]
            .iter()
            .filter(|m| m.flags & 1 != 0)
            .map(|m| m.tamano)
            .fold(0u64, u64::saturating_add);
        Ok(())
    }

    /// ¿Esa familia de colas puede presentar en nuestra superficie? (Sin ventana, sí.)
    fn presenta(&self, fisico: Fisico, familia: u32) -> bool {
        if self.superficie == 0 {
            return true;
        }
        let mut si = 0u32;
        // SAFETY: físico y superficie válidos; `si` es nuestro.
        let r = unsafe { (self.fi.soporta_presentar)(fisico, familia, self.superficie, &raw mut si) };
        r == VK_SUCCESS && si != 0
    }

    fn crear_dispositivo(&mut self) -> Result<(), String> {
        let prioridad = 1.0f32;
        let cola = InfoCola {
            s_type: ST_DEVICE_QUEUE_CREATE_INFO,
            p_next: core::ptr::null(),
            flags: 0,
            familia: self.familia,
            n_colas: 1,
            prioridades: &raw const prioridad,
        };
        let extensiones = [EXT_SWAPCHAIN.as_ptr()];
        let n_ext = u32::from(self.superficie != 0); // sin ventana no hace falta swapchain
        // Vulkan 1.3: renderizado dinámico (dibujar sin pases de render ni framebuffers).
        let mut funciones13 =
            Funciones13 { s_type: ST_VULKAN_1_3_FEATURES, p_next: core::ptr::null_mut(), activas: [0; 15] };
        funciones13.activas[RENDERIZADO_DINAMICO] = 1;
        let info = InfoDispositivo {
            s_type: ST_DEVICE_CREATE_INFO,
            p_next: (&raw const funciones13).cast::<c_void>(),
            flags: 0,
            n_colas: 1,
            colas: &raw const cola,
            n_capas: 0,
            capas: core::ptr::null(),
            n_extensiones: n_ext,
            extensiones: extensiones.as_ptr(),
            funciones: core::ptr::null(),
        };
        let (fisico, salida) = (self.fisico, &raw mut self.dispositivo);
        // SAFETY: el físico es válido y `info` (con lo que apunta) vive durante la llamada.
        let r = unsafe { (self.fi.crear_dispositivo)(fisico, &raw const info, core::ptr::null(), salida) };
        ok(r, "crear el dispositivo")?;
        let fd = FnDispositivo::cargar(&self.fi, self.dispositivo, self.superficie != 0)?;
        // SAFETY: el dispositivo se creó con una cola en esta familia.
        unsafe { (fd.cola)(self.dispositivo, self.familia, 0, &raw mut self.cola) };
        self.fd = Some(fd);
        Ok(())
    }
}

impl Drop for Contexto {
    fn drop(&mut self) {
        // SAFETY: se destruye cada objeto una vez, solo si se creó, y en orden
        // inverso: dispositivo (tras esperar a que acabe), superficie, mensajero, instancia.
        unsafe {
            if !self.dispositivo.is_null() {
                (self.fi.esperar_dispositivo)(self.dispositivo);
                (self.fi.destruir_dispositivo)(self.dispositivo, core::ptr::null());
            }
            if self.superficie != 0 {
                (self.fi.destruir_superficie)(self.instancia, self.superficie, core::ptr::null());
            }
            if let (Some((_, destruir)), true) = (self.fi.mensajero, self.mensajero != 0) {
                destruir(self.instancia, self.mensajero, core::ptr::null());
            }
            (self.fi.destruir)(self.instancia, core::ptr::null());
        }
    }
}

fn info_mensajero() -> InfoMensajero {
    InfoMensajero {
        s_type: ST_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: core::ptr::null(),
        flags: 0,
        gravedad: GRAVEDAD_AVISO | GRAVEDAD_ERROR,
        tipos: TIPOS_DE_MENSAJE,
        llamada: aviso,
        usuario: core::ptr::null_mut(),
    }
}

/// ¿Está instalada la capa `VK_LAYER_KHRONOS_validation` (viene con el SDK)?
fn hay_validacion(global: &Global) -> bool {
    let mut n = 0u32;
    // SAFETY: primero el número, luego un Vec de ese tamaño.
    let capas = unsafe {
        if (global.capas)(&raw mut n, core::ptr::null_mut()) != VK_SUCCESS {
            return false;
        }
        let mut v: Vec<PropiedadesCapa> = (0..n)
            .map(|_| PropiedadesCapa { nombre: [0; 256], version_spec: 0, version_impl: 0, descripcion: [0; 256] })
            .collect();
        (global.capas)(&raw mut n, v.as_mut_ptr());
        v
    };
    let buscada = &CAPA_VALIDACION[..CAPA_VALIDACION.len() - 1];
    capas.iter().any(|c| c.nombre.starts_with(buscada) && c.nombre.get(buscada.len()) == Some(&0))
}

fn crear_instancia(global: &Global, nombre_app: &str, validacion: bool) -> Result<Instancia, String> {
    let nombre: Vec<u8> = nombre_app.bytes().chain(core::iter::once(0)).collect();
    let app = InfoApp {
        s_type: ST_APPLICATION_INFO,
        p_next: core::ptr::null(),
        nombre_app: nombre.as_ptr(),
        version_app: 1,
        nombre_motor: c"TELAR".as_ptr().cast::<u8>(),
        version_motor: 1,
        version_api: VK_API_1_3,
    };
    let capas = [CAPA_VALIDACION.as_ptr()];
    let mut extensiones = vec![EXT_SURFACE.as_ptr(), EXT_WIN32_SURFACE.as_ptr()];
    if validacion {
        extensiones.push(EXT_DEBUG_UTILS.as_ptr());
    }
    // El mensajero también en la cadena de la instancia: así se valida su propia creación y destrucción.
    let mensajero = info_mensajero();
    let info = InfoInstancia {
        s_type: ST_INSTANCE_CREATE_INFO,
        p_next: if validacion { (&raw const mensajero).cast::<c_void>() } else { core::ptr::null() },
        flags: 0,
        app: &raw const app,
        n_capas: u32::from(validacion),
        capas: capas.as_ptr(),
        n_extensiones: u32::try_from(extensiones.len()).unwrap_or(0),
        extensiones: extensiones.as_ptr(),
    };
    let mut instancia: Instancia = core::ptr::null_mut();
    // SAFETY: `info` y todo a lo que apunta (nombres, capas, extensiones, mensajero)
    // vive durante la llamada; las cadenas terminan en cero.
    let r = unsafe { (global.crear_instancia)(&raw const info, core::ptr::null(), &raw mut instancia) };
    ok(r, "crear la instancia")?;
    Ok(instancia)
}
