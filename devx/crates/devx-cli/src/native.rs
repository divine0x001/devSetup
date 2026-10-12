//! Module natif `core` : commandes de base de `devx`.

use devx_core::{CommandSpec, Context, DevxError, Module, Output};

/// Module natif fournissant `help`, `version` et `config`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CoreModule;

impl CoreModule {
    /// Crée le module natif.
    pub fn new() -> Self {
        Self
    }
}

impl Module for CoreModule {
    fn name(&self) -> &'static str {
        "core"
    }

    fn commands(&self) -> Vec<CommandSpec> {
        vec![
            CommandSpec::new("help", "affiche les commandes natives", "help"),
            CommandSpec::new("version", "affiche la version de devx", "version"),
            CommandSpec::new("config", "affiche la configuration chargée", "config"),
        ]
    }

    fn execute(&self, ctx: &Context, command: &str, _args: &[String]) -> Result<Output, DevxError> {
        match command {
            "help" => Ok(Output::new(self.help_lines())),
            "version" => Ok(Output::new(vec![format!(
                "devx {}",
                env!("CARGO_PKG_VERSION")
            )])),
            "config" => Ok(Output::new(vec![
                format!("root       = {}", ctx.workspace.root.display()),
                format!("theme      = {}", ctx.config.theme),
                format!("editor_cmd = {}", ctx.config.editor_cmd),
                format!("tasks      = {}", ctx.config.tasks.len()),
            ])),
            other => Err(DevxError::unknown_command(other)),
        }
    }
}

impl CoreModule {
    fn help_lines(&self) -> Vec<String> {
        let mut lines = vec!["devx — commandes natives :".to_string()];
        for spec in self.commands() {
            lines.push(format!("  {:<10} {}", spec.usage, spec.description));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};

    fn ctx() -> Context {
        Context::new(Workspace::new("."), Config::default())
    }

    #[test]
    fn exposes_native_commands() {
        let names: Vec<String> = CoreModule::new()
            .commands()
            .into_iter()
            .map(|spec| spec.name)
            .collect();
        assert_eq!(names, ["help", "version", "config"]);
    }

    #[test]
    fn version_is_reported() {
        let out = CoreModule::new().execute(&ctx(), "version", &[]).unwrap();
        assert!(out.lines[0].starts_with("devx "));
    }

    #[test]
    fn unknown_command_errors() {
        let err = CoreModule::new().execute(&ctx(), "nope", &[]).unwrap_err();
        assert!(matches!(err, DevxError::UnknownCommand(_)));
    }
}
