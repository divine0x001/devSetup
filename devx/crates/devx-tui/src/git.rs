//! Rapport git de la vue « git » : état et collecte via le binaire `git`,
//! indépendants de l'état applicatif.

use std::path::Path;
use std::process::Command;

/// Informations git affichées dans la vue « git ».
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitState {
    /// Branche courante (ou « HEAD » si détaché).
    pub branch: String,
    /// Lignes de `git status --short --branch`.
    pub status: Vec<String>,
    /// Lignes de `git log --oneline`.
    pub log: Vec<String>,
    /// Message d'erreur éventuel (hors dépôt git, git absent…).
    pub error: Option<String>,
}

/// Lance `git <args>` dans `root` et renvoie la sortie standard (texte).
fn git_capture(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git introuvable : {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        return Err(if detail.is_empty() {
            format!("`git {}` a échoué", args.join(" "))
        } else {
            detail.to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Construit l'état git (branche, statut, derniers commits) pour `root`.
pub(crate) fn load_git(root: &Path) -> GitState {
    let mut state = GitState {
        branch: "HEAD".to_string(),
        ..GitState::default()
    };

    let status = match git_capture(root, &["status", "--short", "--branch"]) {
        Ok(text) => text,
        Err(err) => {
            state.error = Some(err);
            return state;
        }
    };

    let mut lines = status.lines();
    if let Some(first) = lines.next() {
        let head = first.trim_start_matches("## ").trim();
        let name = head.split("...").next().unwrap_or(head);
        let name = name.split_whitespace().next().unwrap_or("HEAD");
        state.branch = if name.is_empty() {
            "HEAD".to_string()
        } else {
            name.to_string()
        };
    }
    state.status = lines.map(|l| l.to_string()).collect();

    state.log = git_capture(root, &["log", "--oneline", "-n", "20"])
        .map(|text| text.lines().map(str::to_string).collect())
        .unwrap_or_default();

    state
}
