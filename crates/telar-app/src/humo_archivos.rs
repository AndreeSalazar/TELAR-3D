//! Configuración de laboratorios; separada del bucle de presentación.
use std::path::Path;
use telar_ui::EditorHumo;
pub(crate) fn guardar_editor(ruta: &Path, editor: &EditorHumo) -> Result<(), String> {
    if editor.vfx.is_some() {
        return crate::proyecto::guardar_bytes(ruta, &crate::vfx_archivo::codificar(editor)?);
    }
    if let Some(p) = editor.particulas {
        return crate::particulas_archivo::guardar(ruta, &p);
    }
    if editor.volumen {
        crate::volumen_archivo::guardar(ruta, &editor.humo, &editor.volumen_opciones, &editor.explosion)
    } else {
        crate::humo_archivo::guardar(ruta, &editor.humo)
    }
}
pub(crate) fn leer_editor(ruta: &Path, editor: &mut EditorHumo) -> Result<(), String> {
    if editor.vfx.is_some() {
        return crate::vfx_archivo::leer(ruta, editor);
    }
    if editor.particulas.is_some() {
        editor.particulas = Some(crate::particulas_archivo::leer(ruta)?);
        return Ok(());
    }
    if editor.volumen {
        let (humo, opciones, explosion) = crate::volumen_archivo::leer(ruta)?;
        editor.humo = humo;
        editor.volumen_opciones = opciones;
        editor.explosion = explosion;
    } else {
        editor.humo = crate::humo_archivo::leer(ruta)?;
    }
    Ok(())
}
