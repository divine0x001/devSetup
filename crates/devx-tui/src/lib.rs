//! # devx-tui
//!
//! IDE terminal pour `devx`, construit sur `ratatui` + `crossterm`.
//!
//! - Barre de statut (haut) : mode, fichier courant, position ligne:colonne,
//!   indicateur de modification.
//! - Corps : arbre de fichiers (gauche) | éditeur (centre) ; panneau de sortie
//!   (bas) pour les commandes de modules.
//! - Barre de commande (bas) : mode `:` (`:help`, `:w`, `:q`, `:wq`, `:q!`,
//!   `:e <fichier>`, ou toute commande de module dispatchée via l'hôte).
//! - Palette `Ctrl-P` listant les [`CommandSpec`] de l'hôte.
//!
//! Le terminal est toujours restauré, y compris en cas d'erreur (garde RAII).

mod app;
mod editor;
mod ui;

pub use app::{
    matching_commands, parse_command_line, App, Executor, Focus, GitState, Mode, Node, View,
    Visible,
};
pub use editor::Buffer;

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use devx_core::{CommandSpec, Context, DevxError};

/// Lance la TUI en mode classique (sans fichier initial). Équivalent de
/// [`run_ide`] avec `initial_file = None`.
pub fn run(ctx: Context, specs: Vec<CommandSpec>, exec: Executor) -> Result<(), DevxError> {
    run_ide(ctx, specs, exec, None)
}

/// Lance l'IDE avec un éventuel fichier déjà ouvert dans l'éditeur.
///
/// `specs` alimente la palette `Ctrl-P`, `exec` exécute les commandes de
/// modules saisies dans le mode `:`. Retourne lorsque l'utilisateur quitte
/// (`q` depuis l'arbre, `Ctrl-C`, `:q`, `:wq`, `:q!`).
pub fn run_ide(
    ctx: Context,
    specs: Vec<CommandSpec>,
    exec: Executor,
    initial_file: Option<PathBuf>,
) -> Result<(), DevxError> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal: Terminal<CrosstermBackend<Stdout>> =
        Terminal::new(backend).map_err(to_err)?;

    let mut app = App::new(ctx, specs, exec, initial_file);

    while !app.should_quit {
        terminal
            .draw(|frame| ui::draw(frame, &mut app))
            .map_err(to_err)?;

        if event::poll(Duration::from_millis(200)).map_err(to_err)? {
            if let Event::Key(key) = event::read().map_err(to_err)? {
                app.on_key(key);
            }
        }
    }

    Ok(())
}

/// Convertit toute erreur affichable en [`DevxError`].
fn to_err<E: std::fmt::Display>(err: E) -> DevxError {
    DevxError::other(err.to_string())
}

/// Garde RAII : bascule le terminal en mode brut/alternatif et restaure l'état
/// au `Drop`, même en cas de panique.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, DevxError> {
        enable_raw_mode().map_err(to_err)?;
        execute!(io::stdout(), EnterAlternateScreen).map_err(to_err)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
