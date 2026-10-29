//! E/S explícita y acotada de GEO; no se ejecuta en cuadros estables.
use std::{
    io::{Read, Write},
    path::Path,
};
pub(crate) fn destino(args: &[String], volumen: bool) -> Result<Option<String>, String> {
    let ruta = argumento(args, "--exportar-geo")?;
    if ruta.is_some() && !volumen {
        return Err("--exportar-geo necesita --humo-3d, --explosion o --particulas".into());
    }
    if let Some(ruta) = &ruta {
        validar_destino(Path::new(ruta))?;
    }
    Ok(ruta)
}
pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    if args.iter().any(|v| v == "--exportar-geo")
        && !args.iter().any(|v| {
            v == "--humo-3d" || v == "--explosion" || v == "--particulas" || v == "--plano-vfx" || v == "--abrir-cache"
        })
    {
        return Some(Err("--exportar-geo necesita --humo-3d, --explosion o --particulas".into()));
    }
    if !args.iter().any(|v| v == "--inspeccionar-geo") {
        return None;
    }
    Some((|| {
        if args.iter().any(|v| {
            matches!(
                v.as_str(),
                "--plano-vfx"
                    | "--abrir-cache"
                    | "--guardar-cache"
                    | "--trazadores"
                    | "--exportar-geo"
                    | "--humo"
                    | "--humo-3d"
                    | "--explosion"
                    | "--estudio"
                    | "--fuego"
                    | "--estudio-rayos"
                    | "--particulas"
                    | "--calculo"
                    | "--esfera-cpu"
                    | "--render-rayos"
                    | "--render-objetos"
            )
        }) {
            return Err("inspeccionar GEO requiere un comando independiente".into());
        }
        let ruta = argumento(args, "--inspeccionar-geo")?.ok_or("falta archivo GEO")?;
        match leer(Path::new(&ruta))? {
            ArchivoGeo::Muestras(puntos) => {
                println!(
                    "telar: GEO clásico V5 · {} muestras · P/v/density/heat/id/sim_time · comparación Houdini ABIERTA",
                    puntos.len()
                );
                println!("telar: {}", resumen(&puntos));
            }
            ArchivoGeo::Particulas(puntos) => {
                let base: Vec<_> = puntos.iter().map(|p| p.punto).collect();
                let edad = puntos.iter().map(|p| p.edad).fold(0.0, f32::max);
                let vida = puntos.iter().map(|p| p.vida).fold(0.0, f32::max);
                println!(
                    "telar: GEO clásico V5 · {} partículas vivas · age/life/birth_time · edad máx {edad} · vida máx {vida} · comparación Houdini ABIERTA",
                    puntos.len()
                );
                println!("telar: {}", resumen(&base));
            }
        }
        Ok(())
    })())
}
fn resumen(puntos: &[telar_intercambio::Punto]) -> String {
    if puntos.is_empty() {
        return "nube vacía".into();
    }
    let mut minimo = [f32::INFINITY; 3];
    let mut maximo = [f32::NEG_INFINITY; 3];
    let mut densidad = [f32::INFINITY, f32::NEG_INFINITY];
    let mut calor = densidad;
    let mut tiempo = densidad;
    for p in puntos {
        for eje in 0..3 {
            minimo[eje] = minimo[eje].min(p.posicion[eje]);
            maximo[eje] = maximo[eje].max(p.posicion[eje]);
        }
        for (rango, valor) in [(&mut densidad, p.densidad), (&mut calor, p.calor), (&mut tiempo, p.tiempo)] {
            rango[0] = rango[0].min(valor);
            rango[1] = rango[1].max(valor);
        }
    }
    format!("P {minimo:?}..{maximo:?} · densidad {densidad:?} · calor {calor:?} · tiempo {tiempo:?}")
}
pub(crate) fn exportar(gpu: &telar_gpu::Gpu, lado: u32, ruta: &Path) -> Result<usize, String> {
    validar_destino(ruta)?;
    if lado < 4 {
        return Err("lado GEO menor que cuatro".into());
    }
    let celdas = lado.checked_pow(3).filter(|v| *v <= 65_536).ok_or("volumen demasiado grande para GEO")?;
    let control = gpu.inspeccionar_rango_campos(16 * celdas, 16)?;
    if control.iter().any(|v| !f32::from_bits(*v).is_finite()) || !matches!(f32::from_bits(control[5]), 0.0 | 1.0 | 2.0)
    {
        return Err("solver inválido; no se exporta GEO".into());
    }
    let campos = gpu.inspeccionar_rango_campos(0, 5 * celdas)?;
    let puntos = crate::puntos_mac::convertir(lado, &campos, f32::from_bits(control[13]))?;
    let texto = telar_intercambio::codificar_geo(&puntos)?;
    let mut archivo = std::fs::OpenOptions::new().write(true).create_new(true).open(ruta).map_err(|e| e.to_string())?;
    archivo.write_all(texto.as_bytes()).map_err(|e| format!("escribir GEO: {e}"))?;
    archivo.sync_all().map_err(|e| e.to_string())?;
    Ok(puntos.len())
}
pub(crate) enum ArchivoGeo {
    Muestras(Vec<telar_intercambio::Punto>),
    Particulas(Vec<telar_intercambio::Particula>),
}
pub(crate) fn leer(ruta: &Path) -> Result<ArchivoGeo, String> {
    extension(ruta)?;
    let mut bytes = Vec::new();
    std::fs::File::open(ruta)
        .map_err(|e| e.to_string())?
        .take(16_777_217)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let texto = core::str::from_utf8(&bytes).map_err(|_| "GEO requiere texto UTF-8 ASCII")?;
    if let Ok(puntos) = telar_intercambio::decodificar_geo(texto) {
        return Ok(ArchivoGeo::Muestras(puntos));
    }
    telar_intercambio::decodificar_particulas_geo(texto).map(ArchivoGeo::Particulas).map_err(Into::into)
}
pub(crate) fn validar_destino(ruta: &Path) -> Result<(), String> {
    extension(ruta)?;
    if ruta.try_exists().map_err(|e| e.to_string())? {
        return Err("el destino GEO ya existe; elige otro nombre".into());
    }
    Ok(())
}
fn extension(ruta: &Path) -> Result<(), String> {
    if !ruta.extension().is_some_and(|v| v.eq_ignore_ascii_case("geo")) {
        return Err("se requiere extensión .geo (texto clásico)".into());
    }
    Ok(())
}
fn argumento(args: &[String], nombre: &str) -> Result<Option<String>, String> {
    let mut posiciones = args.iter().enumerate().filter(|(_, v)| v.as_str() == nombre);
    let Some((indice, _)) = posiciones.next() else {
        return Ok(None);
    };
    if posiciones.next().is_some() {
        return Err(format!("{nombre} repetido"));
    }
    Ok(Some(
        args.get(indice + 1)
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("falta ruta para {nombre}"))?
            .clone(),
    ))
}

