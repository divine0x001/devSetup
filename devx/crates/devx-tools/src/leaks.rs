//! Commande `leaks` : exécute un binaire sous un détecteur de fuites mémoire.
//!
//! Essaie `valgrind` d'abord (Linux), puis `leaks` (macOS, y compris Apple
//! Silicon où valgrind ne fonctionne pas). Le binaire est résolu sous la racine
//! du workspace et exécuté avec les arguments éventuels. La sortie du détecteur
//! est renvoyée telle quelle.

use std::io;
use std::path::Path;
use std::process::{Command, Output as ProcessOutput};

use devx_core::{Context, DevxError, Output};

use crate::util::bytes_to_lines;

/// Exécute `args[0]` (le binaire) sous un détecteur de fuites avec `args[1..]`.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let (bin, prog_args) = match args.split_first() {
        Some((bin, rest)) => (bin, rest),
        None => return Err(DevxError::invalid_args("usage: leaks <binaire> [args...]")),
    };

    let path = {
        let p = Path::new(bin);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            ctx.workspace.root.join(p)
        }
    };
    if !path.is_file() {
        return Err(DevxError::message(format!("binaire introuvable: {bin}")));
    }
    let bin_abs = path.display().to_string();

    // valgrind d'abord ; repli sur `leaks` si le binaire est absent.
    let result = run_valgrind(ctx, &bin_abs, prog_args).or_else(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            run_leaks(ctx, &bin_abs, prog_args)
        } else {
            Err(e)
        }
    });

    match result {
        Ok(out) => {
            let mut lines = bytes_to_lines(&out.stdout);
            lines.extend(bytes_to_lines(&out.stderr));
            if lines.is_empty() {
                lines.push("leaks: aucune sortie du détecteur".to_string());
            }
            Ok(Output::new(lines))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(DevxError::message(
            "ni `valgrind` ni `leaks` disponibles — installe valgrind (Linux) \
             ou les Command Line Tools (macOS)",
        )),
        Err(e) => Err(DevxError::io(format!("échec du détecteur de fuites: {e}"))),
    }
}

fn run_valgrind(ctx: &Context, bin: &str, args: &[String]) -> io::Result<ProcessOutput> {
    Command::new("valgrind")
        .args([
            "--leak-check=full",
            "--show-leak-kinds=all",
            "--error-exitcode=0",
        ])
        .arg(bin)
        .args(args)
        .current_dir(&ctx.workspace.root)
        .output()
}

fn run_leaks(ctx: &Context, bin: &str, args: &[String]) -> io::Result<ProcessOutput> {
    Command::new("leaks")
        .arg("--atExit")
        .arg("--")
        .arg(bin)
        .args(args)
        .current_dir(&ctx.workspace.root)
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};

    fn ctx_for(dir: &std::path::Path) -> Context {
        Context::new(Workspace::new(dir), Config::default())
    }

    #[test]
    fn empty_args_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx_for(dir.path()), &[]),
            Err(DevxError::InvalidArgs(_))
        ));
    }

    #[test]
    fn missing_binary_is_message_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = execute(&ctx_for(dir.path()), &["nope".to_string()]).unwrap_err();
        assert!(
            matches!(err, DevxError::Message(ref m) if m.contains("introuvable")),
            "erreur inattendue: {err:?}"
        );
    }
}
