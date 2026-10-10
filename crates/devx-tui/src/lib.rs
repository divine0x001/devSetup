//! # devx-tui
//!
//! Shell terminal interactif pour `devx`, construit sur `ratatui` + `crossterm`.
//!
//! - Colonne gauche : arbre de fichiers scanné depuis `ctx.workspace.root`.
//! - Zone centrale : sortie / prévisualisation des commandes.
//! - Barre de commande (bas) avec autocomplétion.
//! - Barre de statut (haut).
//! - Palette de commandes (`Ctrl-P`) listant les [`CommandSpec`] de l'hôte.
//!
//! Le terminal est toujours restauré, y compris en cas d'erreur (garde RAII).

mod app;
mod ui;

pub use app::{matching_commands, App, Executor, Mode, Node, Visible};

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use devx_core::{CommandSpec, Context, DevxError};

/// Lance la boucle d'évènements de la TUI.
///
/// `specs` alimente la palette `Ctrl-P`, `exec` exécute les commandes saisies.
/// Retourne lorsque l'utilisateur quitte (`q` ou `Ctrl-C`).
pub fn run(ctx: Context, specs: Vec<CommandSpec>, exec: Executor) -> Result<(), DevxError> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal: Terminal<CrosstermBackend<Stdout>> =
        Terminal::new(backend).map_err(to_err)?;

    let mut app = App::new(ctx, specs, exec);

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