pub(crate) fn salida(gpu: &telar_gpu::Gpu, lado: u32, destino: Option<&str>) -> Result<(), String> {
    if let Some(destino) = destino {
        let puntos = exportar(gpu, lado, Path::new(destino))?;
        println!("telar: {puntos} muestras de campos exportadas a {destino}");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comandos_rechazan_ambiguedad_y_esquema_de_ejemplo_se_inspecciona() -> Result<(), String> {
        let args = |v: &[&str]| v.iter().map(|v| (*v).to_owned()).collect::<Vec<_>>();
        for valores in [
            &["--inspeccionar-geo"][..],
            &["--exportar-geo", "a.geo"][..],
            &["--inspeccionar-geo", "a.geo", "--explosion"][..],
            &["--inspeccionar-geo", "a.geo", "--inspeccionar-geo", "b.geo"][..],
        ] {
            assert!(procesar(&args(valores)).is_some_and(|v| v.is_err()));
        }
        assert!(destino(&args(&["--exportar-geo", "a.bgeo"]), true).is_err());
        assert!(destino(&args(&["--exportar-geo", "a.geo"]), false).is_err());
        let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ejemplos/muestras-telar.geo");
        let ArchivoGeo::Muestras(puntos) = leer(&ruta)? else {
            return Err("perfil inesperado".into());
        };
        assert_eq!(puntos.len(), 2);
        assert!(resumen(&puntos).contains("0.125"));
        Ok(())
    }
}
