//! # devx-cli
//!
//! Cœur du binaire `devx` : parsing des arguments, registre de modules et
//! dispatch des commandes. Séparé du binaire pour être testable.

mod native;

pub use native::CoreModule;

use clap::Parser;
use devx_core::{CommandSpec, Context, DevxError, Module, Output};

/// Arguments de la ligne de commande.
#[derive(Debug, Parser)]
#[command(
    name = "devx",
    version,
    about = "devx — VS Code dans le terminal",
    long_about = None
)]
pub struct Cli {
    /// Commande à exécuter en mode one-shot. Absente => mode TUI.
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,

    /// Arguments transmis à la commande.
    #[arg(
        value_name = "ARGS",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub args: Vec<String>,
}

/// Invocation résolue à partir des arguments.
#[derive(Debug, PartialEq, Eq)]
pub enum Invocation {
    /// Aucun argument : lancement de la TUI.
    Tui,
    /// Exécution one-shot d'une commande.
    Run {
        /// Nom de la commande.
        command: String,
        /// Arguments de la commande.
        args: Vec<String>,
    },
}

/// Résout les arguments parsés en [`Invocation`].
pub fn classify(cli: Cli) -> Invocation {
    match cli.command {
        None => Invocation::Tui,
        Some(command) => Invocation::Run {
            command,
            args: cli.args,
        },
    }
}

/// Registre de tous les modules disponibles.
pub fn all_modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(CoreModule::new()),
        Box::new(devx_files::FilesModule::new()),
        Box::new(devx_tools::ToolsModule::new()),
    ]
}

/// Rassemble les [`CommandSpec`] de tous les modules (palette TUI et aide).
pub fn collect_specs(modules: &[Box<dyn Module>]) -> Vec<CommandSpec> {
    modules
        .iter()
        .flat_map(|module| module.commands())
        .collect()
}

/// Aide complète : toutes les commandes, groupées par module.
pub fn help_lines(modules: &[Box<dyn Module>]) -> Vec<String> {
    let mut lines = vec!["devx — commandes disponibles :".to_string()];
    for module in modules {
        lines.push(format!("[{}]", module.name()));
        for spec in module.commands() {
            lines.push(format!("  {:<22} {}", spec.usage, spec.description));
        }
    }
    lines
}

/// Route `command` vers le premier module qui la reconnaît.
///
/// Un module qui répond [`DevxError::UnknownCommand`] est ignoré et l'on passe
/// au suivant ; toute autre erreur est propagée. Si aucun module ne connaît la
/// commande, [`DevxError::UnknownCommand`] est renvoyée.
pub fn dispatch(
    modules: &[Box<dyn Module>],
    ctx: &Context,
    command: &str,
    args: &[String],
) -> Result<Output, DevxError> {
    for module in modules {
        match module.execute(ctx, command, args) {
            Ok(output) => return Ok(output),
            Err(DevxError::UnknownCommand(_)) => continue,
            Err(err) => return Err(err),
        }
    }
    Err(DevxError::unknown_command(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};
    use std::path::PathBuf;

    /// Module factice : ne connaît que `greet`.
    struct FakeModule;

    impl Module for FakeModule {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn commands(&self) -> Vec<CommandSpec> {
            vec![CommandSpec::new("greet", "salue", "greet <name>")]
        }

        fn execute(
            &self,
            _ctx: &Context,
            command: &str,
            args: &[String],
        ) -> Result<Output, DevxError> {
            match command {
                "greet" => {
                    let who = args.first().cloned().unwrap_or_else(|| "monde".to_string());
                    Ok(Output::new(vec![format!("bonjour {who}")]))
                }
                "boom" => Err(DevxError::message("boum")),
                other => Err(DevxError::unknown_command(other)),
            }
        }
    }

    fn ctx() -> Context {
        Context::new(Workspace::new(PathBuf::from(".")), Config::default())
    }

    fn modules() -> Vec<Box<dyn Module>> {
        vec![Box::new(CoreModule::new()), Box::new(FakeModule)]
    }

    // --- parsing ---

    #[test]
    fn no_args_is_tui() {
        let cli = Cli::try_parse_from(["devx"]).unwrap();
        assert_eq!(classify(cli), Invocation::Tui);
    }

    #[test]
    fn command_maps_to_run() {
        let cli = Cli::try_parse_from(["devx", "files", "find", "x"]).unwrap();
        assert_eq!(
            classify(cli),
            Invocation::Run {
                command: "files".to_string(),
                args: vec!["find".to_string(), "x".to_string()],
            }
        );
    }

    #[test]
    fn edit_without_file_carries_no_args() {
        let cli = Cli::try_parse_from(["devx", "edit"]).unwrap();
        assert_eq!(
            classify(cli),
            Invocation::Run {
                command: "edit".to_string(),
                args: vec![],
            }
        );
    }

    #[test]
    fn edit_with_file_carries_it_as_arg() {
        let cli = Cli::try_parse_from(["devx", "edit", "src/main.rs"]).unwrap();
        assert_eq!(
            classify(cli),
            Invocation::Run {
                command: "edit".to_string(),
                args: vec!["src/main.rs".to_string()],
            }
        );
    }

    #[test]
    fn trailing_hyphen_args_are_allowed() {
        let cli = Cli::try_parse_from(["devx", "echo", "--foo", "-x"]).unwrap();
        assert_eq!(
            classify(cli),
            Invocation::Run {
                command: "echo".to_string(),
                args: vec!["--foo".to_string(), "-x".to_string()],
            }
        );
    }

    #[test]
    fn unknown_flag_is_rejected() {
        assert!(Cli::try_parse_from(["devx", "--nope"]).is_err());
    }

    // --- dispatch ---

    #[test]
    fn dispatch_routes_to_fake_module() {
        let out = dispatch(&modules(), &ctx(), "greet", &["Alice".to_string()]).unwrap();
        assert_eq!(out.lines, vec!["bonjour Alice".to_string()]);
    }

    #[test]
    fn dispatch_routes_to_native_core() {
        let out = dispatch(&modules(), &ctx(), "version", &[]).unwrap();
        assert!(out.lines[0].starts_with("devx "));
    }

    #[test]
    fn dispatch_falls_back_when_unknown() {
        let err = dispatch(&modules(), &ctx(), "nope", &[]).unwrap_err();
        assert_eq!(err, DevxError::UnknownCommand("nope".to_string()));
    }

    #[test]
    fn dispatch_propagates_non_unknown_errors() {
        let err = dispatch(&modules(), &ctx(), "boom", &[]).unwrap_err();
        assert_eq!(err, DevxError::message("boum"));
    }

    #[test]
    fn collect_specs_includes_every_module() {
        let specs = collect_specs(&modules());
        let names: Vec<&str> = specs.iter().map(|spec| spec.name.as_str()).collect();
        assert!(names.contains(&"help"));
        assert!(names.contains(&"greet"));
    }

    #[test]
    fn all_modules_exposes_core() {
        assert!(all_modules().iter().any(|module| module.name() == "core"));
    }

    #[test]
    fn all_modules_exposes_files_and_tools() {
        let names: Vec<&str> = all_modules().iter().map(|module| module.name()).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"files"));
        assert!(names.contains(&"tools"));
    }

    #[test]
    fn help_lists_every_module() {
        let lines = help_lines(&all_modules());
        let joined = lines.join("\n");
        assert!(joined.contains("[core]"));
        assert!(joined.contains("[files]"));
        assert!(joined.contains("[tools]"));
    }
}
