//! Commande `run` : exécute une tâche déclarée dans `devx.toml`.

use std::process::Command;

use devx_core::{Context, DevxError, Output};

use crate::util::bytes_to_lines;

/// Exécute la tâche `ctx.config.tasks[task]` via le shell.
///
/// Le nom de la tâche est uniquement utilisé comme clé de configuration : la
/// chaîne de commande provient exclusivement du `devx.toml` de confiance de
/// l'utilisateur. Une tâche inconnue produit un [`DevxError::Message`] explicite.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let task = match args {
        [task] => task,
        _ => return Err(DevxError::invalid_args("usage: run <task>")),
    };

    let command = ctx
        .config
        .tasks
        .get(task)
        .ok_or_else(|| DevxError::message(format!("tâche inconnue: `{task}`")))?;

    if command.trim().is_empty() {
        return Err(DevxError::message(format!("la tâche `{task}` est vide")));
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&ctx.workspace.root)
        .output()
        .map_err(|e| DevxError::io(format!("impossible de lancer la tâche `{task}`: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevxError::message(format!(
            "la tâche `{task}` a échoué: {}",
            stderr.trim()
        )));
    }

    Ok(Output::new(bytes_to_lines(&output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};
    use std::collections::BTreeMap;

    fn ctx_with(dir: &std::path::Path, tasks: &[(&str, &str)]) -> Context {
        let config = Config {
            tasks: tasks
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect::<BTreeMap<_, _>>(),
            ..Config::default()
        };
        Context::new(Workspace::new(dir), config)
    }

    #[test]
    fn runs_configured_task() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx_with(dir.path(), &[("hello", "echo bonjour")]);
        let out = execute(&ctx, &["hello".to_string()]).unwrap();
        assert_eq!(out.lines, vec!["bonjour"]);
    }

    #[test]
    fn unknown_task_is_message_error() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx_with(dir.path(), &[]);
        assert!(matches!(
            execute(&ctx, &["missing".to_string()]),
            Err(DevxError::Message(_))
        ));
    }

    #[test]
    fn missing_argument_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx_with(dir.path(), &[]);
        assert!(matches!(execute(&ctx, &[]), Err(DevxError::InvalidArgs(_))));
    }
}
