//! Binaire `devx` : mode TUI (sans argument, ou `edit [fichier]`) et exécution
//! one-shot (`devx <command> ...`).

use std::env;
use std::path::PathBuf;

use clap::Parser;

use devx_cli::{all_modules, classify, collect_specs, dispatch, help_lines, Cli, Invocation};
use devx_core::{Context, DevxError, Module, Output};

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("devx: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), DevxError> {
    let modules = all_modules();

    match classify(cli) {
        Invocation::Tui => launch_ide(modules, None),
        Invocation::Run { command, args } => {
            if command == "edit" {
                return launch_ide(modules, args.first().map(PathBuf::from));
            }
            run_one_shot(&modules, &command, &args)
        }
    }
}

/// Lance la TUI/IDE avec un éventuel fichier initial ouvert.
fn launch_ide(
    modules: Vec<Box<dyn Module>>,
    initial_file: Option<PathBuf>,
) -> Result<(), DevxError> {
    let ctx = Context::with_root(env::current_dir()?);
    let specs = collect_specs(&modules);
    let exec: devx_tui::Executor =
        Box::new(move |ctx, command, args| dispatch(&modules, ctx, command, args));
    devx_tui::run_ide(ctx, specs, exec, initial_file)
}

/// Exécute une commande one-shot et imprime sa sortie (une ligne par ligne).
fn run_one_shot(
    modules: &[Box<dyn Module>],
    command: &str,
    args: &[String],
) -> Result<(), DevxError> {
    let ctx = Context::with_root(env::current_dir()?);
    let output = if command == "help" {
        Output::new(help_lines(modules))
    } else {
        dispatch(modules, &ctx, command, args)?
    };
    for line in output.lines {
        println!("{line}");
    }
    Ok(())
}
