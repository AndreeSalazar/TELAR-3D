//! Primer render de mallas por lotes; controles numéricos explícitos por comando.
use std::path::Path;
use telar_render::escena_rayos::Trabajo;

pub(crate) fn procesar(args: &[String]) -> Option<Result<(), String>> {
    args.iter().any(|a| a == "--render-rayos").then(|| ejecutar(args))
}

fn argumento<'a>(args: &'a [String], nombre: &str) -> Result<Option<&'a str>, String> {
    let mut indices = args.iter().enumerate().filter(|(_, a)| a.as_str() == nombre);
    let Some((indice, _)) = indices.next() else { return Ok(None) };
    if indices.next().is_some() {
        return Err(format!("opción duplicada: {nombre}"));
    }
    args.get(indice + 1)
        .filter(|v| !v.starts_with("--"))
        .map(|v| Some(v.as_str()))
        .ok_or_else(|| format!("{nombre} requiere un valor"))
}

fn numero<T: std::str::FromStr>(args: &[String], nombre: &str, base: T) -> Result<T, String> {
    argumento(args, nombre)?.map_or(Ok(base), |v| v.parse().map_err(|_| format!("número inválido para {nombre}")))
}

fn vector(args: &[String], nombre: &str, base: [f32; 3]) -> Result<[f32; 3], String> {
    let Some(valor) = argumento(args, nombre)? else { return Ok(base) };
    let partes: Vec<_> = valor.split(',').map(str::parse::<f32>).collect();
    match partes.as_slice() {
        [Ok(x), Ok(y), Ok(z)] if [x, y, z].iter().all(|v| v.is_finite() && v.abs() <= 100.0) => Ok([*x, *y, *z]),
        _ => Err(format!("{nombre} requiere x,y,z finitos entre -100 y 100")),
    }
}

#[allow(clippy::cast_precision_loss, reason = "resolución validada <=512")]
fn ejecutar(args: &[String]) -> Result<(), String> {
    validar_opciones(args)?;
    let destino = argumento(args, "--render-rayos")?.ok_or("falta salida.pfm")?;
    let ruta = Path::new(destino);
    if ruta.extension().and_then(|v| v.to_str()) != Some("pfm") {
        return Err("--render-rayos requiere salida.pfm".into());
    }
    let ancho = numero(args, "--ancho", 128_u32)?;
    let alto = numero(args, "--alto", 96_u32)?;
    let desplazamiento = numero(args, "--objeto-x", 0.0_f32)?;
    if !desplazamiento.is_finite() || !(-2.0..=2.0).contains(&desplazamiento) {
        return Err("--objeto-x admite -2..2".into());
    }
    let exposicion = numero(args, "--exposicion", 1.0_f32)?;
    if !exposicion.is_finite() || !(0.01..=16.0).contains(&exposicion) {
        return Err("--exposicion admite 0.01..16".into());
    }
    let ojo = vector(args, "--ojo", [4.0, 3.0, 6.0])?;
    let objetivo = vector(args, "--mirar", [0.0, 0.5, 0.0])?;
    let trabajo = Trabajo {
        tamano: [ancho, alto],
        exposicion,
        muestras: numero(args, "--spp", 16)?,
        semilla: numero(args, "--semilla", 1)?,
        camara: crate::escena_final::camara(ojo, objetivo, ancho as f32 / alto.max(1) as f32)?,
        triangulos: crate::escena_final::triangulos(desplazamiento),
        luz: vector(args, "--luz", [-1.0, 4.0, 1.0])?,
        radio_luz: numero(args, "--radio-luz", 0.65)?,
        energia: numero(args, "--energia", 25.0)?,
        color_luz: [1.0; 3],
        sombras: !args.iter().any(|a| a == "--sin-sombras"),
    };
    let codigo = trabajo.compilar()?;
    let gpu = telar_gpu::Calculo::nuevo(&telar_gpu::Opciones {
        validacion: !args.iter().any(|a| a == "--sin-validacion"),
        ..telar_gpu::Opciones::default()
    })?;
    println!(
        "telar: render directo · {} · {ancho}×{alto} · {} spp · {} triángulos · sombras {}",
        gpu.dispositivo(),
        trabajo.muestras,
        trabajo.triangulos.len(),
        trabajo.sombras
    );
    let resultado = gpu.ejecutar(&codigo, ancho * alto * 5, 64)?;
    let (pfm, ppm) = crate::salida_hdr::codificar(&resultado.datos, trabajo.tamano, trabajo.muestras)?;
    let receta = format!(
        "TELAR · iluminación directa experimental, sin GI ni BVH\nGPU: {}\nTamaño: {ancho},{alto}\nSpp: {}\nSemilla: {}\nOjo: {ojo:?}\nMirar: {objetivo:?}\nLuz: {:?}\nRadio luz: {}\nEnergía: {}\nSombras: {}\nObjeto x: {desplazamiento}\nExposición PPM: {exposicion}\nGPU ns (solo despacho): {:?}\n",
        gpu.dispositivo(),
        trabajo.muestras,
        trabajo.semilla,
        trabajo.luz,
        trabajo.radio_luz,
        trabajo.energia,
        trabajo.sombras,
        resultado.nanosegundos
    );
    drop(gpu);
    let avisos = telar_gpu::avisos_de_validacion();
    if avisos != 0 {
        return Err(format!("render rechazado: {avisos} avisos de validación"));
    }
    crate::salida_hdr::guardar(ruta, &pfm, &ppm, &receta)?;
    println!(
        "telar: HDR {} · vista {} · receta {} · 0 avisos de validación",
        ruta.display(),
        ruta.with_extension("ppm").display(),
        ruta.with_extension("txt").display()
    );
    Ok(())
}

fn validar_opciones(args: &[String]) -> Result<(), String> {
    const VALORES: [&str; 12] = [
        "--render-rayos",
        "--ancho",
        "--alto",
        "--spp",
        "--semilla",
        "--objeto-x",
        "--exposicion",
        "--ojo",
        "--mirar",
        "--luz",
        "--radio-luz",
        "--energia",
    ];
    let mut indice = 0;
    while indice < args.len() {
        let opcion = &args[indice];
        if matches!(opcion.as_str(), "--sin-sombras" | "--sin-validacion") {
            indice += 1;
        } else if VALORES.contains(&opcion.as_str()) {
            argumento(args, opcion)?;
            indice += 2;
        } else {
            return Err(format!("opción incompatible con render de rayos: {opcion}"));
        }
    }
    Ok(())
}
