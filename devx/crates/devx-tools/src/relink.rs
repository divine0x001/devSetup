//! Commande `relink` : vérifie qu'un Makefile ne reconstruit pas inutilement.
//!
//! Un critère de soutenance récurrent à 42 : lancer `make` deux fois ne doit
//! rien reconstruire la seconde fois. On construit le projet (`make`), puis on
//! interroge `make -q` (mode « question ») : code de sortie 0 = tout est à
//! jour, non nul = `make` reconstruirait (relink). `make -q` est indépendant de
//! la locale, contrairement au message « Nothing to be done ».

use std::path::{Path, PathBuf};
use std::process::Command;

use devx_core::{Context, DevxError, Output};

use crate::util::bytes_to_lines;

/// Vérifie l'absence de relink dans `args[0]` (défaut : racine du workspace).
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let dir: PathBuf = match args.first() {
        Some(arg) => {
            let p = Path::new(arg);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                ctx.workspace.root.join(p)
            }
        }
        None => ctx.workspace.root.clone(),
    };

    if !dir.join("Makefile").is_file() && !dir.join("makefile").is_file() {
        return Err(DevxError::message(format!(
            "aucun Makefile dans {}",
            dir.display()
        )));
    }

    let build = Command::new("make")
        .current_dir(&dir)
        .output()
        .map_err(|e| DevxError::message(format!("impossible de lancer `make` ({e})")))?;

    let mut lines = bytes_to_lines(&build.stdout);
    lines.extend(bytes_to_lines(&build.stderr));
    if !build.status.success() {
        lines.push("relink: build initial en échec — corrige la compilation d'abord".to_string());
        return Ok(Output::new(lines));
    }

    let question = Command::new("make")
        .arg("-q")
        .current_dir(&dir)
        .output()
        .map_err(|e| DevxError::message(format!("impossible de lancer `make -q` ({e})")))?;

    if question.status.success() {
        lines.push("relink: OK — le 2e `make` ne reconstruit rien".to_string());
    } else {
        lines.push("⚠ relink détecté — `make` reconstruit alors que tout est à jour".to_string());
    }
    Ok(Output::new(lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};

    fn tools_available() -> bool {
        let ok = |bin: &str| {
            Command::new(bin)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        ok("make") && ok("cc")
    }

    fn ctx_for(dir: &std::path::Path) -> Context {
        Context::new(Workspace::new(dir), Config::default())
    }

    fn write(dir: &std::path::Path, name: &str, body: &str) {
        std::fs::write(dir.join(name), body).unwrap();
    }

    #[test]
    fn missing_makefile_is_message_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx_for(dir.path()), &[]),
            Err(DevxError::Message(_))
        ));
    }

    #[test]
    fn well_formed_makefile_has_no_relink() {
        if !tools_available() {
            eprintln!("make/cc indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "main.c",
            "int main(void)\n{\n\treturn (0);\n}\n",
        );
        write(
            dir.path(),
            "Makefile",
            "NAME = prog\nall: $(NAME)\n$(NAME): main.c\n\tcc -o $(NAME) main.c\n",
        );
        let out = execute(&ctx_for(dir.path()), &[]).unwrap();
        assert!(
            out.lines.iter().any(|l| l.contains("relink: OK")),
            "sortie: {:?}",
            out.lines
        );
    }

    #[test]
    fn always_rebuilding_rule_is_flagged() {
        if !tools_available() {
            eprintln!("make/cc indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "main.c",
            "int main(void)\n{\n\treturn (0);\n}\n",
        );
        // `all` sans prérequis : make le relance toujours.
        write(dir.path(), "Makefile", "all:\n\tcc -o prog main.c\n");
        let out = execute(&ctx_for(dir.path()), &[]).unwrap();
        assert!(
            out.lines.iter().any(|l| l.contains("relink détecté")),
            "sortie: {:?}",
            out.lines
        );
    }
}
