//! Guardado/apertura por petición explícita; no hace E/S durante cuadros estables.
use crate::{
    controles::Controles,
    proyecto::{self, Proyecto},
};
use std::path::Path;
use telar_gpu::Gpu;
use telar_ui::AccionProyecto;

pub(crate) fn aplicar(controles: &mut Controles, proyecto: &Proyecto) -> Result<(), String> {
    proyecto.validar()?;
    controles.estudio.restaurar_estado(&proyecto.estudio)?;
    if !proyecto.red.is_empty() {
        controles.estudio.restaurar_red(&proyecto.red)?;
    }
    controles.estudio.solicitar_plan();
    controles.camara = proyecto.camara.clone();
    controles.fase = proyecto.fase;
    controles.restaurar_interaccion();
    Ok(())
}

pub(crate) fn guardar(ruta: &Path, controles: &Controles) -> Result<(), String> {
    let proyecto = Proyecto::capturar(&controles.estudio, &controles.camara, controles.fase);
    proyecto::guardar(ruta, &proyecto)
}

pub(crate) fn procesar(ruta: &Path, controles: &mut Controles, gpu: &mut Gpu) {
    let Some(accion) = controles.estudio.tomar_accion_proyecto() else { return };
    let resultado = match accion {
        AccionProyecto::Guardar => guardar(ruta, controles),
        AccionProyecto::Abrir => (|| {
            let proyecto = proyecto::leer(ruta)?;
            if proyecto.estudio[1] != controles.estudio.rejilla.capacidad {
                return Err("capacidad distinta; reinicia con --abrir-escena y la ruta del archivo".into());
            }
            let reales = gpu.configurar_muestras(proyecto.estudio[8])?;
            aplicar(controles, &proyecto)?;
            controles.estudio.salida.muestras_reales = reales;
            Ok(())
        })(),
    };
    let mensaje = match resultado {
        Ok(()) => {
            println!("telar: escena {accion:?}: {}", ruta.display());
            if accion == AccionProyecto::Guardar { "Escena guardada" } else { "Escena abierta" }
        }
        Err(error) => {
            eprintln!("telar: escena: {error}");
            "Error · ver consola"
        }
    };
    controles.estudio.informar_proyecto(mensaje);
    controles.sucio = true;
}
