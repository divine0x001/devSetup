//! Commandes `session` : sauvegarde/restauration de l'état courant.

use std::path::{Path, PathBuf};

use devx_core::{Context, DevxError, Output};
use serde::{Deserialize, Serialize};

use crate::util::validate_name;

/// État persisté d'une session : nom, racine du workspace et fichiers ouverts.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Session {
    /// Nom logique de la session.
    pub name: String,
    /// Racine du workspace au moment de la sauvegarde.
    pub root: String,
    /// Fichiers considérés comme ouverts.
    #[serde(default)]
    pub files: Vec<String>,
}

/// Dossier de stockage des sessions.
fn sessions_dir(root: &Path) -> PathBuf {
    root.join(".devx").join("sessions")
}

/// Chemin du fichier d'une session (validation du nom incluse).
fn session_path(root: &Path, name: &str) -> Result<PathBuf, DevxError> {
    validate_name("session", name)?;
    Ok(sessions_dir(root).join(format!("{name}.json")))
}

/// Exécute `session save <name> [file...]` ou `session load <name>`.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let (sub, rest) = args
        .split_first()
        .ok_or_else(|| DevxError::invalid_args("usage: session <save|load> <name>"))?;

    match sub.as_str() {
        "save" => {
            let (name, files) = rest
                .split_first()
                .ok_or_else(|| DevxError::invalid_args("usage: session save <name> [file...]"))?;

            let session = Session {
                name: name.clone(),
                root: ctx.workspace.root.display().to_string(),
                files: files.to_vec(),
            };

            let path = session_path(&ctx.workspace.root, name)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DevxError::io(format!("création de {}: {e}", parent.display())))?;
            }
            let raw = serde_json::to_string_pretty(&session)
                .map_err(|e| DevxError::other(format!("sérialisation de la session: {e}")))?;
            std::fs::write(&path, raw)
                .map_err(|e| DevxError::io(format!("écriture de {}: {e}", path.display())))?;

            Ok(Output::new(vec![format!("session `{name}` enregistrée")]))
        }
        "load" => {
            let name = match rest {
                [name] => name,
                _ => return Err(DevxError::invalid_args("usage: session load <name>")),
            };

            let path = session_path(&ctx.workspace.root, name)?;
            let raw = std::fs::read_to_string(&path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DevxError::message(format!("session inconnue: `{name}`"))
                } else {
                    DevxError::io(format!("lecture de {}: {e}", path.display()))
                }
            })?;
            let session: Session = serde_json::from_str(&raw)
                .map_err(|e| DevxError::message(format!("session `{name}` corrompue: {e}")))?;

            Ok(Output::new(render(&session)))
        }
        other => Err(DevxError::unknown_command(format!("session {other}"))),
    }
}

/// Rend une session sous forme de lignes lisibles.
fn render(session: &Session) -> Vec<String> {
    let mut lines = vec![
        format!("name: {}", session.name),
        format!("root: {}", session.root),
    ];
    for file in &session.files {
        lines.push(format!("file: {file}"));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::Workspace;

    fn ctx(dir: &Path) -> Context {
        Context::new(Workspace::new(dir), devx_core::Config::default())
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx(dir.path());

        execute(
            &ctx,
            &[
                "save".to_string(),
                "work".to_string(),
                "a.rs".to_string(),
                "b.rs".to_string(),
            ],
        )
        .unwrap();

        let out = execute(&ctx, &["load".to_string(), "work".to_string()]).unwrap();
        assert!(out.lines.iter().any(|l| l == "name: work"));
        assert!(out.lines.iter().any(|l| l == "file: a.rs"));
        assert!(out.lines.iter().any(|l| l == "file: b.rs"));
    }

    #[test]
    fn load_unknown_session_is_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["load".to_string(), "nope".to_string()]),
            Err(DevxError::Message(_))
        ));
    }

    #[test]
    fn rejects_path_traversal_name() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(
                &ctx(dir.path()),
                &["save".to_string(), "../evil".to_string()]
            ),
            Err(DevxError::InvalidArgs(_))
        ));
    }

    #[test]
    fn save_without_name_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["save".to_string()]),
            Err(DevxError::InvalidArgs(_))
        ));
    }
}
