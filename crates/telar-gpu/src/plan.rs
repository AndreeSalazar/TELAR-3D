//! Plan de pasadas residente. Es código puro: valida el contrato antes de crear
//! objetos Vulkan. Las dependencias siguen el orden de las etapas y sus dominios.

/// Dominio de una pasada y contrato de acceso a los cuatro buffers residentes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DominioProcedural {
    /// Un hilo: lee parámetros y caché, escribe control y argumentos indirectos.
    Preparar,
    /// Lee control; escribe vértices e índices cuando la caché GPU lo exige.
    Generar,
    /// Lee parámetros, control y vértices; escribe vértices activos.
    /// Debe conservar x/z para que la caché de esta demostración siga siendo válida.
    Puntos,
    /// Una invocación escribe solicitudes de servicios en el control residente.
    /// Puede ejecutarse aun con geometría vacía o trabajo suspendido.
    Servicio,
}

impl DominioProcedural {
    pub(crate) const fn argumento(self) -> Option<u64> {
        match self {
            Self::Preparar | Self::Servicio => None,
            Self::Generar => Some(48),
            Self::Puntos => Some(0),
        }
    }
}

/// Una operación con identidad y dominio. El shader usa el ABI de cuatro buffers.
#[derive(Clone, Copy)]
pub struct EtapaProcedural<'a> {
    /// Nombre único, usado también en los tiempos de diagnóstico.
    pub nombre: &'a str,
    /// Módulo SPIR-V con entrada `main`; su validación completa corresponde a spirv-val.
    pub codigo: &'a [u32],
    /// Trabajo directo o argumento indirecto que leerá la GPU.
    pub dominio: DominioProcedural,
}

/// Secuencia validada de 2..=32 etapas. Las dependencias se graban una vez:
/// productor → generador → cero o más operaciones por punto/servicio → dibujo.
/// Es extensible en operaciones; conserva el ABI de geometría de la rejilla.
#[derive(Clone, Copy)]
pub struct PlanProcedural<'a> {
    etapas: &'a [EtapaProcedural<'a>],
}

impl<'a> PlanProcedural<'a> {
    /// Valida orden, dominios, nombres y cabeceras antes de tocar la GPU.
    /// Esto no demuestra accesos ni seguridad del código SPIR-V: los kernels
    /// deben cumplir el contrato de su dominio y pasar la verificación externa.
    ///
    /// # Errors
    /// Rechaza planes vacíos, demasiado grandes, sin productor/generador inicial,
    /// con nombres repetidos o con módulos sin cabecera SPIR-V 1.3.
    pub fn nuevo(etapas: &'a [EtapaProcedural<'a>]) -> Result<Self, String> {
        if !(2..=32).contains(&etapas.len()) {
            return Err("el plan requiere de 2 a 32 etapas".to_owned());
        }
        for (indice, etapa) in etapas.iter().enumerate() {
            let valido = match indice {
                0 => etapa.dominio == DominioProcedural::Preparar,
                1 => etapa.dominio == DominioProcedural::Generar,
                _ => matches!(etapa.dominio, DominioProcedural::Puntos | DominioProcedural::Servicio),
            };
            if !valido {
                return Err(format!(
                    "etapa {indice}: dominio incompatible con productor → generador → puntos/servicios"
                ));
            }
            if etapa.nombre.trim().is_empty() || etapas[..indice].iter().any(|e| e.nombre == etapa.nombre) {
                return Err(format!("etapa {indice}: el nombre debe ser no vacío y único"));
            }
            let codigo = etapa.codigo;
            if codigo.len() < 5
                || codigo[0] != 0x0723_0203
                || codigo[1] != 0x0001_0300
                || codigo[3] == 0
                || codigo[4] != 0
            {
                return Err(format!("{}: cabecera SPIR-V 1.3 inválida", etapa.nombre));
            }
        }
        Ok(Self { etapas })
    }

    /// Operaciones en orden de dependencia; nunca se recorren desde CPU por fotograma.
    #[must_use]
    pub const fn etapas(self) -> &'a [EtapaProcedural<'a>] {
        self.etapas
    }
}
