//! Adaptador del laboratorio MAC: shaders, constantes y diagnóstico explícito.
use telar_gpu::{Gpu, Opciones, PlanCampos, Sombreadores, Superficie};
use telar_ui::{DisenoObjetos, EditorHumo, Fuente};
use telar_ventana::{Config, Ventana};
pub(crate) const LADO: u32 = 128;
const MALLA: [[f32; 6]; 3] =
    [[-1.0, -1.0, 0.5, 0.0, 0.0, 1.0], [3.0, -1.0, 0.5, 0.0, 0.0, 1.0], [-1.0, 3.0, 0.5, 0.0, 0.0, 1.0]];
pub(crate) fn preparar(
    editor: &EditorHumo,
    fuente: &Fuente,
    visible: bool,
    validacion: bool,
) -> Result<(Ventana, Gpu), String> {
    let ventana = Ventana::nueva(&Config {
        titulo: "TELAR · Humo MAC 2D · campos persistentes GPU".into(),
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
    let programas = telar_kernel::humo::Configuracion { lado: LADO, iteraciones: iteraciones(editor) }.compilar()?;
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::humo::fragmentos(LADO)?,
        },
        PlanCampos {
            elementos: LADO * LADO,
            palabras: telar_kernel::humo::CANALES,
            programas: &programas.codigos,
            secuencia: &programas.orden,
        },
    )?;
    gpu.subir_malla(&MALLA, &[0, 2, 1])?;
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
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "iteraciones pares validadas 2..128")]
pub(crate) fn iteraciones(editor: &EditorHumo) -> u32 {
    editor.humo.0[7] as u32
}
pub(crate) fn secuencia(iteraciones: u32) -> Vec<usize> {
    let mut orden = vec![0, 1];
    orden.extend((0..iteraciones).map(|i| if i % 2 == 0 { 2 } else { 3 }));
    orden.extend([4, 5]);
    orden
}
#[allow(clippy::cast_precision_loss, reason = "vista <=3")]
pub(crate) fn bloque(editor: &EditorHumo, srgb: bool) -> [f32; 32] {
    let mut bloque = [0.0; 32];
    bloque[..8].copy_from_slice(&editor.humo.0);
    bloque[7] = editor.vista as f32;
    bloque[9] = f32::from(u8::from(srgb));
    bloque
}
pub(crate) fn datos(gpu: &Gpu) -> Result<String, String> {
    let valores = gpu.inspeccionar_campos()?;
    let n = (LADO * LADO) as usize;
    let plano = |i: usize| valores[i * n..(i + 1) * n].iter().map(|v| f32::from_bits(*v));
    let densidad = plano(2).fold(0.0_f32, f32::max);
    let calor = plano(3).fold(0.0_f32, f32::max);
    let rms = |i: usize| {
        #[allow(clippy::cast_precision_loss, reason = "conteo de 16384 celdas")]
        let media = plano(i).map(|v| f64::from(v).powi(2)).sum::<f64>() / n as f64;
        media.sqrt()
    };
    Ok(format!(
        "Datos bajo petición · dens máx {densidad:.3} · calor máx {calor:.3} · div RMS {:.4} / {:.4}",
        rms(8),
        rms(11)
    ))
}
pub(crate) fn capturar(
    gpu: &mut Gpu,
    editor: &EditorHumo,
    fuente: &Fuente,
    ruta: &std::path::Path,
) -> Result<(), String> {
    use std::io::Write;
    if ruta.extension().is_none_or(|v| !v.eq_ignore_ascii_case("ppm")) {
        return Err("captura requiere .ppm".into());
    }
    gpu.activar_campos(false)?;
    let d = diseno((1280, 800));
    let imagen = gpu.capturar_campos_interfaz(
        d.tamano,
        [0.03, 0.04, 0.05, 1.0],
        &bloque(editor, true),
        telar_gpu::CapaInterfaz {
            shaders: Sombreadores {
                vertices: &hilo_spirv::interfaz::vertices(),
                fragmentos: &hilo_spirv::interfaz::fragmentos(),
            },
            vertices: &editor.dibujar(d, fuente)?,
            area: d.area(),
            atlas: Some(telar_gpu::AtlasInterfaz {
                ancho: fuente.ancho,
                alto: fuente.alto,
                cobertura: &fuente.cobertura,
            }),
        },
    )?;
    let mut bytes = b"P6\n1280 800\n255\n".to_vec();
    for p in imagen.pixeles.as_chunks::<4>().0 {
        bytes.extend_from_slice(&p[..3]);
    }
    let mut archivo = std::fs::OpenOptions::new().create_new(true).write(true).open(ruta).map_err(|e| e.to_string())?;
    archivo.write_all(&bytes).map_err(|e| e.to_string())?;
    Ok(())
}

/// Dominio cuadrado para conservar la proporción de la rejilla MAC.
pub(crate) fn diseno(tamano: (u32, u32)) -> DisenoObjetos {
    let mut d = DisenoObjetos::nuevo(tamano);
    if tamano.0 >= 760 && tamano.1 >= 650 {
        let lado = d.visor.ancho.min(d.visor.alto);
        d.visor.x += (d.visor.ancho - lado) * 0.5;
        d.visor.y += (d.visor.alto - lado) * 0.5;
        d.visor.ancho = lado;
        d.visor.alto = lado;
    }
    d
}
