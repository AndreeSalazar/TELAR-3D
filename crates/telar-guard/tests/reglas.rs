//! Cada regla, contra un repositorio de prueba que la rompe: tiene que
//! bloquear, decir dónde y no confundirse con lo que solo lo parece
//! (un `unsafe` en un comentario, un `dbg!` en una cadena…).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "en los tests, un fallo es un panic con su mensaje")]

use std::fs;
use std::path::PathBuf;

use telar_guard::informe::Problema;
use telar_guard::reglas;
use telar_guard::repo::Repo;

/// Un repositorio mínimo y correcto en una carpeta temporal; se borra al soltarlo.
struct Prueba {
    raiz: PathBuf,
}

impl Prueba {
    fn nueva(nombre: &str) -> Self {
        let raiz = std::env::temp_dir().join(format!("telar-guard-{nombre}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&raiz);
        let p = Self { raiz };
        p.escribir("Cargo.toml", "[workspace]\nresolver = \"3\"\nmembers = [\"crates/ejemplo\"]\n");
        p.escribir(
            "crates/ejemplo/Cargo.toml",
            "[package]\nname = \"ejemplo\"\nversion = \"0.1.0\"\n\n[package.metadata.telar]\npuede-usar = [\"hermano\"]\n\n[dependencies]\nhermano = { path = \"../hermano\" }\n",
        );
        p.escribir("crates/ejemplo/README.md", "# ejemplo\n");
        p.escribir(
            "crates/ejemplo/src/lib.rs",
            "//! Ejemplo.\n\n/// Suma.\npub fn suma(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        );
        p.escribir(
            "crates/ejemplo/tests/suma.rs",
            "#[test]\nfn suma() {\n    assert_eq!(ejemplo::suma(1, 2), 3);\n}\n",
        );
        for carpeta in ["", "crates/", "crates/ejemplo/", "crates/ejemplo/src/", "crates/ejemplo/tests/"] {
            p.escribir(&format!("{carpeta}SEMAFORO.md"), SEMAFORO);
        }
        p
    }

    fn escribir(&self, rel: &str, texto: &str) {
        let ruta = self.raiz.join(rel);
        fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        fs::write(ruta, texto).unwrap();
    }

    fn borrar(&self, rel: &str) {
        fs::remove_file(self.raiz.join(rel)).unwrap();
    }

    /// Los problemas de una regla.
    fn problemas(&self, regla: &str) -> Vec<Problema> {
        let repo = Repo::abrir(&self.raiz).unwrap();
        reglas::revisar(&repo).into_iter().find(|r| r.regla.id == regla).unwrap().problemas
    }

    /// Los sitios (`archivo:línea`) de una regla.
    fn sitios(&self, regla: &str) -> Vec<String> {
        self.problemas(regla).iter().map(Problema::sitio).collect()
    }

    /// Las reglas rotas.
    fn rotas(&self) -> Vec<&'static str> {
        let repo = Repo::abrir(&self.raiz).unwrap();
        reglas::revisar(&repo).into_iter().filter(|r| !r.problemas.is_empty()).map(|r| r.regla.id).collect()
    }
}

impl Drop for Prueba {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.raiz);
    }
}

const LIB: &str = "crates/ejemplo/src/lib.rs";
const SEMAFORO: &str = "Estado: VERDE\nMotivo: prueba aislada\nCPU: solo pruebas\nGPU: cero directo\nVRAM: cero directo\nRAM: texto pequeño\nE/S: archivos temporales\nFrecuencia: al verificar\nLimites: carpeta de prueba\nEvidencia: pruebas del guardián\n";
const FFI: &str = "crates/ejemplo/src/ffi/mod.rs";

#[test]
fn un_repo_correcto_pasa_todas() {
    assert_eq!(Prueba::nueva("limpio").rotas(), Vec::<&str>::new());
}

