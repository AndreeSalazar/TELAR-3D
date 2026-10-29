//! Salida final HDR y secuencia reproducible; simulación y muestras siguen en GPU.
use std::path::{Path, PathBuf};
use telar_ui::EditorHumo;
pub(crate) struct Salida {
    final_pfm: Option<String>,
    carpeta: Option<PathBuf>,
    muestras: u32,
    intervalo: u32,
    pases: bool,
    pub objetivo: Option<u32>,
    pub cuadros: u32,
}
impl Salida {
    pub(crate) fn nueva(args: &[String]) -> Result<Self, String> {
        let valor = crate::estudio_humo::valor;
        let final_pfm = valor(args, "--render-vfx")?;
        let carpeta = valor(args, "--secuencia-vfx")?.map(PathBuf::from);
        let muestras =
            valor(args, "--spp")?.map(|v| v.parse::<u32>().map_err(|_| "spp inválido")).transpose()?.unwrap_or(4);
        let intervalo = valor(args, "--intervalo")?
            .map(|v| v.parse::<u32>().map_err(|_| "intervalo inválido"))
            .transpose()?
            .unwrap_or(1);
        let cuadros = valor(args, "--fotogramas")?
            .map(|v| v.parse::<u32>().map_err(|_| "fotogramas inválidos"))
            .transpose()?
            .unwrap_or(1);
        if !(1..=1024).contains(&muestras) || !(1..=2000).contains(&intervalo) || !(1..=1000).contains(&cuadros) {
            return Err("calidad o secuencia fuera de presupuesto".into());
        }
        let total = cuadros.checked_mul(intervalo).filter(|v| *v <= 2000).ok_or("secuencia supera 2000 pasos")?;
        if let Some(r) = &final_pfm {
            validar_pfm(Path::new(r))?;
        }
        Ok(Self {
            final_pfm,
            carpeta,
            muestras,
            intervalo,
            pases: args.iter().any(|v| v == "--pases-vfx"),
            objetivo: (valor(args, "--secuencia-vfx")?.is_some()).then_some(total),
            cuadros: 0,
        })
    }
    pub(crate) fn iniciar(&self, editor: &EditorHumo) -> Result<(), String> {
        self.validar(editor)?;
        if let Some(p) = &self.carpeta {
            std::fs::create_dir(p).map_err(|e| format!("la carpeta de secuencia debe ser nueva: {e}"))?;
            crate::proyecto::guardar_bytes(&p.join("escena.tlar"), &crate::vfx_archivo::codificar(editor)?)?;
            std::fs::write(
                p.join("manifesto.txt"),
                format!(
                    "TELAR VFX · 256x256 · {} spp · intervalo {} cuadros
Tiempo simulado acotado por CFL; consultar el reloj GPU.
PFM lineal, PPM Reinhard/sRGB; campos normalizados, sin calibración espectral.
",
                    self.muestras, self.intervalo
                ),
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub(crate) fn activa(&self) -> bool {
        self.final_pfm.is_some() || self.carpeta.is_some()
    }
    pub(crate) fn validar(&self, editor: &EditorHumo) -> Result<(), String> {
        if self.activa() && editor.vfx.is_none() {
            Err("salida VFX requiere --plano-vfx o caché reactiva".into())
        } else {
            Ok(())
        }
    }
    pub(crate) fn cuadro(
        &mut self,
        gpu: &mut telar_gpu::Gpu,
        editor: &EditorHumo,
        bloque: [f32; 32],
        pasos: u32,
    ) -> Result<(), String> {
        if pasos == 0 || !pasos.is_multiple_of(self.intervalo) {
            return Ok(());
        }
        if let Some(p) = &self.carpeta {
            let ruta = p.join(format!("plano-{:04}.pfm", self.cuadros + 1));
            self.renderizar(gpu, editor, bloque, &ruta)?;
            self.cuadros += 1;
        }
        Ok(())
    }
    pub(crate) fn finalizar(
        &self,
        gpu: &mut telar_gpu::Gpu,
        editor: &EditorHumo,
        bloque: [f32; 32],
    ) -> Result<(), String> {
        if let Some(p) = &self.final_pfm {
            self.renderizar(gpu, editor, bloque, Path::new(p))?;
        }
        Ok(())
    }
    fn renderizar(
        &self,
        gpu: &mut telar_gpu::Gpu,
        editor: &EditorHumo,
        mut bloque: [f32; 32],
        ruta: &Path,
    ) -> Result<(), String> {
        self.validar(editor)?;
        let estado = gpu.inspeccionar_rango_campos(16 * 32u32.pow(3), 17)?;
        if estado.iter().any(|v| !f32::from_bits(*v).is_finite())
            || f32::from_bits(estado[5]) >= 3.0
            || f32::from_bits(estado[14]) >= 2.0
        {
            return Err("estado GPU inválido: no producir render final".into());
        }
        let tiempo = f32::from_bits(estado[13]);
        let orden = crate::volumen_vista::secuencia(editor)?;
        gpu.activar_campos(true)?;
        let resultado = (|| {
            let vistas = if self.pases { 4 } else { 1 };
            for vista in 0..vistas {
                #[allow(clippy::cast_precision_loss, reason = "cuatro pases diagnósticos")]
                {
                    bloque[7] = vista as f32;
                }
                gpu.secuencia_campos(&[17])?;
                gpu.ejecutar_campos(&bloque)?;
                gpu.secuencia_campos(&[16])?;
                for _ in 0..self.muestras {
                    gpu.ejecutar_campos(&bloque)?;
                }
                let datos = gpu.inspeccionar_rango_campos(
                    telar_render::vfx_final::base(32),
                    telar_render::vfx_final::PALABRAS_IMAGEN,
                )?;
                let (pfm, ppm) = crate::salida_hdr::codificar(&datos, [256, 256], self.muestras)?;
                let nombre = if vista == 0 {
                    ruta.to_path_buf()
                } else {
                    let stem = ruta.file_stem().ok_or("salida sin nombre")?.to_string_lossy();
                    ruta.with_file_name(format!(
                        "{stem}-{}.pfm",
                        ["belleza", "calor", "combustible", "particulas"][vista]
                    ))
                };
                let receta = format!(
                    "TELAR VFX · radiancia lineal PFM / revisión Reinhard sRGB PPM
GPU: {}
256x256 · {} spp · pase {}
Tiempo simulado: {tiempo:.8}s
Dispersión simple y sombra de 8 pasos; sin dispersión múltiple ni equivalencia Houdini.
",
                    gpu.dispositivo(),
                    self.muestras,
                    vista
                );
                crate::salida_hdr::guardar(&nombre, &pfm, &ppm, &receta)?;
                println!("telar: HDR VFX {} · {} spp", nombre.display(), self.muestras);
            }
            Ok(())
        })();
        let restaurar = gpu.secuencia_campos(&orden);
        restaurar?;
        resultado
    }
}
fn validar_pfm(ruta: &Path) -> Result<(), String> {
    if !ruta.extension().is_some_and(|v| v.eq_ignore_ascii_case("pfm")) {
        return Err("render final requiere .pfm".into());
    }
    for p in [ruta.to_path_buf(), ruta.with_extension("ppm"), ruta.with_extension("txt")] {
        if p.try_exists().map_err(|e| e.to_string())? {
            return Err("el destino de render ya existe".into());
        }
    }
    Ok(())
}
