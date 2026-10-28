//! Adaptación del laboratorio 3D a campos residentes y visor orbitable.
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
use telar_ui::{DisenoObjetos, EditorHumo, Fuente};
use telar_ventana::{Config, Ventana};
pub(crate) const LADO: u32 = 32;
pub(crate) fn preparar(
    editor: &EditorHumo,
    fuente: &Fuente,
    visible: bool,
    validacion: bool,
) -> Result<(Ventana, Gpu), String> {
    let ventana = Ventana::nueva(&Config {
        titulo: "TELAR · Humo físico 3D · prototipo 32³".into(),
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
    let programas =
        telar_kernel::humo3d::Configuracion { lado: LADO, iteraciones: crate::humo_vista::iteraciones(editor) }
            .compilar_temporal(subpasos(editor))?;
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::volumen::fragmentos(LADO, 64)?,
        },
        PlanCampos {
            elementos: LADO * LADO * LADO,
            palabras: telar_kernel::humo3d::CANALES,
            programas: &programas.codigos,
            secuencia: &programas.orden,
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
    let lado = d.visor.ancho.min(d.visor.alto).min(512.0);
    d.visor.x += (d.visor.ancho - lado) * 0.5;
    d.visor.y += (d.visor.alto - lado) * 0.5;
    d.visor.ancho = lado;
    d.visor.alto = lado;
    d
}
pub(crate) fn bloque(editor: &EditorHumo, camara: &nudo::Camara, srgb: bool) -> Result<[f32; 32], String> {
    let camara = crate::escena_final::camara(camara.posicion(), camara.objetivo, 1.0)?;
    let mut bloque = crate::humo_vista::bloque(editor, srgb);
    for (inicio, vector) in [(16, camara.frente), (20, camara.derecha), (24, camara.arriba), (28, camara.origen)] {
        bloque[inicio..inicio + 3].copy_from_slice(&vector);
    }
    let [subpasos, radio, altura, cfl] = editor.volumen_opciones.0;
    bloque[13] = subpasos;
    bloque[14] = radio;
    bloque[15] = altura;
    bloque[19] = cfl;
    let [duracion, expansion, calor, humo] = editor.explosion.0;
    bloque[8] = duracion;
    bloque[23] = expansion;
    bloque[27] = calor;
    bloque[31] = humo;
    Ok(bloque)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "UI valida entero 1..4")]
pub(crate) fn subpasos(editor: &EditorHumo) -> u32 {
    editor.volumen_opciones.0[0] as u32
}
pub(crate) fn secuencia(editor: &EditorHumo) -> Result<Vec<usize>, String> {
    telar_kernel::humo3d::orden_temporal(crate::humo_vista::iteraciones(editor), subpasos(editor)).map_err(Into::into)
}
