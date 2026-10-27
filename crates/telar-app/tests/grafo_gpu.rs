//! El grafo editable se evalúa en GPU contra ecuaciones independientes.
#![allow(clippy::unwrap_used, clippy::cast_precision_loss, reason = "referencias pequeñas de prueba")]
#[path = "../src/arranque.rs"]
mod arranque;
#[path = "../src/grafo_compilar.rs"]
mod grafo_compilar;
#[allow(dead_code, reason = "codec compartido; E/S y CLI probados en proyecto.rs")]
#[path = "../src/proyecto.rs"]
mod proyecto;
use telar_gpu::{
    Calculo, DominioProcedural, EtapaProcedural, Opciones, ParametrosNodos, ParametrosProcedurales, PlanProcedural,
};
use telar_ui::{Estudio, TipoNodo};

fn comprobar(gpu: &Calculo, estudio: &Estudio, dos_ondas: bool, relieve_final: bool) {
    let programa = grafo_compilar::compilar(estudio).unwrap();
    let codigo = [telar_kernel::compilar_con_cache(), hilo_spirv::rejilla::generar(), programa.compilar()];
    let etapas = [
        EtapaProcedural { nombre: "preparar", codigo: &codigo[0], dominio: DominioProcedural::Preparar },
        EtapaProcedural { nombre: "generar", codigo: &codigo[1], dominio: DominioProcedural::Generar },
        EtapaProcedural { nombre: "grafo", codigo: &codigo[2], dominio: DominioProcedural::Puntos },
    ];
    let cabecera = arranque::crear(gpu.perfil(), 17, etapas.len()).unwrap();
    let mut residente =
        gpu.procedural_con_plan(17, PlanProcedural::nuevo(&etapas).unwrap().con_arranque(&cabecera).unwrap()).unwrap();
    let inicial = residente.actividad();
    for fase in [0.0, 0.6, 1.9, 1.9] {
        residente
            .paso_nodos(ParametrosNodos {
                superficie: ParametrosProcedurales { lado: 17, fase, amplitud: 0.35 },
                valores: [1.3, 0.0, 0.0, 0.0],
            })
            .unwrap();
        let datos = residente.inspeccionar().unwrap();
        for v in &datos.vertices {
            let x = v[0];
            let z = v[2];
            let h1 = 0.35 * (2.0 * x + fase).sin() * (2.0 * z).cos();
            let h2 = if dos_ondas { 0.2 * (2.0 * x + fase * 0.7).sin() * (2.0 * z).cos() } else { 0.0 };
            let esperado = if relieve_final { (h1 + h2) * 1.3 } else { h1 * 1.3 + h2 };
            assert!((v[1] - esperado).abs() < 0.00003, "altura GPU {} != ecuación {esperado}", v[1]);
            let gradiente = |amplitud: f32, velocidad: f32| {
                [
                    2.0 * amplitud * (2.0 * x + fase * velocidad).cos() * (2.0 * z).cos(),
                    -2.0 * amplitud * (2.0 * x + fase * velocidad).sin() * (2.0 * z).sin(),
                ]
            };
            let g1 = gradiente(0.35, 1.0);
            let g2 = gradiente(if dos_ondas { 0.2 } else { 0.0 }, 0.7);
            let g = if relieve_final {
                [(g1[0] + g2[0]) * 1.3, (g1[1] + g2[1]) * 1.3]
            } else {
                [g1[0] * 1.3 + g2[0], g1[1] * 1.3 + g2[1]]
            };
            let normal = nudo::mate::normalizar([-g[0], 1.0, -g[1]]);
            for i in 0..3 {
                assert!((v[i + 3] - normal[i]).abs() < 0.00003);
            }
        }
    }
    let finalizado = residente.actividad();
    assert_eq!(
        (finalizado.buffers, finalizado.pipelines, finalizado.grabaciones, finalizado.mapeos),
        (inicial.buffers, inicial.pipelines, inicial.grabaciones, inicial.mapeos)
    );
    assert_eq!(finalizado.geometria_subida, 0);
}

#[test]
fn dos_ondas_reconexion_y_bypass_cambian_calculo_sin_geometria_cpu() {
    let gpu = Calculo::nuevo(&Opciones::default()).unwrap();
    let mut estudio = Estudio::default();
    estudio.configurar_capacidad(17);
    let onda = estudio.crear_nodo(TipoNodo::Onda).unwrap();
    comprobar(&gpu, &estudio, true, false);
    estudio.conectar_nodos(1, onda).unwrap();
    estudio.conectar_nodos(onda, 2).unwrap();
    estudio.conectar_nodos(2, 3).unwrap();
    comprobar(&gpu, &estudio, true, true);
    estudio.habilitar_nodo(onda, false).unwrap();
    comprobar(&gpu, &estudio, false, true);
    drop(gpu);
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}

#[test]
fn ocho_operaciones_caben_en_ir_y_spirv_valida_el_modulo_mas_grande() {
    let mut estudio = Estudio::default();
    while estudio.cantidad_nodos() < 10 {
        estudio.crear_nodo(TipoNodo::Onda).unwrap();
    }
    let programa = grafo_compilar::compilar(&estudio).unwrap();
    assert!(programa.operaciones().len() <= 256);
    let ruta = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/grafo-maximo.spv");
    std::fs::write(&ruta, programa.compilar().iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<_>>()).unwrap();
    let validador = std::env::var_os("VULKAN_SDK")
        .map(|p| std::path::PathBuf::from(p).join("Bin/spirv-val.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| "spirv-val".into());
    let resultado =
        std::process::Command::new(validador).args(["--target-env", "vulkan1.3"]).arg(ruta).output().unwrap();
    assert!(resultado.status.success(), "{}", String::from_utf8_lossy(&resultado.stderr));
}

#[test]
fn escena_v3_reabre_nodos_y_ejecutable_preserva_bytes_en_pausa() {
    let mut estudio = Estudio::default();
    estudio.configurar_capacidad(17);
    estudio.crear_nodo(TipoNodo::Onda).unwrap();
    estudio.pausado = true;
    estudio.salida.muestras = 2;
    let proyecto = proyecto::Proyecto::capturar(&estudio, &nudo::Camara::default(), 0.6);
    let original = proyecto.codificar().unwrap();
    assert_eq!(&original[4..8], &3u32.to_le_bytes());
    let entrada = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/grafo-v3.tlar");
    let salida = entrada.with_file_name("grafo-v3-salida.tlar");
    std::fs::write(&entrada, &original).unwrap();
    let resultado = std::process::Command::new(env!("CARGO_BIN_EXE_telar"))
        .args(["--prueba", "--abrir-escena"])
        .arg(entrada)
        .arg("--guardar-escena")
        .arg(&salida)
        .output()
        .unwrap();
    let texto = String::from_utf8_lossy(&resultado.stdout);
    assert!(resultado.status.success(), "{texto}\n{}", String::from_utf8_lossy(&resultado.stderr));
    assert!(texto.contains("red compilada · 5 nodos · 3 operaciones alcanzables"));
    assert!(texto.contains("120 fotogramas · 0 avisos de validación"));
    assert_eq!(std::fs::read(salida).unwrap(), original);
}