#[test]
fn semaforo_obligatorio_con_consumo_y_riesgo_coherente() {
    let p = Prueba::nueva("semaforo");
    p.borrar("crates/ejemplo/src/SEMAFORO.md");
    assert!(p.sitios("semaforos").iter().any(|s| s.contains("src/SEMAFORO.md")));
    p.escribir("crates/ejemplo/src/SEMAFORO.md", &SEMAFORO.replace("CPU: solo pruebas\n", ""));
    assert!(p.problemas("semaforos").iter().any(|p| p.detalle.contains("CPU")));
    p.escribir("crates/ejemplo/src/SEMAFORO.md", SEMAFORO);
    p.escribir(FFI, "//! frontera de prueba\n");
    p.escribir("crates/ejemplo/src/ffi/SEMAFORO.md", SEMAFORO);
    assert!(p.problemas("semaforos").iter().any(|p| p.detalle.contains("FFI")));
    p.escribir("crates/ejemplo/src/ffi/SEMAFORO.md", &SEMAFORO.replace("VERDE", "ROJO"));
    assert!(!p.problemas("semaforos").is_empty(), "los padres también deben reflejar el riesgo");
    for carpeta in ["", "crates/", "crates/ejemplo/", "crates/ejemplo/src/"] {
        p.escribir(&format!("{carpeta}SEMAFORO.md"), &SEMAFORO.replace("VERDE", "ROJO"));
    }
    assert_eq!(p.sitios("semaforos"), Vec::<String>::new());
}

#[test]
fn cero_deps_bloquea_crates_de_terceros() {
    let p = Prueba::nueva("deps");
    p.escribir(
        "crates/ejemplo/Cargo.toml",
        "[package]\nname = \"ejemplo\"\n\n[dependencies]\nhermano = { path = \"../hermano\" }\nserde = \"1\"\nfuera = { path = \"../../../otro\" }\ncompartido = { workspace = true }\n\n[target.'cfg(windows)'.dependencies]\nwinapi = \"0.3\"\n\n[dev-dependencies.rand]\nversion = \"0.8\"\n",
    );
    assert_eq!(
        p.sitios("cero-deps"),
        [
            "crates/ejemplo/Cargo.toml:6",
            "crates/ejemplo/Cargo.toml:7",
            "crates/ejemplo/Cargo.toml:11",
            "crates/ejemplo/Cargo.toml:13"
        ]
    );
}

#[test]
fn nucleo_sin_std_exige_no_std_y_solo_core_y_alloc() {
    let p = Prueba::nueva("nucleo");
    p.escribir("crates/nudo/Cargo.toml", "[package]\nname = \"nudo\"\n");
    p.escribir("crates/nudo/README.md", "# nudo\n");
    p.escribir("crates/nudo/tests/a.rs", "#[test]\nfn a() {}\n");
    p.escribir("crates/nudo/src/lib.rs", "//! Sin no_std.\nuse std::vec::Vec;\n");
    assert_eq!(p.sitios("nucleo-sin-std"), ["crates/nudo/src/lib.rs:1", "crates/nudo/src/lib.rs:2"]);

    // Bien: no_std, alloc y core. Un `std::` en un comentario no cuenta.
    p.escribir(
        "crates/nudo/src/lib.rs",
        "//! Núcleo (no uses std:: aquí).\n#![no_std]\nextern crate alloc;\nuse alloc::vec::Vec;\nuse core::fmt;\n",
    );
    assert_eq!(p.sitios("nucleo-sin-std"), Vec::<String>::new());

    p.escribir("crates/nudo/src/otro.rs", "extern crate std;\n");
    assert_eq!(p.sitios("nucleo-sin-std"), ["crates/nudo/src/otro.rs:1"]);
}

#[test]
fn unsafe_acotado_solo_en_ffi_y_justificado() {
    let p = Prueba::nueva("unsafe");
    p.escribir(
        LIB,
        "//! Un `unsafe` en un comentario no cuenta.\n#![allow(unsafe_code, reason = \"x\")]\npub fn f() -> &'static str {\n    unsafe { \"unsafe en una cadena\" }\n}\n",
    );
    assert_eq!(p.sitios("unsafe-acotado"), [format!("{LIB}:4"), format!("{LIB}:2")]);

    p.escribir(LIB, "//! Bien.\nmod ffi;\n");
    p.escribir(
        FFI,
        "#![allow(unsafe_code, reason = \"la puerta al sistema\")]\n\
         // SAFETY: el puntero viene de una referencia válida y vive todo el bloque.\n\
         // (Un comentario de varias líneas también vale.)\n\
         #[inline]\n\
         pub fn a(p: &u8) -> u8 { unsafe { *core::ptr::from_ref(p) } }\n\
         /// Lee.\n\
         ///\n\
         /// # Safety\n\
         ///\n\
         /// `p` tiene que ser válido.\n\
         pub unsafe fn b(p: *const u8) -> u8 { 0 }\n\
         pub fn c() {}\n\
         pub fn d() { unsafe {} }\n",
    );
    assert_eq!(p.sitios("unsafe-acotado"), [format!("{FFI}:13")]);
}

