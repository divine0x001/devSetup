//! Commande `cc` : compile des sources C avec les flags obligatoires de 42.
//!
//! Lance `cc -Wall -Wextra -Werror -fsyntax-only <fichiers>` dans la racine du
//! workspace : vérifie qu'un fichier compile proprement (zéro warning) sans
//! produire d'objet. Les diagnostics du compilateur sortent au format
//! `fichier:ligne:col: ...`, que la TUI sait reparser pour sauter à l'erreur.
//!
//! Un échec de compilation n'est pas une erreur d'outil : les diagnostics sont
//! renvoyés dans l'[`Output`] pour affichage. Seul l'échec de lancement de `cc`
//! produit une [`DevxError`].

use std::process::Command;

use devx_core::{Context, DevxError, Output};

use crate::util::bytes_to_lines;

/// Compile les fichiers fournis avec les flags 42 (mode `-fsyntax-only`).
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    if args.is_empty() {
        return Err(DevxError::invalid_args(
            "usage: cc <fichier.c...> (compile avec -Wall -Wextra -Werror)",
        ));
    }

    let output = Command::new("cc")
        .args(["-Wall", "-Wextra", "-Werror", "-fsyntax-only"])
        .args(args)
        .current_dir(&ctx.workspace.root)
        .output()
        .map_err(|e| {
            DevxError::message(format!(
                "impossible de lancer `cc` ({e}) — installe un compilateur C"
            ))
        })?;

    if output.status.success() {
        let mut lines = bytes_to_lines(&output.stderr);
        if lines.is_empty() {
            lines.push(format!(
                "cc: OK — {} fichier(s), aucun warning (-Wall -Wextra -Werror)",
                args.len()
            ));
        }
        return Ok(Output::new(lines));
    }

    // Échec : remonter les diagnostics (stderr, sinon stdout).
    let mut lines = bytes_to_lines(&output.stderr);
    if lines.is_empty() {
        lines = bytes_to_lines(&output.stdout);
    }
    if lines.is_empty() {
        lines.push("cc: échec de compilation (aucun diagnostic)".to_string());
    }
    Ok(Output::new(lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};

    fn cc_available() -> bool {
        Command::new("cc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

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
    fn clean_file_compiles_without_warning() {
        if !cc_available() {
            eprintln!("cc indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("ok.c"),
            "int main(void)\n{\n\treturn (0);\n}\n",
        )
        .unwrap();
        let out = execute(&ctx_for(dir.path()), &["ok.c".to_string()]).unwrap();
        assert!(
            out.lines.iter().any(|l| l.starts_with("cc: OK")),
            "sortie inattendue: {:?}",
            out.lines
        );
    }

    #[test]
    fn warning_is_reported_as_locatable_diagnostic() {
        if !cc_available() {
            eprintln!("cc indisponible: test ignoré");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        // Variable inutilisée : -Wall -Werror transforme le warning en erreur.
        std::fs::write(
            dir.path().join("bad.c"),
            "int main(void)\n{\n\tint x;\n\treturn (0);\n}\n",
        )
        .unwrap();
        let out = execute(&ctx_for(dir.path()), &["bad.c".to_string()]).unwrap();
        let text = out.lines.join("\n");
        assert!(text.contains("bad.c:"), "diagnostic attendu: {text}");
    }
}
