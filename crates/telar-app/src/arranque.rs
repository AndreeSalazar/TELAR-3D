//! Adaptación explícita entre perfil Vulkan y contrato puro, sin unir sus crates.

pub(crate) fn crear(perfil: telar_gpu::PerfilGpu, capacidad: u32, etapas: usize) -> Result<[u32; 32], String> {
    let dispositivo = telar_kernel::arranque::Dispositivo {
        fabricante: perfil.fabricante,
        dispositivo: perfil.dispositivo,
        tipo: perfil.tipo,
        controlador: perfil.controlador,
        vulkan: perfil.vulkan,
        grupos_x: perfil.grupos_x,
        invocaciones: perfil.invocaciones,
        hilos_x: perfil.hilos_x,
        compartida: perfil.compartida,
        rango_almacen: perfil.rango_almacen,
        almacenes: perfil.almacenes,
        memoria_local: perfil.memoria_local,
    };
    telar_kernel::arranque::cabecera(dispositivo, capacidad, u32::try_from(etapas).map_err(|_| "demasiadas etapas")?)
        .map_err(str::to_owned)
}
