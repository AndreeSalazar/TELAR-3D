//! Humo 2D residente: MAC, advección semi-lagrangiana, fuerzas y proyección Jacobi.
use crate::humo_memoria::Memoria;
use alloc::vec::Vec;
use hilo_spirv::{
    Modulo,
    op::{self, Clase},
};
/// Canales por celda: u,v,densidad,calor; cuatro provisionales; div,pA,pB,div final.
pub const CANALES: u32 = 12;
/// Configuración de compilación; el ejecutor puede cambiar el presupuesto conservando campos.
#[derive(Clone, Copy)]
pub struct Configuracion {
    /// Celdas por lado, 8..256.
    pub lado: u32,
    /// Iteraciones Jacobi, 1..128; no equivale a tolerancia de convergencia.
    pub iteraciones: u32,
}
/// Programas únicos y orden de despacho; parámetros dinámicos en constantes.
pub struct Programas {
    /// Advectar, divergencia, Jacobi A/B, proyectar y diagnosticar.
    pub codigos: Vec<Vec<u32>>,
    /// Secuencia determinista de índices de programas.
    pub orden: Vec<usize>,
}
impl Configuracion {
    /// Emite SPIR-V y plan de un paso fijo. No ejecuta física en CPU.
    /// # Errors
    /// Rechaza tamaños/iteraciones fuera del presupuesto.
    pub fn compilar(self) -> Result<Programas, &'static str> {
        if !(8..=256).contains(&self.lado) || !(1..=128).contains(&self.iteraciones) {
            return Err("humo fuera de presupuesto");
        }
        let codigos = (0..6).map(|etapa| programa(self, etapa, false, false)).collect();
        let mut orden = alloc::vec![0, 1];
        orden.extend((0..self.iteraciones).map(|i| if i % 2 == 0 { 2 } else { 3 }));
        orden.extend([4, 5]);
        Ok(Programas { codigos, orden })
    }
}
/// Quince planos en modo adaptativo, incluidos parciales y control GPU.
pub const CANALES_ADAPTATIVOS: u32 = 15;
/// Plano de control: activo, iteraciones, div∞, residual∞, umbral, estado, máximos y contador.
pub const CONTROL_ADAPTATIVO: u32 = 14;
impl Configuracion {
    /// Compila control de error en GPU, con comprobaciones después de pares Jacobi.
    /// # Errors
    /// Rechaza lado fuera de 8..256 o presupuesto impar/fuera de 2..128.
    pub fn compilar_adaptativo(self) -> Result<Programas, &'static str> {
        if !(8..=256).contains(&self.lado) {
            return Err("lado adaptativo inválido");
        }
        let orden = orden_adaptativo(self.iteraciones)?;
        let codigos = (0..14).map(|etapa| programa(self, etapa, true, false)).collect();
        Ok(Programas { codigos, orden })
    }
}
/// Plan acotado: los kernels descartan el trabajo posterior a convergencia.
/// # Errors
/// Rechaza presupuestos fuera de 2..128 o impares.
pub fn orden_adaptativo(maximo: u32) -> Result<Vec<usize>, &'static str> {
    if !(2..=128).contains(&maximo) || !maximo.is_multiple_of(2) {
        return Err("presupuesto adaptativo inválido");
    }
    let mut orden = alloc::vec![0, 1, 6, 7, 8];
    for _ in 0..maximo / 2 {
        orden.extend([9, 10, 11, 12, 13]);
    }
    orden.extend([4, 5]);
    Ok(orden)
}
pub(super) fn programa(config: Configuracion, etapa: u32, adaptativo: bool, volumen: bool) -> Vec<u32> {
    programa_trazadores(config, etapa, adaptativo, volumen, false)
}
pub(super) fn programa_trazadores(
    config: Configuracion,
    etapa: u32,
    adaptativo: bool,
    volumen: bool,
    trazadores: bool,
) -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let memoria = Memoria::nueva(&mut modulo, config.lado, volumen);
    let v3 = modulo.tipo_vector(memoria.entero, 3);
    let ptr = modulo.tipo_puntero(Clase::Entrada, v3);
    let global = modulo.variable(ptr, Clase::Entrada);
    modulo.decorar(global, op::DECORACION_INTEGRADA, &[op::INTEGRADA_ID_GLOBAL]);
    let void = modulo.tipo_void();
    let firma = modulo.tipo_funcion(void, &[]);
    let mut f = modulo.funcion(void, firma);
    let [inicio, cuerpo, fin] = core::array::from_fn(|_| f.etiqueta());
    f.bloque(inicio);
    let cargado = f.cargar(v3, global);
    let pixel = f.extraer(memoria.entero, cargado, 0);
    let invocaciones = match etapa {
        7 | 12 | 15 => memoria.celdas().div_ceil(64),
        8 | 13 | 16 => 1,
        _ => memoria.celdas(),
    };
    let cantidad = Memoria::u(&mut f, invocaciones);
    let canales = if volumen {
        crate::humo3d::CANALES
    } else if adaptativo {
        CANALES_ADAPTATIVOS
    } else {
        CANALES
    };
    let tamano = Memoria::u(&mut f, canales * memoria.celdas());
    let capacidad = f.longitud(memoria.entero, memoria.buffer, 0);
    let cabe = f.binaria(op::U_LESS_THAN_EQUAL, memoria.booleano, tamano, capacidad);
    let activo = f.binaria(op::U_LESS_THAN, memoria.booleano, pixel, cantidad);
    let activo = f.binaria(op::LOGICAL_AND, memoria.booleano, activo, cabe);
    f.fusion(fin);
    f.saltar_si(activo, cuerpo, fin);
    f.bloque(cuerpo);
    if volumen && etapa >= 15 {
        crate::humo3d_tiempo::emitir(&mut f, &memoria, pixel, etapa);
    } else if adaptativo && etapa == 4 {
        crate::humo_control::emitir(&mut f, &memoria, pixel, 14);
    } else if etapa < 6 && volumen {
        crate::humo3d_pasos::emitir(&mut f, &memoria, pixel, etapa, config.iteraciones);
    } else if etapa < 6 {
        crate::humo_pasos::emitir(&mut f, &memoria, pixel, etapa, config.iteraciones);
    } else {
        crate::humo_control::emitir(&mut f, &memoria, pixel, etapa);
    }
    if trazadores {
        crate::trazadores_paso::emitir(&mut f, &memoria, pixel);
    }
    f.saltar(fin);
    f.bloque(fin);
    f.volver();
    let main = f.terminar();
    modulo.entrada_calculo(main, "main", &[global], [64, 1, 1]);
    modulo.ensamblar()
}
/// Referencia CPU pequeña para pruebas, no utilizada por el visor.
#[path = "humo_referencia.rs"]
pub mod referencia;
