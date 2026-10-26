//! Las piezas del guardián por separado: el escáner de Rust, los commits,
//! el JSON y los hooks de Claude Code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "en los tests, un fallo es un panic con su mensaje"
)]

use std::path::Path;

use telar_guard::commit::validar;
use telar_guard::escaner::{blanquear, buscar_palabra, buscar_seguida};
use telar_guard::hooks::{comando_prohibido, ejecutar};
use telar_guard::json::{Valor, cadena_json, leer};

#[test]
fn blanquear_borra_comentarios_y_cadenas_y_conserva_las_lineas() {
    let src = "let a = \"unsafe \\\" x\"; // unsafe\n/* unsafe /* anidado */ unsafe */ let b = 1;\nunsafe {}\n";
    let codigo = blanquear(src);
    assert_eq!(codigo.lines().count(), 3);
    assert_eq!(buscar_palabra(&codigo, "unsafe"), [(3, 0)]);
    assert!(codigo.contains("let b = 1;"));
}

#[test]
fn blanquear_entiende_cadenas_en_bruto_caracteres_y_lifetimes() {
    let src = "let r = r#\"un \"unsafe\" dentro\"#;\nlet c = '\"'; let d = '\\''; fn f<'a>(x: &'a str) {}\nlet b = br\"dbg!()\";\n";
    let codigo = blanquear(src);
    assert_eq!(
        buscar_palabra(&codigo, "unsafe"),
        Vec::<(usize, usize)>::new()
    );
    assert_eq!(buscar_seguida(&codigo, "dbg", "!"), Vec::<usize>::new());
    assert!(
        codigo.contains("fn f<'a>(x: &'a str) {}"),
        "los lifetimes quedan: {codigo}"
    );
}

#[test]
fn buscar_palabra_respeta_los_bordes() {
    assert_eq!(
        buscar_palabra("unsafe_code mis_unsafe", "unsafe"),
        Vec::<(usize, usize)>::new()
    );
    assert_eq!(
        buscar_seguida("extern crate alloc;\nextern \"C\" {}", "extern", "\""),
        [2]
    );
}

#[test]
fn commits_convencionales() {
    for bien in [
        "feat(telar-guard): regla nucleo-sin-std",
        "fix: el escáner y las cadenas en bruto",
        "refactor(hilo-spirv)!: otra API",
        "# comentario de git\ndocs: el plan\n\ncuerpo",
        "Merge branch 'x'",
    ] {
        assert!(validar(bien).is_ok(), "{bien}");
    }
    for mal in [
        "Añade cosas",
        "feat:sin espacio",
        "feat(Mayus): algo",
        "feat(): algo",
        "feat: ab",
        "",
    ] {
        assert!(validar(mal).is_err(), "{mal}");
    }
}

#[test]
fn json_lee_eventos_y_escapa_respuestas() {
    let v =
        leer(r#"{"tool_input":{"command":"cargo add serde","n":[1,-2.5e3,true,null]},"t":"é\n"}"#)
            .unwrap();
    assert_eq!(
        v.en(&["tool_input", "command"]).and_then(Valor::texto),
        Some("cargo add serde")
    );
    assert_eq!(v.en(&["t"]).and_then(Valor::texto), Some("é\n"));
    assert!(leer("{\"a\": }").is_err());

    // El escape \u de JSON (se monta en dos trozos para que ningún editor lo convierta).
    let escapado = String::from("\"caf\\") + "u00e9\"";
    assert_eq!(leer(&escapado).unwrap(), Valor::Texto("café".to_owned()));

    let raro = "comillas \" barra \\ y\nsalto";
    assert_eq!(
        leer(&cadena_json(raro)).unwrap(),
        Valor::Texto(raro.to_owned())
    );
}

#[test]
fn pre_bash_deniega_lo_que_salta_las_reglas() {
    for prohibido in [
        "cargo add serde",
        "cd crates && cargo install ripgrep",
        "git commit --no-verify -m 'x'",
        "git commit -nm 'x'",
        "git -c core.hooksPath=/dev/null commit -m 'x'",
        "git push --force",
        "git push -f origin main",
    ] {
        assert!(comando_prohibido(prohibido).is_some(), "{prohibido}");
    }
    for permitido in [
        "cargo build --release",
        "git commit -m 'no es un flag'",
        "git config core.hooksPath .githooks",
        "git push origin main",
        "echo cargo add",
    ] {
        assert!(comando_prohibido(permitido).is_none(), "{permitido}");
    }
}

#[test]
fn el_hook_pre_bash_responde_en_json_de_claude_code() {
    let evento = leer(r#"{"tool_name":"Bash","tool_input":{"command":"cargo add rand"}}"#).unwrap();
    let respuesta = leer(&ejecutar("pre-bash", &evento, Path::new("."))).unwrap();
    assert_eq!(
        respuesta.en(&["hookSpecificOutput", "permissionDecision"]),
        Some(&Valor::Texto("deny".to_owned()))
    );

    let tranquilo = leer(r#"{"tool_input":{"command":"cargo test"}}"#).unwrap();
    assert_eq!(ejecutar("pre-bash", &tranquilo, Path::new(".")), "");
}
