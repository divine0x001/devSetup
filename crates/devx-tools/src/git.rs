//! Commande `git` : wrapper léger autour du binaire `git`.

use std::process::Command;

use devx_core::{Context, DevxError, Output};

use crate::util::bytes_to_lines;

/// Exécute `git` avec les arguments fournis dans la racine du workspace.
///
/// Les arguments sont transmis individuellement au processus (aucun shell
/// n'est invoqué), ce qui neutralise toute injection. En cas de code de sortie
/// non nul, l'erreur standard est remontée dans un [`DevxError::Message`].
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    if args.is_empty() {
        return Err(DevxError::invalid_args(
            "usage: git <args...> (ex. git status, git log --oneline -n 20)",
        ));
    }

    let output = Command::new("git")
        .args(args)
        .current_dir(&ctx.workspace.root)
        .output()
        .map_err(|e| DevxError::io(format!("impossible de lancer `git`: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        let message = if detail.is_empty() {
            format!("`git {}` a échoué", args.join(" "))
        } else {
            format!("`git {}` a échoué: {detail}", args.join(" "))
        };
        return Err(DevxError::message(message));
    }

    Ok(Output::new(bytes_to_lines(&output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::Workspace;

    fn ctx_for(dir: &std::path::Path) -> Context {
        Context::new(Workspace::new(dir), devx_core::Config::default())
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run_git(dir: &std::path::Path, args: &[&str]) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "devx")
            .env("GIT_AUTHOR_EMAIL", "devx@example.com")
            .env("GIT_COMMITTER_NAME", "devx")
            .env("GIT_COMMITTER_EMAIL", "devx@example.com")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn init_repo(dir: &std::path::Path) {
        assert!(run_git(dir, &["init", "-q"]), "git init a échoué");
        std::fs::write(dir.join("hello.txt"), "hello\n").unwrap();
        assert!(run_git(dir, &["add", "hello.txt"]));
        assert!(run_git(
            dir,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "init"]
        ));
    }

    #[test]
    fn git_status_on_temp_repo() {
        if !git_available() {
            eprintln!("git indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        let out = execute(&ctx_for(dir.path()), &["status".to_string()]).unwrap();
        let text = out.lines.join("\n");
        assert!(text.contains("branch"), "sortie inattendue: {text}");
    }

    #[test]
    fn git_log_oneline_limits_output() {
        if !git_available() {
            eprintln!("git indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        let args = vec![
            "log".to_string(),
            "--oneline".to_string(),
            "-n".to_string(),
            "1".to_string(),
        ];
        let out = execute(&ctx_for(dir.path()), &args).unwrap();
        assert_eq!(out.lines.len(), 1);
        assert!(out.lines[0].contains("init"));
    }

    #[test]
    fn empty_args_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx_for(dir.path()), &[]),
            Err(DevxError::InvalidArgs(_))
        ));
    }
}
