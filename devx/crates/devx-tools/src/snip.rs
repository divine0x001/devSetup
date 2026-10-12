//! Commandes `snip` : gestion de snippets persistés en JSON.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use devx_core::{Context, DevxError, Output};

use crate::util::validate_name;

type Snippets = BTreeMap<String, String>;

/// Chemin du fichier de stockage des snippets.
fn store_path(root: &Path) -> PathBuf {
    root.join(".devx").join("snippets.json")
}

/// Charge la table des snippets (table vide si le fichier n'existe pas).
fn load(path: &Path) -> Result<Snippets, DevxError> {
    match std::fs::read_to_string(path) {
        Ok(raw) => {
            if raw.trim().is_empty() {
                Ok(Snippets::new())
            } else {
                serde_json::from_str(&raw).map_err(|e| {
                    DevxError::message(format!("snippets corrompus ({}): {e}", path.display()))
                })
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Snippets::new()),
        Err(e) => Err(DevxError::io(format!("lecture de {}: {e}", path.display()))),
    }
}

/// Écrit la table des snippets sur disque (crée `.devx/` au besoin).
fn save(path: &Path, snippets: &Snippets) -> Result<(), DevxError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DevxError::io(format!("création de {}: {e}", parent.display())))?;
    }
    let raw = serde_json::to_string_pretty(snippets)
        .map_err(|e| DevxError::other(format!("sérialisation des snippets: {e}")))?;
    std::fs::write(path, raw)
        .map_err(|e| DevxError::io(format!("écriture de {}: {e}", path.display())))
}

/// Exécute `snip list|add|get`.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let (sub, rest) = args
        .split_first()
        .ok_or_else(|| DevxError::invalid_args("usage: snip <list|add|get>"))?;

    let path = store_path(&ctx.workspace.root);

    match sub.as_str() {
        "list" => {
            if !rest.is_empty() {
                return Err(DevxError::invalid_args("usage: snip list"));
            }
            let snippets = load(&path)?;
            Ok(Output::new(snippets.keys().cloned().collect()))
        }
        "add" => match rest {
            [name, body] => {
                validate_name("snippet", name)?;
                let mut snippets = load(&path)?;
                snippets.insert(name.clone(), body.clone());
                save(&path, &snippets)?;
                Ok(Output::new(vec![format!("snippet `{name}` enregistré")]))
            }
            _ => Err(DevxError::invalid_args("usage: snip add <name> <body>")),
        },
        "get" => match rest {
            [name] => {
                validate_name("snippet", name)?;
                let snippets = load(&path)?;
                let body = snippets
                    .get(name)
                    .ok_or_else(|| DevxError::message(format!("snippet inconnu: `{name}`")))?;
                Ok(Output::new(body.lines().map(str::to_owned).collect()))
            }
            _ => Err(DevxError::invalid_args("usage: snip get <name>")),
        },
        other => Err(DevxError::unknown_command(format!("snip {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::Workspace;

    fn ctx(dir: &Path) -> Context {
        Context::new(Workspace::new(dir), devx_core::Config::default())
    }

    #[test]
    fn add_then_get_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx(dir.path());

        execute(
            &ctx,
            &[
                "add".to_string(),
                "greet".to_string(),
                "hello\nworld".to_string(),
            ],
        )
        .unwrap();

        let out = execute(&ctx, &["get".to_string(), "greet".to_string()]).unwrap();
        assert_eq!(out.lines, vec!["hello", "world"]);

        let listed = execute(&ctx, &["list".to_string()]).unwrap();
        assert_eq!(listed.lines, vec!["greet"]);
    }

    #[test]
    fn persists_across_contexts() {
        let dir = tempfile::tempdir().unwrap();
        execute(
            &ctx(dir.path()),
            &["add".to_string(), "k".to_string(), "v".to_string()],
        )
        .unwrap();
        let out = execute(&ctx(dir.path()), &["get".to_string(), "k".to_string()]).unwrap();
        assert_eq!(out.lines, vec!["v"]);
    }

    #[test]
    fn unknown_snippet_is_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["get".to_string(), "nope".to_string()]),
            Err(DevxError::Message(_))
        ));
    }

    #[test]
    fn rejects_path_traversal_name() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(
                &ctx(dir.path()),
                &["add".to_string(), "../evil".to_string(), "x".to_string()]
            ),
            Err(DevxError::InvalidArgs(_))
        ));
    }

    #[test]
    fn list_empty_when_no_store() {
        let dir = tempfile::tempdir().unwrap();
        let out = execute(&ctx(dir.path()), &["list".to_string()]).unwrap();
        assert!(out.lines.is_empty());
    }
}
