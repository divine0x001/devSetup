//! Binaire `devx` : mode TUI (sans argument) ou one-shot (`devx <command> ...`).

use std::env;

use clap::Parser;

use devx_cli::{all_modules, classify, collect_specs, dispatch, Cli, Invocation};
use devx_core::{Context, DevxError};

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
        Invocation::Tui => {
            let ctx = Context::with_root(env::current_dir()?);
            let specs = collect_specs(&modules);
            let exec: devx_tui::Executor =
                Box::new(move |ctx, command, args| dispatch(&modules, ctx, command, args));
            devx_tui::run(ctx, specs, exec)
        }
        Invocation::Run { command, args } => {
            let ctx = Context::with_root(env::current_dir()?);
            let output = dispatch(&modules, &ctx, &command, &args)?;
            for line in output.lines {
                println!("{line}");
            }
            Ok(())
        }
    }
}
