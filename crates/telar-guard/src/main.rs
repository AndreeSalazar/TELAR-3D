//! `telar-guard`, la línea de comandos.
//!
//! ```text
//! telar-guard check                 revisa las reglas (sale con 1 si alguna falla)
//! telar-guard rules                 lista las reglas con su porqué y su arreglo
//! telar-guard verify                guardián + rustfmt + clippy + tests (y activa los hooks de git)
//! telar-guard commit-msg <archivo>  valida un mensaje de commit (hook commit-msg)
//! telar-guard hook <nombre>         hook de Claude Code: pre-bash, post-edit o stop (evento por stdin)
//! ```

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use telar_guard::{commit, hooks, informe, json, reglas, repo, verificar};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let comando = args.first().map_or("check", String::as_str);

    let desde = std::env::var_os("CLAUDE_PROJECT_DIR").map(PathBuf::from).or_else(|| std::env::current_dir().ok());
    let Some(raiz) = desde.and_then(|d| repo::buscar_raiz(&d)) else {
        eprintln!("telar-guard: no encuentro la raíz del repo (un Cargo.toml con [workspace] y una carpeta crates/)");
        return ExitCode::from(2);
    };

    match comando {
        "check" => match repo::Repo::abrir(&raiz) {
            Ok(repo) => {
                let resultados = reglas::revisar(&repo);
                print!("{}", informe::texto(&resultados, repo.archivos.len()));
                salida(resultados.iter().all(|r| r.problemas.is_empty()))
            }
            Err(e) => {
                eprintln!("telar-guard: {e}");
                ExitCode::from(2)
            }
        },
        "rules" => {
            for r in &reglas::REGLAS {
                println!(
                    "{:<16} {}\n{:<16} por qué: {}\n{:<16} arreglo: {}\n",
                    r.id, r.titulo, "", r.motivo, "", r.arreglo
                );
            }
            ExitCode::SUCCESS
        }
        "verify" => salida(verificar::verify(&raiz)),
        "commit-msg" => {
            let Some(texto) = args.get(1).and_then(|f| std::fs::read_to_string(f).ok()) else {
                eprintln!("telar: uso: telar-guard commit-msg <archivo>");
                return ExitCode::from(2);
            };
            match commit::validar(&texto) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("telar: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "hook" => {
            let mut entrada = String::new();
            // Sin evento legible no se bloquea nada: un hook roto no debe paralizar el trabajo.
            if std::io::stdin().read_to_string(&mut entrada).is_err() {
                return ExitCode::SUCCESS;
            }
            let evento = json::leer(&entrada).unwrap_or(json::Valor::Objeto(Vec::new()));
            let respuesta = hooks::ejecutar(args.get(1).map_or("", String::as_str), &evento, &raiz);
            if !respuesta.is_empty() {
                println!("{respuesta}");
            }
            ExitCode::SUCCESS
        }
        otro => {
            eprintln!("telar-guard: comando desconocido «{otro}» (usa check, rules, verify, commit-msg o hook)");
            ExitCode::from(2)
        }
    }
}

fn salida(ok: bool) -> ExitCode {
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
