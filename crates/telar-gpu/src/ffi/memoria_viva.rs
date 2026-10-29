//! `EXT_memory_budget` opcional. Estructuras Vulkan 1.3 y EXT cotejadas con `vulkan_core.h`.
use super::{
    contexto::{Contexto, ok},
    tipos::Fisico,
    tipos_calculo::{Monton, PropiedadesMemoria, TipoMemoria},
};
use core::ffi::{CStr, c_void};
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Extension {
    nombre: [u8; 256],
    version: u32,
}
#[repr(C)]
pub(crate) struct Propiedades2 {
    s_type: u32,
    p_next: *mut c_void,
    memoria: PropiedadesMemoria,
}
#[repr(C)]
struct DatosBudget {
    s_type: u32,
    p_next: *mut c_void,
    presupuesto: [u64; 16],
    uso: [u64; 16],
}
pub(crate) type Listar = unsafe extern "system" fn(Fisico, *const u8, *mut u32, *mut Extension) -> i32;
pub(crate) type Leer = unsafe extern "system" fn(Fisico, *mut Propiedades2);
pub(crate) fn soporta(ctx: &Contexto) -> Result<bool, String> {
    for _ in 0..3 {
        let mut cantidad = 0;
        ok(
            // SAFETY: físico válido; consulta de tamaño sin capa explícita.
            unsafe { (ctx.fi.extensiones)(ctx.fisico, core::ptr::null(), &raw mut cantidad, core::ptr::null_mut()) },
            "contar extensiones",
        )?;
        if cantidad > 4096 {
            return Err("demasiadas extensiones Vulkan".into());
        }
        let mut extensiones = vec![Extension { nombre: [0; 256], version: 0 }; cantidad as usize];
        let resultado =
            // SAFETY: capacidad para cantidad elementos; estructura VkExtensionProperties exacta.
            unsafe { (ctx.fi.extensiones)(ctx.fisico, core::ptr::null(), &raw mut cantidad, extensiones.as_mut_ptr()) };
        if resultado == 5 {
            continue;
        } // VK_INCOMPLETE: recontar si cambió la lista.
        ok(resultado, "leer extensiones")?;
        return Ok(extensiones[..cantidad as usize]
            .iter()
            .any(|e| CStr::from_bytes_until_nul(&e.nombre).is_ok_and(|n| n.to_bytes() == b"VK_EXT_memory_budget")));
    }
    Err("la lista de extensiones no se estabilizó".into())
}
pub(crate) fn consultar(ctx: &Contexto) -> Vec<crate::MemoriaHeap> {
    let mut presupuesto =
        DatosBudget { s_type: 1_000_237_000, p_next: core::ptr::null_mut(), presupuesto: [0; 16], uso: [0; 16] };
    let mut datos = Propiedades2 {
        s_type: 1_000_059_006,
        p_next: if ctx.presupuesto_memoria { (&raw mut presupuesto).cast() } else { core::ptr::null_mut() },
        memoria: PropiedadesMemoria {
            n_tipos: 0,
            tipos: [TipoMemoria { propiedades: 0, monton: 0 }; 32],
            n_montones: 0,
            montones: [Monton { tamano: 0, flags: 0 }; 16],
        },
    };
    // SAFETY: físico válido, estructuras repr(C) y cadena opcional soportada y habilitada.
    unsafe { (ctx.fi.memoria2)(ctx.fisico, &raw mut datos) };
    (0..(datos.memoria.n_montones as usize).min(16))
        .map(|i| crate::MemoriaHeap {
            capacidad: datos.memoria.montones[i].tamano,
            local: datos.memoria.montones[i].flags & 1 != 0,
            uso: ctx.presupuesto_memoria.then_some(presupuesto.uso[i]),
            presupuesto: ctx.presupuesto_memoria.then_some(presupuesto.presupuesto[i]),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn abi_coincide_con_vulkan_x64() {
        use core::mem::{offset_of, size_of};
        assert_eq!(size_of::<super::Extension>(), 260);
        assert_eq!(size_of::<super::Propiedades2>(), 536);
        assert_eq!(offset_of!(super::Propiedades2, memoria), 16);
        assert_eq!(size_of::<super::DatosBudget>(), 272);
        assert_eq!(offset_of!(super::DatosBudget, presupuesto), 16);
        assert_eq!(offset_of!(super::DatosBudget, uso), 144);
    }
}
