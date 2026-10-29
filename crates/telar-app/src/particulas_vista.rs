//! Preparación de partículas residentes, visor orbitable y diagnóstico explícito.
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
use telar_ui::{DisenoObjetos, EditorHumo, Fuente};
use telar_ventana::{Config, Ventana};
pub(crate) const CAPACIDAD: u32 = 128;
pub(crate) fn preparar(
    _editor: &EditorHumo,
    fuente: &Fuente,
    visible: bool,
    validacion: bool,
) -> Result<(Ventana, Gpu), String> {
    let ventana = Ventana::nueva(&Config {
        titulo: "TELAR · Partículas GPU · 128 plazas".into(),
        ancho: 1280,
        alto: 800,
        visible,
    })?;
    let m = ventana.manejadores();
    let mut gpu = Gpu::nueva(
        &Opciones { muestras: 1, validacion, ..Default::default() },
        Superficie { instancia: m.instancia, ventana: m.ventana },
        ventana.tamano(),
    )?;
    let programas = telar_kernel::particulas::compilar();
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::particulas::fragmentos(),
        },
        PlanCampos {
            elementos: CAPACIDAD,
            palabras: telar_kernel::particulas::CANALES,
            programas: &programas,
            secuencia: &[0, 1],
        },
    )?;
    gpu.subir_malla(
        &[[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]],
        &[0, 2, 1],
    )?;
    gpu.preparar_interfaz(Sombreadores {
        vertices: &hilo_spirv::interfaz::vertices(),
        fragmentos: &hilo_spirv::interfaz::fragmentos(),
    })?;
    gpu.fuente_interfaz(telar_gpu::AtlasInterfaz {
        ancho: fuente.ancho,
        alto: fuente.alto,
        cobertura: &fuente.cobertura,
    })?;
    Ok((ventana, gpu))
}

pub(crate) fn diseno(tamano: (u32, u32)) -> DisenoObjetos {
    let mut d = crate::humo_vista::diseno(tamano);
    let lado = d.visor.ancho.min(d.visor.alto).min(384.0);
    d.visor.x += (d.visor.ancho - lado) * 0.5;
    d.visor.y += (d.visor.alto - lado) * 0.5;
    d.visor.ancho = lado;
    d.visor.alto = lado;
    d
}
pub(crate) fn bloque(editor: &EditorHumo, camara: &nudo::Camara, srgb: bool) -> Result<[f32; 32], String> {
    let parametros = editor.particulas.ok_or("faltan parámetros de partículas")?;
    parametros.validar()?;
    let camara = crate::escena_final::camara(camara.posicion(), camara.objetivo, 1.0)?;
    let mut b = [0.0; 32];
    b[..10].copy_from_slice(&parametros.0);
    b[10] = 1.0;
    b[11] = f32::from(u8::from(srgb));
    #[allow(clippy::cast_precision_loss, reason = "cuatro vistas")]
    {
        b[12] = editor.vista as f32;
    }
    for (inicio, v) in [(16, camara.frente), (20, camara.derecha), (24, camara.arriba), (28, camara.origen)] {
        b[inicio..inicio + 3].copy_from_slice(&v);
    }
    Ok(b)
}
pub(crate) fn datos(gpu: &Gpu) -> Result<String, String> {
    let datos = gpu.inspeccionar_rango_campos(telar_kernel::particulas::CONTROL * CAPACIDAD, 8)?;
    let d: Vec<_> = datos.into_iter().map(f32::from_bits).collect();
    Ok(format!(
        "GPU: {} vivas · {} nacimientos · {} cuarentena · t {:.3}s · dt {:.5} · D datos / R reinicia",
        d[1], d[2], d[3], d[0], d[6]
    ))
}