#[test]
fn los_tipos_de_puntero_a_funcion_no_necesitan_safety() {
    let p = Prueba::nueva("tipos");
    p.escribir(LIB, "//! Bien.\nmod ffi;\n");
    p.escribir(
        FFI,
        "pub type Crear = unsafe extern \"system\" fn(*const u8) -> i32;\n\
         pub struct Tabla {\n    pub crear: unsafe extern \"system\" fn(u32),\n    pub otra: unsafe fn(u8),\n}\n\
         pub unsafe fn sin_doc() {}\n",
    );
    // Los tipos pasan; la declaración de una `unsafe fn` sin `# Safety`, no.
    assert_eq!(p.sitios("unsafe-acotado"), [format!("{FFI}:6")]);

    // Fuera de ffi/, ni siquiera el tipo.
    p.escribir(LIB, "//! Mal.\nmod ffi;\npub type F = unsafe fn();\n");
    assert_eq!(p.sitios("unsafe-acotado"), [format!("{FFI}:6"), format!("{LIB}:3")]);
}

#[test]
fn ffi_aislada_solo_en_ffi() {
    let p = Prueba::nueva("ffi");
    p.escribir(
        LIB,
        "//! Las declaraciones del sistema, fuera de ffi/.\nextern crate alloc;\nextern \"C\" fn llamada() {}\n#[link(name = \"kernel32\")]\nfn x() { let _ = LoadLibraryW; }\n",
    );
    assert_eq!(p.sitios("ffi-aislada"), [format!("{LIB}:3"), format!("{LIB}:4"), format!("{LIB}:5")]);

    // Dentro de ffi/ es su sitio.
    p.escribir(LIB, "//! Bien.\nmod ffi;\n");
    p.escribir(FFI, "extern \"C\" fn llamada() {}\nfn x() { let _ = GetProcAddress; }\n");
    assert_eq!(p.sitios("ffi-aislada"), Vec::<String>::new());

    // Y la puerta no se exporta.
    p.escribir(LIB, "//! Mal.\npub mod ffi;\npub use ffi::llamada;\n");
    assert_eq!(p.sitios("ffi-aislada"), [format!("{LIB}:2"), format!("{LIB}:3")]);
}

/// Un crate con su `Cargo.toml` (y README y test, para que solo hable la regla que se prueba).
fn crate_(p: &Prueba, nombre: &str, cargo: &str) {
    p.escribir(&format!("crates/{nombre}/Cargo.toml"), &format!("[package]\nname = \"{nombre}\"\n{cargo}"));
    p.escribir(&format!("crates/{nombre}/README.md"), "# x\n");
    p.escribir(&format!("crates/{nombre}/src/lib.rs"), "#![no_std]\n");
    p.escribir(&format!("crates/{nombre}/tests/a.rs"), "#[test]\nfn a() {}\n");
}

#[test]
fn fronteras_solo_lo_declarado() {
    let p = Prueba::nueva("fronteras");
    crate_(&p, "telar-app", "\n[dependencies]\ntelar-gpu = { path = \"../telar-gpu\" }\n");
    assert_eq!(p.sitios("fronteras"), ["crates/telar-app/Cargo.toml:5"]);

    crate_(
        &p,
        "telar-app",
        "\n[package.metadata.telar]\npuede-usar = [\"telar-ventana\", \"telar-gpu\"]\n\n[dependencies]\ntelar-gpu = { path = \"../telar-gpu\" }\n",
    );
    assert_eq!(p.sitios("fronteras"), Vec::<String>::new());
}

