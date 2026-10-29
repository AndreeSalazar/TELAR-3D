//! Integración optativa de checkpoint; nunca descarga campos durante cuadros estables.
use crate::{
    cache_campos::{self, Cache},
    humo_entrada::Entrada,
};
use std::path::Path;
use telar_ui::EditorHumo;
pub(crate) struct Sesion {
    pub abrir: Option<Cache>,
    pub guardar: Option<String>,
    pub anteriores: u32,
}
impl Sesion {
    pub(crate) fn nueva(args: &[String]) -> Result<Self, String> {
        let abrir = crate::estudio_humo::valor(args, "--abrir-cache")?;
        let guardar = crate::estudio_humo::valor(args, "--guardar-cache")?;
        if (abrir.is_some() || guardar.is_some())
            && args.iter().any(|v| matches!(v.as_str(), "--humo" | "--particulas" | "--abrir-escena"))
        {
            return Err("caché requiere volumen 3D y no se combina con otra escena".into());
        }
        if guardar
            .as_ref()
            .is_some_and(|v| !Path::new(v).extension().is_some_and(|v| v.eq_ignore_ascii_case("tlcache")))
        {
            return Err("guardar caché requiere .tlcache".into());
        }
        if let Some(p) = &guardar
            && Path::new(p).try_exists().map_err(|e| e.to_string())?
        {
            return Err("el destino de caché ya existe".into());
        }
        let cache = abrir.map(|v| cache_campos::leer(Path::new(&v))).transpose()?;
        let anteriores = cache.as_ref().map_or(0, |v| v.pasos);
        Ok(Self { abrir: cache, guardar, anteriores })
    }
    pub(crate) fn configurar(&self, editor: &mut EditorHumo) -> Result<(), String> {
        if let Some(c) = &self.abrir {
            if c.escena.len() == 200 {
                crate::vfx_archivo::aplicar(&c.escena, editor)?;
                editor.vista = 0;
                return Ok(());
            }
            let (humo, tiempo, pulso) = crate::volumen_archivo::decodificar(&c.escena)?;
            editor.humo = humo;
            editor.volumen_opciones = tiempo;
            editor.explosion = pulso;
            editor.trazadores = c.trazadores;
            editor.volumen = true;
            editor.vista = if c.trazadores { 3 } else { 1 };
        }
        Ok(())
    }
    pub(crate) fn cargar(&self, gpu: &telar_gpu::Gpu, entrada: &mut Entrada) -> Result<(), String> {
        if let Some(c) = &self.abrir {
            if c.perfil != perfil(gpu) {
                return Err("caché pertenece a otra GPU/controlador; reanudación exacta no certificada".into());
            }
            gpu.cargar_campos(&c.campos)?;
            entrada.restaurar_camara(c.camara);
        }
        Ok(())
    }
    pub(crate) fn finalizar(
        &self,
        gpu: &telar_gpu::Gpu,
        editor: &EditorHumo,
        entrada: &Entrada,
        pasos: u32,
    ) -> Result<(), String> {
        if let Some(ruta) = &self.guardar {
            if !editor.volumen || editor.particulas.is_some() {
                return Err("caché solo disponible para volumen MAC".into());
            }
            let cache = Cache {
                trazadores: editor.trazadores,
                pasos: self.anteriores.checked_add(pasos).ok_or("contador de caché agotado")?,
                perfil: perfil(gpu),
                escena: if editor.vfx.is_some() {
                    crate::vfx_archivo::codificar(editor)?
                } else {
                    crate::volumen_archivo::codificar(&editor.humo, &editor.volumen_opciones, &editor.explosion)?
                },
                camara: entrada.camara_guardable(),
                campos: gpu.inspeccionar_campos()?,
            };
            cache_campos::guardar(Path::new(ruta), &cache)?;
            println!(
                "telar: caché reanudable {ruta} · {} pasos totales · {} palabras",
                cache.pasos,
                cache.campos.len()
            );
        }
        Ok(())
    }
}
fn perfil(gpu: &telar_gpu::Gpu) -> [u32; 4] {
    let p = gpu.perfil();
    [p.fabricante, p.dispositivo, p.controlador, p.vulkan]
}
