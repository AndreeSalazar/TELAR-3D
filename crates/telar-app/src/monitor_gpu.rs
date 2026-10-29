//! Monitor de ventana: refresco 2 Hz, estadísticas acotadas y exportación explícita al cerrar.
use std::{
    collections::VecDeque,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use telar_gpu::Gpu;
use telar_ui::{EditorHumo, Monitor};
pub(crate) struct Observador {
    activo: bool,
    control: Option<u32>,
    refresco: Instant,
    inicio: Instant,
    anterior: Option<Instant>,
    cuadros: VecDeque<f64>,
    host: VecDeque<f64>,
    fases_host: VecDeque<telar_gpu::TelemetriaHost>,
    gpu: VecDeque<[f64; 3]>,
    muestra: u64,
    archivo: Option<PathBuf>,
    csv: Vec<String>,
    detalle: String,
    temporal: bool,
}
impl Observador {
    pub(crate) fn nuevo(args: &[String], editor: &EditorHumo, gpu: &mut Gpu) -> Result<Self, String> {
        // ABI MAC 3D: física en 0..15 y los componentes W 19/23/27/31; cámara XYZ solo gráfica.
        let mascara = if editor.volumen && editor.particulas.is_none() { 0x8888_ffff } else { u32::MAX };
        gpu.configurar_cache_campos(!args.iter().any(|v| v == "--sin-cache-compute"), mascara)?;
        if editor.particulas.is_none() {
            let grupos = gpu.carga_campos().grupos;
            let indirectos = !args.iter().any(|v| v == "--sin-indirectos");
            let mut rutas = vec![
                (7, telar_gpu::RutaCampos::Directo(grupos.div_ceil(64))),
                (8, telar_gpu::RutaCampos::Productor(1)),
                (13, telar_gpu::RutaCampos::Productor(1)),
            ];
            for (programa, offset) in
                telar_kernel::humo_indirecto::PROGRAMAS.into_iter().zip(telar_kernel::humo_indirecto::OFFSETS)
            {
                let ruta = if indirectos {
                    telar_gpu::RutaCampos::Indirecto(offset)
                } else {
                    telar_gpu::RutaCampos::Directo(if programa == 12 { grupos.div_ceil(64) } else { grupos })
                };
                rutas.push((programa, ruta));
            }
            if editor.volumen {
                rutas.extend([
                    (14, telar_gpu::RutaCampos::Directo(grupos.div_ceil(64))),
                    (15, telar_gpu::RutaCampos::Directo(1)),
                ]);
            }
            gpu.configurar_despachos_campos(&rutas)?;
        }
        let archivo = crate::estudio_humo::valor(args, "--benchmark-gpu")?.map(PathBuf::from);
        if archivo.as_ref().is_some_and(|p| p.exists() || p.with_extension("txt").exists()) {
            return Err("el informe GPU ya existe".into());
        }
        if archivo.as_ref().is_some_and(|p| !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("csv"))) {
            return Err("el benchmark requiere destino .csv".into());
        }
        if archivo.is_some() && args.iter().any(|v| v == "--sin-telemetria") {
            return Err("--benchmark-gpu requiere telemetría activa".into());
        }
        let activo = (editor.vfx.is_some() || archivo.is_some() || args.iter().any(|v| v == "--telemetria"))
            && !args.iter().any(|v| v == "--sin-telemetria");
        let control = if editor.particulas.is_some() {
            None
        } else if editor.volumen {
            Some(16 * 32u32.pow(3))
        } else {
            Some(14 * 128u32.pow(2))
        };
        gpu.activar_telemetria(activo, control)?;
        let ahora = Instant::now();
        Ok(Self {
            activo,
            control,
            refresco: ahora,
            inicio: ahora,
            anterior: None,
            cuadros: VecDeque::new(),
            host: VecDeque::new(),
            fases_host: VecDeque::new(),
            gpu: VecDeque::new(),
            muestra: 0,
            archivo,
            csv: Vec::new(),
            detalle: String::new(),
            temporal: editor.volumen && editor.particulas.is_none(),
        })
    }
    pub(crate) fn alternar(&mut self, editor: &mut EditorHumo, gpu: &mut Gpu) -> Result<(), String> {
        self.activo = !self.activo;
        gpu.activar_telemetria(self.activo, self.control)?;
        editor.monitor = None;
        self.anterior = None;
        self.cuadros.clear();
        self.host.clear();
        self.fases_host.clear();
        self.gpu.clear();
        Ok(())
    }
    pub(crate) fn cuadro(&mut self, gpu: &Gpu, costo_host: Duration) {
        if !self.activo {
            return;
        }
        let ahora = Instant::now();
        if let Some(antes) = self.anterior {
            agregar(&mut self.cuadros, ahora.duration_since(antes).as_secs_f64() * 1000.0);
        }
        self.anterior = Some(ahora);
        agregar(&mut self.host, costo_host.as_secs_f64() * 1000.0);
        if let Some(host) = gpu.telemetria_host() {
            agregar(&mut self.fases_host, host);
        }
        let muestra = gpu.telemetria();
        if muestra.muestras != self.muestra {
            self.muestra = muestra.muestras;
            if let Some(ms) = muestra.milisegundos {
                agregar(&mut self.gpu, ms);
            }
        }
    }
    pub(crate) fn actualizar(&mut self, editor: &mut EditorHumo, gpu: &Gpu, forzar: bool) -> bool {
        if !self.activo {
            editor.monitor = None;
            return false;
        }
        if !forzar && editor.monitor.is_some() && self.refresco.elapsed() < Duration::from_millis(500) {
            return false;
        }
        self.refresco = Instant::now();
        let mut lineas = vec!["MONITOR VIVO · M: ocultar".into(), gpu.dispositivo().into()];
        let (frame, p95) = estadisticas(&self.cuadros);
        let fps = if frame > 0.0 { 1000.0 / frame } else { 0.0 };
        if self.cuadros.is_empty() {
            lineas.push("FPS/cuadro/P95: pendientes".into());
        } else {
            lineas.push(format!("{fps:.1} fps · cuadro {frame:.2} ms"));
            lineas.push(format!("P95 cuadro {p95:.2} ms"));
        }
        lineas.push(crate::monitor_host::resumen(&self.host));
        let fases_host = crate::monitor_host::media(&self.fases_host);
        lineas.extend(crate::monitor_host::lineas(fases_host));
        let tiempos = media_gpu(&self.gpu);
        if let Some([calculo, escena, interfaz]) = tiempos {
            lineas.push(format!("GPU cálculo medio {calculo:.3} ms"));
            lineas.push(format!("GPU escena media {escena:.3} ms"));
            lineas.push(format!("GPU UI/final media {interfaz:.3} ms"));
        } else {
            lineas.push("Tiempos GPU: N/D o pendientes".into());
        }
        let mut estado = 0;
        let heaps = gpu.memoria_heaps();
        for (i, heap) in heaps.iter().enumerate() {
            let tipo = if heap.local { "local" } else { "host" };
            lineas.push(format!("Heap {i} {tipo} · {} MiB", heap.capacidad / 1_048_576));
            if let (Some(uso), Some(presupuesto)) = (heap.uso, heap.presupuesto) {
                lineas.push(format!("Uso~ {} / {} MiB~", uso / 1_048_576, presupuesto / 1_048_576));
                if presupuesto > 0 {
                    estado = estado.max(if uso >= presupuesto.saturating_mul(9) / 10 {
                        2
                    } else {
                        usize::from(uso >= presupuesto.saturating_mul(7) / 10)
                    });
                }
            } else {
                lineas.push("Uso/presupuesto: N/D".into());
            }
        }
        let carga = gpu.carga_campos();
        lineas.push(format!("Campos lógicos {} KiB", carga.bytes / 1024));
        lineas.push(format!("{} ranuras · máx {} grupos", carga.ranuras, carga.grupos));
        lineas.push(format!("I{} · caché {}g/{}r", carga.indirectos, carga.grabaciones, carga.reusos));
        let perfil = gpu.perfil();
        lineas.push(format!("Límite {} hilos/grupo", perfil.invocaciones));
        lineas.push(format!("Compartida {} KiB/grupo", perfil.compartida / 1024));
        let muestra = gpu.telemetria();
        if let Some(c) = muestra.control {
            if c.iter().any(|v| !v.is_finite()) || c[5] >= 3.0 || (editor.volumen && c[14] >= 2.0) {
                estado = 2;
            } else if c[5].to_bits() == 2.0f32.to_bits() {
                estado = estado.max(1);
            }
            lineas.push(format!(
                "Presión {} · {:.0}/{:.0} iter",
                match c[5] {
                    1.0 => "OK",
                    2.0 => "límite",
                    3.0 => "inválida",
                    _ => "espera",
                },
                c[1],
                editor.humo.0[7]
            ));
            lineas.push(format!("Residual {:.4} / {:.4}", c[3], c[4]));
            lineas.push(format!("Dens {:.2} · calor {:.2}", c[6], c[7]));
            if editor.volumen {
                lineas.push(format!("dt {:.5} · CFL {:.3}", c[10], c[12]));
                lineas.push(format!("Reloj GPU {:.3} s", c[13]));
            }
        }
        lineas.push("Temperatura/vatios/reloj: N/D".into());
        lineas.push("Ocupación global GPU: N/D".into());
        lineas.push("~ estimación · ventana 120 cuadros".into());
        self.registrar(fps, frame, tiempos, &heaps, gpu);
        self.detalle = format!(
            "TELAR · monitor vivo\nValidación Vulkan: {}\nFormato/tamaño: {:?}\nPerfil (límites; no velocidad): {perfil:#?}\nHeaps (estimaciones de proceso): {heaps:#?}\nControl GPU: {:?}\nHumo: {:?}\nTiempo/esfera: {:?}\nPulso: {:?}\nVFX: {:?}\nPartículas: {:?}\n{}\nNo mide temperatura, potencia, frecuencia ni ocupación global.\n",
            gpu.validacion(),
            gpu.imagenes(),
            muestra.control,
            editor.humo.0,
            editor.volumen_opciones.0,
            editor.explosion.0,
            editor.vfx,
            editor.particulas.as_ref().map(|p| p.0),
            lineas.join("\n")
        );
        #[allow(clippy::cast_possible_truncation, reason = "gráfico UI de milisegundos finitos")]
        let historial = self.gpu.iter().map(|ms| ms.iter().sum::<f64>() as f32).collect();
        editor.monitor = Some(Monitor { lineas, estado, historial });
        true
    }
    fn registrar(&mut self, fps: f64, frame: f64, ms: Option<[f64; 3]>, heaps: &[telar_gpu::MemoriaHeap], gpu: &Gpu) {
        if self.archivo.is_none() || self.csv.len() >= 4096 {
            return;
        }
        let control = gpu.telemetria().control;
        let host = crate::monitor_host::media(&self.fases_host);
        let carga = gpu.carga_campos();
        let numero = |v: Option<f64>| v.map_or_else(String::new, |v| format!("{v:.6}"));
        let heap = heaps.iter().find(|h| h.local);
        self.csv.push(format!(
            "{:.6},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            self.inicio.elapsed().as_secs_f64(),
            numero((frame > 0.0).then_some(fps)),
            numero((frame > 0.0).then_some(frame)),
            numero(ms.map(|v| v[0])),
            numero(ms.map(|v| v[1])),
            numero(ms.map(|v| v[2])),
            heap.and_then(|h| h.uso).map_or_else(String::new, |v| v.to_string()),
            heap.and_then(|h| h.presupuesto).map_or_else(String::new, |v| v.to_string()),
            numero(control.map(|c| f64::from(c[3]))),
            numero(control.filter(|_| self.temporal).map(|c| f64::from(c[13]))),
            crate::monitor_host::csv(host),
            numero(crate::monitor_host::promedio(&self.host)),
            carga.grabaciones,
            carga.reusos,
            carga.directas
        ));
    }
    pub(crate) fn finalizar(&self) -> Result<(), String> {
        if let Some(path) = &self.archivo {
            let mut detalle = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.with_extension("txt"))
                .map_err(|e| e.to_string())?;
            detalle.write_all(self.detalle.as_bytes()).map_err(|e| e.to_string())?;
            let mut archivo =
                std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
            archivo.write_all(b"segundos,fps,cuadro_ms,gpu_calculo_ms,gpu_escena_ms,gpu_ui_ms,uso_local_estimado_B,presupuesto_local_estimado_B,residual,reloj_sim_s,host_prep_ms,host_valla_ms,host_obs_ms,host_adquirir_ms,host_grabar_ms,host_enviar_ms,host_presentar_ms,host_total_medio_ms,compute_grabaciones,compute_reusos,compute_directas\n").map_err(|e| e.to_string())?;
            for fila in &self.csv {
                archivo.write_all(fila.as_bytes()).map_err(|e| e.to_string())?;
            }
            println!(
                "telar: benchmark GPU {} · {} muestras · estimaciones del driver marcadas",
                path.display(),
                self.csv.len()
            );
        }
        Ok(())
    }
}
#[allow(clippy::cast_precision_loss, reason = "ventana de hasta 120 muestras GPU")]
fn media_gpu(muestras: &VecDeque<[f64; 3]>) -> Option<[f64; 3]> {
    (!muestras.is_empty())
        .then(|| core::array::from_fn(|i| muestras.iter().map(|m| m[i]).sum::<f64>() / muestras.len() as f64))
}
fn agregar<T>(cola: &mut VecDeque<T>, valor: T) {
    if cola.len() == 120 {
        cola.pop_front();
    }
    cola.push_back(valor);
}
#[allow(clippy::cast_precision_loss, reason = "hasta 120 cuadros por ventana estadística")]
fn estadisticas(cola: &VecDeque<f64>) -> (f64, f64) {
    if cola.is_empty() {
        return (0.0, 0.0);
    }
    let mut valores: Vec<_> = cola.iter().copied().collect();
    valores.sort_by(f64::total_cmp);
    (valores.iter().sum::<f64>() / valores.len() as f64, valores[(valores.len() * 95).div_ceil(100).saturating_sub(1)])
}
