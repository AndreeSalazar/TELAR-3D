//! Conecta el parámetro de Salida con la configuración gráfica aislada.
use crate::controles::Controles;
use telar_gpu::Gpu;

pub(crate) fn sincronizar(gpu: &mut Gpu, controles: &mut Controles, aplicadas: &mut u32) -> Result<(), String> {
    let solicitadas = controles.estudio.salida.muestras;
    if solicitadas != *aplicadas {
        let reales = gpu.configurar_muestras(solicitadas)?;
        controles.estudio.salida.muestras_reales = reales;
        *aplicadas = solicitadas;
        controles.sucio = true;
        println!("telar: Salida · MSAA solicitado {solicitadas}x · efectivo {reales}x");
    }
    Ok(())
}
