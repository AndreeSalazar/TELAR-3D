//! Humo 2D residente: MAC, advección semi-lagrangiana, fuerzas y proyección Jacobi.
use crate::humo_memoria::Memoria;
use alloc::vec::Vec;
use hilo_spirv::{
    Modulo,
    op::{self, Clase},
};
/// Canales por celda: u,v,densidad,calor; cuatro provisionales; div,pA,pB,div final.
pub const CANALES: u32 = 12;
/// Configuración estructural. Cambiarla reinicializa los campos.
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
        let codigos = (0..6).map(|etapa| programa(self, etapa)).collect();
        let mut orden = alloc::vec![0, 1];
        orden.extend((0..self.iteraciones).map(|i| if i % 2 == 0 { 2 } else { 3 }));
        orden.extend([4, 5]);
        Ok(Programas { codigos, orden })
    }
}
fn programa(config: Configuracion, etapa: u32) -> Vec<u32> {
    let mut modulo = Modulo::nuevo();
    let memoria = Memoria::nueva(&mut modulo, config.lado);
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
    let cantidad = Memoria::u(&mut f, config.lado * config.lado);
    let tamano = Memoria::u(&mut f, CANALES * config.lado * config.lado);
    let capacidad = f.longitud(memoria.entero, memoria.buffer, 0);
    let cabe = f.binaria(op::U_LESS_THAN_EQUAL, memoria.booleano, tamano, capacidad);
    let activo = f.binaria(op::U_LESS_THAN, memoria.booleano, pixel, cantidad);
    let activo = f.binaria(op::LOGICAL_AND, memoria.booleano, activo, cabe);
    f.fusion(fin);
    f.saltar_si(activo, cuerpo, fin);
    f.bloque(cuerpo);
    crate::humo_pasos::emitir(&mut f, &memoria, pixel, etapa, config.iteraciones);
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
