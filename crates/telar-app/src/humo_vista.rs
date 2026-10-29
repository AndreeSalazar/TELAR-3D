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
    if editor.particulas.is_some() {
        return crate::particulas_vista::preparar(editor, fuente, visible, validacion);
    }
    if editor.volumen {
        return crate::volumen_vista::preparar(editor, fuente, visible, validacion);
    }
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
    let programas =
        telar_kernel::humo::Configuracion { lado: LADO, iteraciones: iteraciones(editor) }.compilar_adaptativo()?;
    gpu.preparar_campos(
        Sombreadores {
            vertices: &telar_render::visor_rayos::vertices(),
            fragmentos: &telar_render::humo::fragmentos_adaptativo(LADO)?,
        },
        PlanCampos {
            elementos: LADO * LADO,
            palabras: telar_kernel::humo::CANALES_ADAPTATIVOS,
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
pub(crate) fn secuencia(iteraciones: u32) -> Result<Vec<usize>, String> {
    telar_kernel::humo::orden_adaptativo(iteraciones).map_err(Into::into)
}
#[allow(clippy::cast_precision_loss, reason = "vista <=3")]
pub(crate) fn bloque(editor: &EditorHumo, srgb: bool) -> [f32; 32] {
    let mut bloque = [0.0; 32];
    bloque[..8].copy_from_slice(&editor.humo.0[..8]);
    bloque[10] = editor.humo.0[7];
    bloque[11] = editor.humo.0[8];
    bloque[12] = editor.humo.0[9];
    bloque[7] = editor.vista as f32;
    bloque[9] = f32::from(u8::from(srgb));
    bloque
}
pub(crate) fn datos(gpu: &Gpu) -> Result<String, String> {
    resumen(gpu, telar_kernel::humo::CONTROL_ADAPTATIVO * LADO * LADO, false)
}
pub(crate) fn datos_editor(gpu: &Gpu, editor: &EditorHumo) -> Result<String, String> {
    if editor.particulas.is_some() {
        return crate::particulas_vista::datos(gpu);
    }
    if editor.volumen {
        let resumen = resumen(gpu, telar_kernel::humo3d::CONTROL * crate::volumen_vista::LADO.pow(3), true)?;
        let resumen = if gpu.carga_campos().indirectos > 0 {
            let argumentos = gpu.inspeccionar_argumentos_campos()?;
            format!("{resumen} · próximos grupos GPU {}/{}", argumentos[0], argumentos[3])
        } else {
            resumen
        };
        if editor.trazadores {
            let flags = gpu.inspeccionar_rango_campos(
                (if editor.vfx.is_some() { 20 } else { 17 }) * crate::volumen_vista::LADO.pow(3) + 10 * 128,
                128,
            )?;
            let vivas = flags.iter().filter(|v| f32::from_bits(**v) > 0.0).count();
            let malas = flags.iter().filter(|v| f32::from_bits(**v) < 0.0).count();
            Ok(format!("{resumen} · {vivas} trazadores · {malas} cuarentena"))
        } else {
            Ok(resumen)
        }
    } else {
        datos(gpu)
    }
}
fn resumen(gpu: &Gpu, inicio: u32, temporal: bool) -> Result<String, String> {
    let palabras = gpu.inspeccionar_rango_campos(inicio, 16)?;
    let valores: Vec<_> = palabras.into_iter().map(f32::from_bits).collect();
    let estado = match valores[5] {
        1.0 => "tolerancia",
        2.0 => "límite",
        3.0 => "inválido",
        _ => "sin pasos",
    };
    let texto = format!(
        "GPU: {estado} · {} iter · residual {:.5} / umbral {:.5} · dens máx {:.3} · calor {:.3}",
        valores[1], valores[3], valores[4], valores[6], valores[7]
    );
    if temporal {
        let estado_tiempo = match valores[14] {
            1.0 => "CFL limita",
            2.0 => "tiempo inválido",
            _ => "dt",
        };
        return Ok(format!(
            "{texto} · {estado_tiempo} {:.5} · t {:.3}s · CFL {:.3}",
            valores[10], valores[13], valores[12]
        ));
    }
    Ok(texto)
}

pub(crate) fn capturar_bloque(
    gpu: &mut Gpu,
    editor: &EditorHumo,
    fuente: &Fuente,
    ruta: &std::path::Path,
    bloque: &[f32; 32],
) -> Result<(), String> {
    use std::io::Write;
    if ruta.extension().is_none_or(|v| !v.eq_ignore_ascii_case("ppm")) {
        return Err("captura requiere .ppm".into());
    }
    gpu.activar_campos(false)?;
    let d = diseno_editor(editor, (1280, 800));
    let imagen = gpu.capturar_campos_interfaz(
        d.tamano,
        [0.03, 0.04, 0.05, 1.0],
        bloque,
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

pub(crate) fn diseno_editor(editor: &EditorHumo, tamano: (u32, u32)) -> DisenoObjetos {
    if editor.particulas.is_some() {
        return crate::particulas_vista::diseno(tamano);
    }
    if editor.vfx.is_some() {
        let mut d = crate::volumen_vista::diseno(tamano);
        let lado = d.visor.ancho.min(384.0);
        d.visor.x += (d.visor.ancho - lado) * 0.5;
        d.visor.y += (d.visor.alto - lado) * 0.5;
        d.visor.ancho = lado;
        d.visor.alto = lado;
        d
    } else if editor.volumen {
        crate::volumen_vista::diseno(tamano)
    } else {
        diseno(tamano)
    }
}
