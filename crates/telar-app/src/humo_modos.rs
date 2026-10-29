//! Construcción y validación de modos del laboratorio.
use crate::humo_archivos::leer_editor;
use std::path::Path;
use telar_ui::EditorHumo;
pub(crate) fn validar_modo(args: &[String]) -> Result<(), String> {
    if args.iter().any(|v| v == "--plano-vfx") && args.iter().any(|v| matches!(v.as_str(), "--particulas" | "--humo")) {
        return Err("plano VFX requiere su propio volumen 3D".into());
    }
    if args.iter().any(|v| v == "--guardar-cache")
        && !args.iter().any(|v| matches!(v.as_str(), "--humo-3d" | "--explosion" | "--plano-vfx" | "--abrir-cache"))
    {
        return Err("guardar caché requiere volumen 3D".into());
    }
    if args.iter().any(|v| v == "--trazadores")
        && !args.iter().any(|v| v == "--humo-3d" || v == "--explosion" || v == "--plano-vfx" || v == "--abrir-cache")
    {
        return Err("--trazadores requiere --humo-3d o --explosion".into());
    }
    if args.iter().any(|v| v == "--particulas")
        && args.iter().any(|v| matches!(v.as_str(), "--humo" | "--humo-3d" | "--explosion"))
    {
        return Err("elige partículas o humo/explosión".into());
    }
    if args.iter().any(|v| v == "--humo") && args.iter().any(|v| v == "--humo-3d" || v == "--explosion") {
        return Err("elige humo 2D o 3D".into());
    }
    if args.iter().any(|v| {
        matches!(
            v.as_str(),
            "--fuego" | "--estudio-rayos" | "--render-objetos" | "--render-rayos" | "--calculo" | "--esfera-cpu"
        )
    }) {
        return Err("--humo requiere su propio laboratorio".into());
    }
    Ok(())
}

fn configurar_pulso(args: &[String], editor: &mut EditorHumo) {
    if args.iter().any(|v| v == "--explosion") {
        editor.explosion.0[0] = 0.35;
        editor.humo.0[0] = 0.13;
        editor.volumen_opciones.0[1] = 0.0;
        editor.vista = 1;
        editor.pagina = 3;
        editor.mensaje = "Pulso expansivo · R reinicia reloj · sin combustión química".into();
    }
}

pub(crate) fn predeterminada(args: &[String], volumen: bool) -> &'static str {
    if args.iter().any(|v| v == "--plano-vfx") {
        return "plano-vfx.tlar";
    }
    if args.iter().any(|v| v == "--trazadores") {
        return "trazadores.tlar";
    }
    if args.iter().any(|v| v == "--particulas") {
        return "particulas.tlar";
    }
    if args.iter().any(|v| v == "--explosion") {
        "explosion.tlar"
    } else if volumen {
        "humo3d.tlar"
    } else {
        "humo.tlar"
    }
}

pub(crate) fn crear_editor(args: &[String], volumen: bool, abrir: Option<&str>) -> Result<EditorHumo, String> {
    let mut editor = EditorHumo::default();
    editor.volumen = volumen;
    configurar_pulso(args, &mut editor);
    if args.iter().any(|v| v == "--plano-vfx") {
        editor.volumen = true;
        editor.vfx = Some(telar_ui::Vfx::default());
        editor.trazadores = true;
        editor.humo.0[0] = 0.13;
        editor.explosion.0 = [0.35, 8.0, 5.0, 1.0];
        editor.volumen_opciones.0[1] = 0.0;
        editor.vista = 0;
        editor.pagina = 4;
    }
    editor.trazadores = editor.vfx.is_some() || args.iter().any(|v| v == "--trazadores");
    if editor.trazadores && editor.vfx.is_none() {
        editor.vista = 3;
        editor.mensaje = "128 trazadores MAC GPU · vida fija 2s · R reinicia · F8 partículas GEO".into();
    }
    if args.iter().any(|v| v == "--particulas") {
        editor.particulas = Some(telar_ui::Particulas::default());
        editor.mensaje = "Emisor GPU · plazas reutilizables · Espacio / N / R · D datos · F5/F9 · F8 GEO".into();
    }
    if let Some(ruta) = abrir {
        leer_editor(Path::new(ruta), &mut editor)?;
    }
    Ok(editor)
}

pub(crate) fn nombre_editor(editor: &EditorHumo) -> &'static str {
    if editor.vfx.is_some() {
        "plano VFX GPU"
    } else if editor.particulas.is_some() {
        "partículas GPU"
    } else if editor.trazadores {
        "trazadores MAC GPU"
    } else if editor.volumen {
        "humo MAC 3D"
    } else {
        "humo MAC 2D"
    }
}