#[test]
fn fronteras_ventana_y_gpu_aisladas_por_completo() {
    let p = Prueba::nueva("aisladas");
    // Aunque lo declare: la ventana no puede usar la GPU, ni al revés.
    crate_(
        &p,
        "telar-ventana",
        "\n[package.metadata.telar]\npuede-usar = [\"telar-gpu\"]\n\n[dependencies]\ntelar-gpu = { path = \"../telar-gpu\" }\n",
    );
    crate_(&p, "telar-gpu", "\n[dev-dependencies]\ntelar-ventana = { path = \"../telar-ventana\" }\n");
    assert_eq!(p.sitios("fronteras"), ["crates/telar-gpu/Cargo.toml:5", "crates/telar-ventana/Cargo.toml:8"]);
}

#[test]
fn fronteras_el_nucleo_solo_usa_el_nucleo() {
    let p = Prueba::nueva("nucleo-fronteras");
    crate_(
        &p,
        "nudo",
        "\n[package.metadata.telar]\npuede-usar = [\"hilo\", \"telar-gpu\"]\n\n[dependencies]\nhilo = { path = \"../hilo\" }\ntelar-gpu = { path = \"../telar-gpu\" }\n",
    );
    assert_eq!(p.sitios("fronteras"), ["crates/nudo/Cargo.toml:9"]);
}

#[test]
fn tamano_hasta_400_lineas() {
    let p = Prueba::nueva("tamano");
    p.escribir("crates/ejemplo/src/justo.rs", &"// 400\n".repeat(400));
    assert_eq!(p.sitios("tamano"), Vec::<String>::new());
    p.escribir("crates/ejemplo/src/largo.rs", &"// 401\n".repeat(401));
    assert_eq!(p.sitios("tamano"), ["crates/ejemplo/src/largo.rs"]);
}

#[test]
fn sin_restos_de_depuracion() {
    let p = Prueba::nueva("restos");
    p.escribir(
        LIB,
        "//! dbg! en un comentario no cuenta.\nfn a(x: u8) -> u8 { dbg!(x) }\nfn b() { todo!() }\nfn c() { unimplemented!() }\n#[test]\n#[ignore]\nfn d() {}\n#[test]\n#[ignore = \"tarda un minuto\"]\nfn e() { let _ = \"todo!()\"; }\n",
    );
    assert_eq!(
        p.sitios("sin-restos"),
        [format!("{LIB}:2"), format!("{LIB}:3"), format!("{LIB}:4"), format!("{LIB}:6")]
    );
}

#[test]
fn lf_rechaza_crlf_con_su_linea() {
    let p = Prueba::nueva("lf");
    p.escribir("crates/ejemplo/README.md", "# ejemplo\nbien\nmal\r\n");
    assert_eq!(p.sitios("lf"), ["crates/ejemplo/README.md:3"]);
}

#[test]
fn crate_completo_exige_readme_y_tests() {
    let p = Prueba::nueva("completo");
    p.borrar("crates/ejemplo/README.md");
    p.borrar("crates/ejemplo/tests/suma.rs");
    assert_eq!(p.problemas("crate-completo").len(), 2);

    // Los tests dentro de src/ también valen.
    p.escribir("crates/ejemplo/README.md", "# ejemplo\n");
    p.escribir(LIB, "//! Con tests dentro.\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn a() {}\n}\n");
    assert_eq!(p.sitios("crate-completo"), Vec::<String>::new());
}

#[test]
fn nodo_completo_exige_codigo_libreto_test_y_referencia() {
    let p = Prueba::nueva("nodos");
    p.escribir("crates/telar-nodos/src/caja/mod.rs", "//! Caja.\n");
    let carpeta = "crates/telar-nodos/src/caja/";
    assert_eq!(p.sitios("nodo-completo"), [carpeta, carpeta, carpeta]);

    p.escribir("crates/telar-nodos/src/caja/libreto.md", "# caja\n");
    p.escribir("crates/telar-nodos/tests/caja.rs", "#[test]\nfn a() {}\n");
    p.escribir("referencias/caja.md", "# caja · ABIERTO\n");
    assert_eq!(p.sitios("nodo-completo"), Vec::<String>::new());

    // Los archivos sueltos de src/ (lib.rs, parametros.rs) no son nodos.
    p.escribir("crates/telar-nodos/src/lib.rs", "//! Nodos.\n");
    assert_eq!(p.sitios("nodo-completo"), Vec::<String>::new());
}
