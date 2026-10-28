//! Un trabajo final a la vez; la ventana sondea mensajes sin esperar a Vulkan.
use crate::render_objetos::Avance;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};
use telar_ui::EscenaObjetos;

pub(crate) enum Fin {
    Guardado(PathBuf),
    Cancelado,
}
struct Tarea {
    hilo: JoinHandle<Result<Fin, String>>,
    cancelar: Arc<AtomicBool>,
    mensajes: Receiver<Avance>,
}
#[derive(Default)]
pub(crate) struct RenderAsincrono {
    tarea: Option<Tarea>,
}
impl RenderAsincrono {
    pub(crate) fn activo(&self) -> bool {
        self.tarea.is_some()
    }
    pub(crate) fn iniciar(&mut self, escena: EscenaObjetos, ruta: PathBuf, validacion: bool) -> Result<(), String> {
        self.lanzar(move |cancelar, mensajes| {
            let terminado = crate::render_objetos::exportar_controlado(&escena, &ruta, validacion, |avance| {
                let _ = mensajes.try_send(avance);
                !cancelar.load(Ordering::Acquire)
            })?;
            Ok(if terminado { Fin::Guardado(ruta) } else { Fin::Cancelado })
        })
    }
    fn lanzar(
        &mut self,
        trabajo: impl FnOnce(Arc<AtomicBool>, SyncSender<Avance>) -> Result<Fin, String> + Send + 'static,
    ) -> Result<(), String> {
        if self.activo() {
            return Err("ya hay un render en curso".into());
        }
        let cancelar = Arc::new(AtomicBool::new(false));
        let copia = Arc::clone(&cancelar);
        let (mensajes, receptor) = mpsc::sync_channel(8);
        let hilo = thread::Builder::new()
            .name("telar-render-final".into())
            .spawn(move || trabajo(copia, mensajes))
            .map_err(|e| e.to_string())?;
        self.tarea = Some(Tarea { hilo, cancelar, mensajes: receptor });
        Ok(())
    }
    pub(crate) fn cancelar(&self) {
        if let Some(tarea) = &self.tarea {
            tarea.cancelar.store(true, Ordering::Release);
        }
    }
    pub(crate) fn cancelando(&self) -> bool {
        self.tarea.as_ref().is_some_and(|t| t.cancelar.load(Ordering::Acquire))
    }
    /// Retorna inmediatamente cuando el trabajador sigue ocupado.
    pub(crate) fn sondear(&mut self, mut progreso: impl FnMut(Avance)) -> Option<Result<Fin, String>> {
        let tarea = self.tarea.as_ref()?;
        while let Ok(avance) = tarea.mensajes.try_recv() {
            progreso(avance);
        }
        if !tarea.hilo.is_finished() {
            return None;
        }
        self.tarea
            .take()
            .map(|t| t.hilo.join().unwrap_or_else(|_| Err("el trabajador de render terminó con un fallo".into())))
    }
}
impl Drop for RenderAsincrono {
    fn drop(&mut self) {
        // La salida normal de la ventana espera sondeando; este respaldo cubre errores.
        self.cancelar();
        if let Some(tarea) = self.tarea.take() {
            let _ = tarea.hilo.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sondeo_no_espera_y_cancelacion_no_bloquea_el_bucle() -> Result<(), String> {
        let (listo, esperar) = mpsc::sync_channel(1);
        let (liberar, puerta) = mpsc::sync_channel(1);
        let mut render = RenderAsincrono::default();
        render.lanzar(move |cancelar, _| {
            let _ = listo.send(());
            let _ = puerta.recv();
            if cancelar.load(Ordering::Acquire) { Ok(Fin::Cancelado) } else { Err("se esperaba cancelación".into()) }
        })?;
        esperar.recv().map_err(|e| e.to_string())?;
        for _ in 0..100 {
            assert!(render.sondear(|_| {}).is_none());
        }
        assert!(render.activo());
        render.cancelar();
        liberar.send(()).map_err(|e| e.to_string())?;
        let inicio = std::time::Instant::now();
        loop {
            if let Some(resultado) = render.sondear(|_| {}) {
                assert!(matches!(resultado?, Fin::Cancelado));
                break;
            }
            if inicio.elapsed() > std::time::Duration::from_secs(5) {
                return Err("el trabajador no terminó".into());
            }
            thread::yield_now();
        }
        assert!(!render.activo());
        Ok(())
    }
}
