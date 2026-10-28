//! Lotes por desplazamiento de grupos: mismo programa y salida residente, sin copias parciales.
use super::tipos::{ACCESO_TRANSFER_WRITE, ETAPA_TRANSFER};
use super::{
    contexto::{Contexto, ok},
    recursos::Recursos,
    tipos::{Comandos, InfoEnvio, InfoInicioComandos, ST_COMMAND_BUFFER_BEGIN_INFO, ST_SUBMIT_INFO, USO_UNA_VEZ},
    tipos_calculo::{
        ACCESO_HOST_READ, ACCESO_SHADER_READ, ACCESO_SHADER_WRITE, ACCESO_TRANSFER_READ, BUFFER_ALMACEN,
        BUFFER_TRANSFER_DST, BUFFER_TRANSFER_SRC, BarreraMemoria, Copia, ETAPA_CALCULO, ETAPA_HOST, PUNTO_CALCULO,
        ST_MEMORY_BARRIER,
    },
};
use crate::{
    PlanLotes, ProgresoLotes,
    eleccion::{MEMORIA_COHERENTE, MEMORIA_CON_CACHE, MEMORIA_GPU, MEMORIA_VISIBLE},
};

pub(crate) fn ejecutar_lotes(
    ctx: &Contexto,
    codigo: &[u32],
    plan: PlanLotes,
    mut continuar: impl FnMut(ProgresoLotes) -> bool,
) -> Result<Option<Vec<u32>>, String> {
    let total = plan.invocaciones.div_ceil(plan.grupo);
    if total > ctx.grupos_max_x {
        return Err("grupos de lotes exceden el límite físico".into());
    }
    if !continuar(ProgresoLotes { completadas: 0, total: plan.invocaciones }) {
        return Ok(None);
    }
    let fd = ctx.fd.ok_or("falta dispositivo")?;
    let bytes = u64::from(plan.elementos) * 4;
    let mut recursos = Recursos::nuevo(&fd, ctx.dispositivo);
    recursos.buffer(ctx, 0, bytes, BUFFER_ALMACEN | BUFFER_TRANSFER_SRC, (MEMORIA_GPU, 0))?;
    recursos.buffer(ctx, 1, bytes, BUFFER_TRANSFER_DST, (MEMORIA_VISIBLE | MEMORIA_COHERENTE, MEMORIA_CON_CACHE))?;
    recursos.pipeline_base(codigo)?;
    recursos.descriptores(bytes)?;
    let cmd = recursos.preparar_reutilizable(ctx)?;
    let mut base = 0;
    while base < total {
        let cantidad = plan.grupos_por_lote.min(total - base);
        enviar(ctx, &recursos, cmd, Some((base, cantidad)), bytes)?;
        base += cantidad;
        if !continuar(ProgresoLotes {
            completadas: base.saturating_mul(plan.grupo).min(plan.invocaciones),
            total: plan.invocaciones,
        }) {
            return Ok(None);
        }
    }
    enviar(ctx, &recursos, cmd, None, bytes)?;
    Ok(Some(recursos.leer(bytes, plan.elementos)?))
}

fn enviar(ctx: &Contexto, r: &Recursos, cmd: Comandos, lote: Option<(u32, u32)>, bytes: u64) -> Result<(), String> {
    let fd = r.fd;
    let fc = fd.calculo;
    let nulo = core::ptr::null();
    let inicio =
        InfoInicioComandos { s_type: ST_COMMAND_BUFFER_BEGIN_INFO, p_next: nulo, flags: USO_UNA_VEZ, herencia: nulo };
    let barrera = BarreraMemoria {
        s_type: ST_MEMORY_BARRIER,
        p_next: nulo,
        acceso_origen: ACCESO_SHADER_WRITE,
        acceso_destino: if lote.is_some() { ACCESO_SHADER_READ | ACCESO_SHADER_WRITE } else { ACCESO_TRANSFER_READ },
    };
    let host = BarreraMemoria {
        s_type: ST_MEMORY_BARRIER,
        p_next: nulo,
        acceso_origen: ACCESO_TRANSFER_WRITE,
        acceso_destino: ACCESO_HOST_READ,
    };
    let copia = Copia { origen: 0, destino: 0, tamano: bytes };
    // SAFETY: la valla del envío previo ya se esperó; pool permite reset de comandos.
    // Recursos y descriptores conservan su vida hasta la valla, también ante error.
    unsafe {
        ok((fd.reiniciar_comandos)(cmd, 0), "reiniciar comandos de lote")?;
        ok((fd.reiniciar_vallas)(r.d, 1, &raw const r.valla), "reiniciar valla de lote")?;
        ok((fd.empezar)(cmd, &raw const inicio), "empezar lote")?;
        // GPU: ordena escrituras de todos los envíos anteriores sobre la misma salida.
        (fd.barrera)(
            cmd,
            ETAPA_CALCULO,
            if lote.is_some() { ETAPA_CALCULO } else { ETAPA_TRANSFER },
            0,
            1,
            (&raw const barrera).cast(),
            0,
            nulo,
            0,
            nulo.cast(),
        );
        if let Some((base, cantidad)) = lote {
            (fc.usar_pipeline)(cmd, PUNTO_CALCULO, r.pipeline);
            (fc.usar_conjuntos)(
                cmd,
                PUNTO_CALCULO,
                r.disposicion_pipeline,
                0,
                1,
                &raw const r.conjunto,
                0,
                nulo.cast(),
            );
            // GPU: intervalos de grupos disjuntos; el shader limita la cola incompleta.
            (fc.despachar_base)(cmd, base, 0, 0, cantidad, 1, 1);
        } else {
            // GPU: una única copia después de todas las escrituras de shader.
            (fc.copiar_buffer)(cmd, r.buffers[0], r.buffers[1], 1, &raw const copia);
            (fd.barrera)(cmd, ETAPA_TRANSFER, ETAPA_HOST, 0, 1, (&raw const host).cast(), 0, nulo, 0, nulo.cast());
        }
        ok((fd.terminar)(cmd), "terminar lote")?;
        let envio = InfoEnvio {
            s_type: ST_SUBMIT_INFO,
            p_next: nulo,
            n_esperas: 0,
            esperas: nulo.cast(),
            etapas_espera: nulo.cast(),
            n_comandos: 1,
            comandos: &raw const cmd,
            n_senales: 0,
            senales: nulo.cast(),
        };
        ok((fd.enviar)(ctx.cola, 1, &raw const envio, r.valla), "enviar lote")?;
        ok((fd.esperar_vallas)(r.d, 1, &raw const r.valla, 1, 10_000_000_000), "esperar lote")
    }
}
