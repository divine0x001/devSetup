//! Module `tools` du projet `devx`.
//!
//! Expose [`ToolsModule`], une implémentation de [`devx_core::Module`] qui
//! regroupe des commandes de confort pour développeurs :
//!
//! - `git <args...>` : wrappe le binaire `git` (ex. `git status`).
//! - `run <task>` : exécute une tâche déclarée dans `devx.toml`.
//! - `snip list|add|get` : snippets persistés dans `.devx/snippets.json`.
//! - `session save|load` : sessions persistées dans `.devx/sessions/<name>.json`.
//! - `replace <pattern> <replacement>` : remplacement regex projet-wide
//!   (respecte `.gitignore`, ignore les binaires).
//! - `outline <file>` : symboles de premier niveau d'un fichier source.
//! - `norm [chemin...]` : vérifie la Norme 42 via `norminette`.
//!
//! Aucune logique TUI n'est présente ici : uniquement de la logique pure et des
//! appels `std::process::Command`.

#![forbid(unsafe_code)]

mod git;
mod norm;
mod outline;
mod replace;
mod run;
mod session;
mod snip;
mod util;

use devx_core::{CommandSpec, Context, DevxError, Module, Output};

/// Point d'entrée du module de commandes de confort.
///
/// Implémente [`Module`] avec `name() == "tools"`.
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolsModule;

impl ToolsModule {
    /// Crée une instance du module.
    pub fn new() -> Self {
        Self
    }
}

impl Module for ToolsModule {
    fn name(&self) -> &'static str {
        "tools"
    }

    fn commands(&self) -> Vec<CommandSpec> {
        vec![
            CommandSpec::new(
                "git",
                "Exécute une commande git arbitraire dans la racine du workspace",
                "git <args...>",
            )
            .with_aliases(["g"]),
            CommandSpec::new(
                "run",
                "Exécute une tâche déclarée dans la section [tasks] de devx.toml",
                "run <task>",
            ),
            CommandSpec::new(
                "snip",
                "Gère les snippets (list, add, get)",
                "snip <list|add <name> <body>|get <name>>",
            )
            .with_aliases(["snippet"]),
            CommandSpec::new(
                "session",
                "Sauvegarde ou restaure une session (save, load)",
                "session <save <name> [file...]|load <name>>",
            )
            .with_aliases(["sess"]),
            CommandSpec::new(
                "replace",
                "Remplacement regex projet-wide (respecte .gitignore, ignore les binaires)",
                "replace <pattern> <replacement>",
            )
            .with_aliases(["sub"]),
            CommandSpec::new(
                "outline",
                "Liste les symboles de premier niveau d'un fichier source",
                "outline <file>",
            )
            .with_aliases(["symbols"]),
            CommandSpec::new(
                "norm",
                "Vérifie la Norme 42 avec norminette (fichier, dossier ou projet)",
                "norm [chemin...]",
            )
            .with_aliases(["norminette"]),
        ]
    }

    fn execute(&self, ctx: &Context, command: &str, args: &[String]) -> Result<Output, DevxError> {
        match command {
            "git" | "g" => git::execute(ctx, args),
            "run" => run::execute(ctx, args),
            "snip" | "snippet" => snip::execute(ctx, args),
            "session" | "sess" => session::execute(ctx, args),
            "replace" | "sub" => replace::execute(ctx, args),
            "outline" | "symbols" => outline::execute(ctx, args),
            "norm" | "norminette" => norm::execute(ctx, args),
            other => Err(DevxError::unknown_command(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(root: &std::path::Path) -> Context {
        Context::with_root(root)
    }

    #[test]
    fn name_is_tools() {
        assert_eq!(ToolsModule::new().name(), "tools");
    }

    #[test]
    fn declares_expected_commands() {
        let names: Vec<_> = ToolsModule::new()
            .commands()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(
            names,
            vec!["git", "run", "snip", "session", "replace", "outline", "norm"]
        );
    }

    #[test]
    fn unknown_command_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let err = ToolsModule::new()
            .execute(&ctx(dir.path()), "nope", &[])
            .unwrap_err();
        assert_eq!(err, DevxError::UnknownCommand("nope".to_string()));
    }

    #[test]
    fn aliases_route_to_same_handler() {
        let dir = tempfile::tempdir().unwrap();
        let module = ToolsModule::new();
        let ctx = ctx(dir.path());
        let args = vec!["add".to_string(), "x".to_string(), "body".to_string()];
        module.execute(&ctx, "snippet", &args).unwrap();
        let out = module
            .execute(&ctx, "snip", &["get".to_string(), "x".to_string()])
            .unwrap();
        assert_eq!(out.lines, vec!["body"]);
    }
}
