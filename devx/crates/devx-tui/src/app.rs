//! État applicatif de la TUI : arbre de fichiers, éditeur, sortie, événements.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use devx_core::{CommandSpec, Context, DevxError, Output};
use ratatui::layout::Rect;

use crate::editor::Buffer;
use crate::git::load_git;
use crate::tree::{expanded_paths, flatten, restore_expanded, scan_dir, set_expanded_by_path};

pub use crate::git::GitState;
pub use crate::tree::{Node, Visible};

/// Largeur (en cellules) de la gouttière de numéros de ligne dans l'éditeur.
pub const EDITOR_GUTTER: usize = 6;

/// Largeur d'une tabulation dans l'éditeur (insertion d'espaces).
pub const EDITOR_TAB: usize = 4;

/// Callback d'exécution de commande fourni par l'hôte (le binaire `devx`).
///
/// La TUI reste découplée des modules : elle délègue l'exécution.
pub type Executor = Box<dyn FnMut(&Context, &str, &[String]) -> Result<Output, DevxError>>;

/// Profondeur maximale de scan de l'arbre de fichiers.
const MAX_DEPTH: usize = 6;

/// Nombre maximal d'entrées scannées (garde-fou contre les arbres géants).
const MAX_ENTRIES: usize = 20_000;

/// Commandes éditeur disponibles dans le mode `:` (nom, description).
const EDITOR_COMMANDS: [(&str, &str); 15] = [
    (":help", "liste toutes les commandes disponibles"),
    (":<n> | :goto <n>", "place le curseur sur la ligne n"),
    (":w", "sauvegarde le fichier courant"),
    (":wq", "sauvegarde puis quitte"),
    (":wa", "sauvegarde tous les onglets modifiés"),
    (":wqa", "sauvegarde tous les onglets puis quitte"),
    (":q", "quitte (refusé si modifications en attente)"),
    (":q!", "quitte sans sauvegarder"),
    (
        ":e <fichier>",
        "ouvre un fichier et passe le focus à l'éditeur",
    ),
    (
        ":new <fichier>",
        "crée un nouveau fichier et l'ouvre dans l'éditeur",
    ),
    (":next", "onglet suivant"),
    (":prev", "onglet précédent"),
    (":close", "ferme l'onglet courant (refusé si modifié)"),
    (":git", "affiche un rapport git à la place du code"),
    (":code", "revient à la vue code (arbre + éditeur)"),
];

/// Raccourcis clavier de l'éditeur (touche, description), affichés par `:help`.
const EDITOR_SHORTCUTS: [(&str, &str); 13] = [
    ("Ctrl-S", "sauvegarde le fichier courant"),
    ("Ctrl-Z / Ctrl-Y", "annule / rétablit"),
    ("Maj + flèches", "sélectionne du texte"),
    ("Ctrl-C/X/V", "copier / couper / coller"),
    ("Ctrl-F", "recherche incrémentale (Entrée: suivant)"),
    ("Ctrl-D / Ctrl-K", "duplique / supprime la ligne"),
    ("Ctrl-W", "supprime le mot précédent"),
    ("Ctrl-←/→", "onglet précédent / suivant"),
    ("Ctrl-P", "palette de commandes"),
    ("Alt-←/→", "déplacement par mot"),
    ("Alt-↑/↓", "déplace la ligne"),
    ("Ctrl-Home/End", "début / fin du document"),
    ("Tab / Maj-Tab", "indente / désindente"),
];

/// Paneau qui reçoit les frappes de navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// Arbre de fichiers (gauche).
    Tree,
    /// Éditeur (centre).
    Editor,
}

/// Mode d'interaction courant (au-dessus du [`Focus`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Navigation / raccourcis.
    Normal,
    /// Saisie dans la barre de commande (`:`).
    Command,
    /// Palette de commandes (`Ctrl-P`).
    Palette,
    /// Recherche incrémentale dans l'éditeur (`Ctrl-F`).
    Search,
}

/// Vue principale affichée par l'IDE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Vue « code » : arbre de fichiers + éditeur (vue par défaut).
    Code,
    /// Vue « git » : branche, statut et derniers commits.
    Git,
}

/// Géométrie du dernier rendu, utilisée pour la souris (carte de zones).
#[derive(Debug, Clone, Default)]
pub struct LayoutInfo {
    /// Zone de l'arbre de fichiers.
    pub tree: Rect,
    /// Défilement vertical appliqué à l'arbre.
    pub tree_offset: usize,
    /// Zone de la barre d'onglets (au-dessus de l'éditeur).
    pub tabs: Rect,
    /// Rects individuels de chaque onglet (clic → sélection).
    pub tab_spans: Vec<Rect>,
    /// Zone de l'éditeur (cadre compris).
    pub editor: Rect,
    /// Défilement vertical de l'éditeur.
    pub editor_y_offset: usize,
    /// Défilement horizontal de l'éditeur.
    pub editor_x_offset: usize,
}

/// État complet de l'application TUI (IDE).
pub struct App {
    /// Contexte d'exécution (workspace + config).
    pub ctx: Context,
    /// Commandes disponibles (tous modules confondus).
    pub specs: Vec<CommandSpec>,
    /// Paneau focalisé.
    pub focus: Focus,
    /// Mode courant.
    pub mode: Mode,
    /// Demande de sortie de la boucle.
    pub should_quit: bool,
    /// Arbre de fichiers du workspace.
    pub nodes: Vec<Node>,
    /// Index de la ligne sélectionnée dans l'arbre.
    pub selection: usize,
    /// Onglets ouverts (un tampon par onglet) ; `tabs[active]` est édité.
    pub tabs: Vec<Buffer>,
    /// Index de l'onglet actif dans `tabs`.
    pub active: usize,
    /// Lignes de la zone de sortie.
    pub output: Vec<String>,
    /// Contenu de la barre de commande.
    pub input: String,
    /// Index sélectionné dans la palette.
    pub palette_selected: usize,
    /// Motif de recherche courant (mode [`Mode::Search`]).
    pub search_query: String,
    /// Position du curseur à l'entrée en recherche (pour annuler/boucler).
    search_origin: (usize, usize),
    /// Presse-papier interne (copier/couper/coller), partagé entre onglets.
    clipboard: String,
    /// Message de statut.
    pub status: String,
    /// Vue principale courante.
    pub view: View,
    /// Informations git (vue « git »).
    pub git: GitState,
    /// Géométrie du dernier rendu (carte pour la souris).
    pub layout: LayoutInfo,
    executor: Executor,
}

impl App {
    /// Crée une application à partir du contexte, des specs, de l'exécuteur et
    /// d'un fichier initial éventuel (ouvert dans l'éditeur).
    pub fn new(
        ctx: Context,
        specs: Vec<CommandSpec>,
        executor: Executor,
        initial_file: Option<PathBuf>,
    ) -> Self {
        let mut budget = MAX_ENTRIES;
        let nodes = scan_dir(&ctx.workspace.root, MAX_DEPTH, &mut budget);
        let mut output = vec![
            "devx — VS Code dans le terminal".to_string(),
            "Onglets: Ctrl-←/→ · Tab: changer de panneau · : commande · Ctrl-P: palette · q: quitter"
                .to_string(),
        ];

        let buffer = match initial_file {
            Some(path) => {
                let path = if path.is_absolute() {
                    path
                } else {
                    ctx.workspace.root.join(path)
                };
                match Self::load_into(&path) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        output.push(format!("error: {err}"));
                        Buffer::empty()
                    }
                }
            }
            None => Buffer::empty(),
        };
        let focus = if buffer.path.is_some() {
            Focus::Editor
        } else {
            Focus::Tree
        };

        Self {
            ctx,
            specs,
            focus,
            mode: Mode::Normal,
            should_quit: false,
            nodes,
            selection: 0,
            tabs: vec![buffer],
            active: 0,
            output,
            input: String::new(),
            palette_selected: 0,
            search_query: String::new(),
            search_origin: (0, 0),
            clipboard: String::new(),
            status: "prêt".to_string(),
            view: View::Code,
            git: GitState::default(),
            layout: LayoutInfo::default(),
            executor,
        }
    }

    /// Tampon de l'onglet actif (lecture).
    pub fn editor(&self) -> &Buffer {
        &self.tabs[self.active]
    }

    /// Tampon de l'onglet actif (édition).
    pub fn editor_mut(&mut self) -> &mut Buffer {
        let active = self.active;
        &mut self.tabs[active]
    }

    /// Nombre d'onglets ouverts.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Passe à l'onglet voisin (bouclant) ; sans effet avec un seul onglet.
    pub fn switch_tab(&mut self, delta: isize) {
        let n = self.tabs.len();
        if n < 2 {
            return;
        }
        self.active = (self.active as isize + delta).rem_euclid(n as isize) as usize;
        self.status = format!("onglet {}/{}", self.active + 1, n);
    }

    /// Ouvre un tampon dans un onglet : si un onglet pointe déjà vers le même
    /// fichier, il est simplement activé (les modifications en attente sont
    /// préservées) ; sinon l'onglet est ajouté en fin de liste.
    pub fn open_tab(&mut self, buffer: Buffer) {
        if let Some(pos) = self.tabs.iter().position(|tab| tab.path == buffer.path) {
            self.active = pos;
        } else {
            self.tabs.push(buffer);
            self.active = self.tabs.len() - 1;
        }
        self.focus = Focus::Editor;
    }

    /// Ferme l'onglet actif : refusé si modifié ; avec le dernier onglet.
    /// ouvert, remplace par un tampon vide.
    pub fn close_tab(&mut self) {
        if self.editor().dirty {
            let msg = "onglet modifié — :w puis :close, ou :q!".to_string();
            self.output.push(msg.clone());
            self.status = msg;
            return;
        }
        self.tabs.remove(self.active);
        if self.tabs.is_empty() {
            self.tabs.push(Buffer::empty());
        }
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.status = format!("onglet {}/{} fermé", self.active + 1, self.tabs.len());
    }

    /// Noms des onglets ayant des modifications en attente (tous onglets).
    fn dirty_tab_names(&self) -> Vec<String> {
        self.tabs
            .iter()
            .filter(|tab| tab.dirty)
            .map(Buffer::display_name)
            .collect()
    }

    /// Quitte si aucun onglet n'a de modification en attente ; sinon refuse en
    /// nommant les onglets concernés (évite la perte d'un onglet non actif).
    fn quit_if_clean(&mut self) {
        let dirty = self.dirty_tab_names();
        if dirty.is_empty() {
            self.should_quit = true;
        } else {
            let msg = format!(
                "modifications non sauvegardées ({}) — :w puis :q, ou :q!",
                dirty.join(", ")
            );
            self.output.push(msg.clone());
            self.status = msg;
        }
    }

    /// Aplati l'arbre en tenant compte des répertoires dépliés.
    pub fn visible(&self) -> Vec<Visible> {
        let mut out = Vec::new();
        flatten(&self.nodes, 0, &mut out);
        out
    }

    /// Commandes correspondant au préfixe saisi.
    pub fn suggestions(&self) -> Vec<&CommandSpec> {
        matching_commands(&self.specs, &self.input)
    }

    /// Texte d'aide : commandes éditeur puis commandes des modules.
    pub fn help_text(&self) -> Vec<String> {
        let mut lines = vec!["devx — commandes :".to_string(), "[éditeur]".to_string()];
        for (name, description) in EDITOR_COMMANDS {
            lines.push(format!("  {:<18} {description}", name));
        }
        lines.push(String::new());
        lines.push("[raccourcis]".to_string());
        for (keys, description) in EDITOR_SHORTCUTS {
            lines.push(format!("  {:<18} {description}", keys));
        }
        lines.push(String::new());
        lines.push("[modules]".to_string());
        for spec in &self.specs {
            lines.push(format!("  {:<22} {}", spec.usage, spec.description));
        }
        lines
    }

    /// Traite un événement clavier.
    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    // Copie la sélection si l'éditeur en a une ; sinon quitte.
                    if self.focus == Focus::Editor {
                        if let Some(text) = self.editor().selected_text() {
                            self.clipboard = text;
                            self.editor_mut().clear_selection();
                            self.status = "copié".to_string();
                            return;
                        }
                    }
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('x') => {
                    if self.focus == Focus::Editor {
                        if let Some(text) = self.editor().selected_text() {
                            self.clipboard = text;
                            self.editor_mut().delete_selection();
                            self.status = "coupé".to_string();
                        }
                    }
                    return;
                }
                KeyCode::Char('v') => {
                    if self.focus == Focus::Editor && !self.clipboard.is_empty() {
                        let text = self.clipboard.clone();
                        self.editor_mut().insert_str(&text);
                        self.status = "collé".to_string();
                    }
                    return;
                }
                KeyCode::Char('p') => {
                    self.mode = if self.mode == Mode::Palette {
                        Mode::Normal
                    } else {
                        Mode::Palette
                    };
                    self.palette_selected = 0;
                    return;
                }
                KeyCode::Char('s') => {
                    if self.focus == Focus::Editor {
                        self.save_editor();
                    }
                    return;
                }
                KeyCode::Char('g') => {
                    self.toggle_view();
                    return;
                }
                KeyCode::Char('n') => {
                    self.mode = Mode::Command;
                    self.input = "new ".to_string();
                    self.palette_selected = 0;
                    self.status = "nouveau fichier — tape le nom puis Entrée".to_string();
                    return;
                }
                KeyCode::Char('f') => {
                    if self.focus == Focus::Editor {
                        self.enter_search();
                    }
                    return;
                }
                KeyCode::Char('w') => {
                    if self.focus == Focus::Editor {
                        self.editor_mut().delete_word_left();
                    }
                    return;
                }
                KeyCode::Char('d') => {
                    if self.focus == Focus::Editor {
                        self.editor_mut().duplicate_line();
                    }
                    return;
                }
                KeyCode::Char('k') => {
                    if self.focus == Focus::Editor {
                        self.editor_mut().delete_line();
                    }
                    return;
                }
                KeyCode::Char('z') => {
                    if self.focus == Focus::Editor {
                        self.status = if self.editor_mut().undo() {
                            "annulé".to_string()
                        } else {
                            "rien à annuler".to_string()
                        };
                    }
                    return;
                }
                KeyCode::Char('y') => {
                    if self.focus == Focus::Editor {
                        self.status = if self.editor_mut().redo() {
                            "rétabli".to_string()
                        } else {
                            "rien à rétablir".to_string()
                        };
                    }
                    return;
                }
                KeyCode::Left => {
                    self.switch_tab(-1);
                    return;
                }
                KeyCode::Right => {
                    self.switch_tab(1);
                    return;
                }
                KeyCode::Home if self.focus == Focus::Editor => {
                    self.editor_mut().move_doc_start();
                    return;
                }
                KeyCode::End if self.focus == Focus::Editor => {
                    self.editor_mut().move_doc_end();
                    return;
                }
                _ => {}
            }
        }

        match self.mode {
            Mode::Command => self.on_key_command(key),
            Mode::Palette => self.on_key_palette(key),
            Mode::Search => self.on_key_search(key),
            Mode::Normal => match key.code {
                KeyCode::Char(':') | KeyCode::Char('/') => {
                    self.mode = Mode::Command;
                    self.input.clear();
                    self.status = "commande (:) — Échap pour annuler".to_string();
                }
                KeyCode::Tab if self.focus == Focus::Editor => {
                    self.editor_mut().insert_tab(EDITOR_TAB);
                }
                KeyCode::BackTab if self.focus == Focus::Editor => {
                    self.editor_mut().remove_tab(EDITOR_TAB);
                }
                KeyCode::Tab => self.toggle_focus(),
                _ => match self.focus {
                    Focus::Tree => self.on_key_tree(key),
                    Focus::Editor => self.on_key_editor(key),
                },
            },
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => Focus::Editor,
            Focus::Editor => Focus::Tree,
        };
        self.status = match self.focus {
            Focus::Tree => "focus: arbre de fichiers".to_string(),
            Focus::Editor => "focus: éditeur".to_string(),
        };
    }

    /// Traite un événement souris (clic, molette).
    pub fn on_mouse(&mut self, mouse: MouseEvent) {
        if self.mode != Mode::Normal {
            return;
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.on_left_click(mouse.column, mouse.row),
            MouseEventKind::ScrollUp => self.on_scroll(mouse.column, mouse.row, -1),
            MouseEventKind::ScrollDown => self.on_scroll(mouse.column, mouse.row, 1),
            _ => {}
        }
    }

    fn on_left_click(&mut self, x: u16, y: u16) {
        if inside(self.layout.tabs, x, y) {
            self.on_tab_click(x);
        } else if inside(self.layout.tree, x, y) {
            self.on_tree_click(x, y);
        } else if inside(self.layout.editor, x, y) {
            self.on_editor_click(x, y);
        }
    }

    /// Clic dans la barre d'onglets : active l'onglet cliqué.
    fn on_tab_click(&mut self, x: u16) {
        let Some(pos) = self
            .layout
            .tab_spans
            .iter()
            .position(|r| r.width > 0 && x >= r.x && x < r.x + r.width)
        else {
            return;
        };
        self.active = pos;
        self.status = format!("onglet {}/{}", pos + 1, self.tabs.len());
    }

    pub fn on_scroll(&mut self, x: u16, y: u16, delta: i32) {
        if inside(self.layout.editor, x, y) {
            for _ in 0..delta.abs() {
                if delta < 0 {
                    self.editor_mut().move_up();
                } else {
                    self.editor_mut().move_down();
                }
            }
            self.focus = Focus::Editor;
        } else if inside(self.layout.tree, x, y) {
            let len = self.visible().len();
            if len == 0 {
                return;
            }
            let next = self.selection as i32 + delta;
            self.selection = next.clamp(0, len as i32 - 1) as usize;
        }
    }

    fn on_tree_click(&mut self, _x: u16, y: u16) {
        let area = self.layout.tree;
        let inner_y = area.y as usize + 1;
        if (y as usize) < inner_y {
            return;
        }
        let row = (y as usize - inner_y) + self.layout.tree_offset;
        if row >= self.visible().len() {
            return;
        }
        self.selection = row;
        self.focus = Focus::Tree;
        self.open_selected();
    }

    /// Place le curseur de l'éditeur sur le caractère cliqué.
    fn on_editor_click(&mut self, x: u16, y: u16) {
        let area = self.layout.editor;
        let origin_x = area.x as i32 + 1 + EDITOR_GUTTER as i32;
        let origin_y = area.y as i32 + 1;
        let rel_x = x as i32 - origin_x;
        let rel_y = y as i32 - origin_y;
        if rel_y < 0 {
            return;
        }
        let last_row = self.editor().lines.len().saturating_sub(1);
        let row = ((rel_y as usize) + self.layout.editor_y_offset).min(last_row);
        let col_vis = if rel_x < 0 { 0 } else { rel_x as usize };
        let line_len = self.editor().lines[row].chars().count();
        let col = (col_vis + self.layout.editor_x_offset).min(line_len);
        let tab = self.editor_mut();
        tab.clear_selection();
        tab.cursor_row = row;
        tab.cursor_col = col;
        self.focus = Focus::Editor;
        self.status = format!("curseur {}:{}", row + 1, col + 1);
    }

    /// Bascule entre la vue code et la vue git (Ctrl-G).
    pub fn toggle_view(&mut self) {
        match self.view {
            View::Code => self.open_git_view(),
            View::Git => self.open_code_view(),
        }
    }

    /// Affiche la vue git (recharge le rapport).
    pub fn open_git_view(&mut self) {
        self.refresh_git();
        self.view = View::Git;
        self.status = "vue git".to_string();
    }

    /// Revient à la vue code.
    pub fn open_code_view(&mut self) {
        self.view = View::Code;
        self.status = "vue code".to_string();
    }

    /// Recharge l'état git depuis le workspace.
    pub fn refresh_git(&mut self) {
        self.git = load_git(&self.ctx.workspace.root);
    }

    /// Re-scan l'arbre de fichiers en préservant l'état déplié des répertoires.
    pub fn refresh_tree(&mut self) {
        let expanded: HashSet<PathBuf> = self.nodes.iter().flat_map(expanded_paths).collect();
        let mut budget = MAX_ENTRIES;
        let mut nodes = scan_dir(&self.ctx.workspace.root, MAX_DEPTH, &mut budget);
        restore_expanded(&mut nodes, &expanded);
        self.nodes = nodes;
    }

    fn on_key_tree(&mut self, key: KeyEvent) {
        let len = self.visible().len();
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selection = self.selection.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.selection + 1 < len {
                    self.selection += 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => self.open_selected(),
            KeyCode::Left | KeyCode::Char('h') => self.set_expanded(false),
            _ => {}
        }
    }

    /// Nombre de lignes par « page » (hauteur interne de l'éditeur, ≥ 1).
    fn editor_page(&self) -> usize {
        (self.layout.editor.height as usize)
            .saturating_sub(2)
            .max(1)
    }

    /// Prépare la sélection avant un déplacement : l'étend avec Maj (pose une
    /// ancre au besoin), sinon l'annule.
    fn prepare_selection(&mut self, shift: bool) {
        let ed = self.editor_mut();
        if shift {
            ed.anchor_selection();
        } else {
            ed.clear_selection();
        }
    }

    fn on_key_editor(&mut self, key: KeyEvent) {
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            // Alt + flèches : opérations sur les mots / lignes (pas de sélection).
            KeyCode::Left if alt => self.editor_mut().move_word_left(),
            KeyCode::Right if alt => self.editor_mut().move_word_right(),
            KeyCode::Up if alt => self.editor_mut().move_line_up(),
            KeyCode::Down if alt => self.editor_mut().move_line_down(),
            // Déplacements du curseur : Maj étend la sélection.
            KeyCode::Up => {
                self.prepare_selection(shift);
                self.editor_mut().move_up();
            }
            KeyCode::Down => {
                self.prepare_selection(shift);
                self.editor_mut().move_down();
            }
            KeyCode::Left => {
                self.prepare_selection(shift);
                self.editor_mut().move_left();
            }
            KeyCode::Right => {
                self.prepare_selection(shift);
                self.editor_mut().move_right();
            }
            KeyCode::Home => {
                self.prepare_selection(shift);
                self.editor_mut().move_home();
            }
            KeyCode::End => {
                self.prepare_selection(shift);
                self.editor_mut().move_end();
            }
            KeyCode::PageUp => {
                self.prepare_selection(shift);
                let page = self.editor_page();
                self.editor_mut().page_up(page);
            }
            KeyCode::PageDown => {
                self.prepare_selection(shift);
                let page = self.editor_page();
                self.editor_mut().page_down(page);
            }
            // Édition : une sélection active est d'abord supprimée.
            KeyCode::Backspace => {
                if !self.editor_mut().delete_selection() {
                    self.editor_mut().backspace();
                }
            }
            KeyCode::Delete => {
                if !self.editor_mut().delete_selection() {
                    self.editor_mut().delete();
                }
            }
            KeyCode::Enter => {
                self.editor_mut().delete_selection();
                self.editor_mut().insert_newline();
            }
            KeyCode::Esc => {
                if self.editor().selection_range().is_some() {
                    self.editor_mut().clear_selection();
                    self.status = "sélection annulée".to_string();
                } else {
                    self.focus = Focus::Tree;
                    self.status = "focus: arbre de fichiers".to_string();
                }
            }
            KeyCode::Char(c) => {
                self.editor_mut().delete_selection();
                self.editor_mut().insert_char(c);
            }
            _ => {}
        }
    }

    fn on_key_command(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input.clear();
                self.status = "prêt".to_string();
            }
            KeyCode::Enter => self.submit(),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Tab => self.autocomplete(),
            KeyCode::Char(c) => self.input.push(c),
            _ => {}
        }
    }

    /// Entre en recherche incrémentale depuis l'éditeur (Ctrl-F).
    fn enter_search(&mut self) {
        let ed = self.editor();
        self.search_origin = (ed.cursor_row, ed.cursor_col);
        self.search_query.clear();
        self.mode = Mode::Search;
        self.status = "recherche — Entrée: occurrence suivante · Échap: annuler".to_string();
    }

    fn on_key_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let (row, col) = self.search_origin;
                let ed = self.editor_mut();
                ed.cursor_row = row;
                ed.cursor_col = col;
                self.mode = Mode::Normal;
                self.search_query.clear();
                self.status = "recherche annulée".to_string();
            }
            KeyCode::Enter => {
                if self.search_query.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.search_next();
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.search_from_origin();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.search_from_origin();
            }
            _ => {}
        }
    }

    /// Recherche depuis la position d'origine (saisie incrémentale).
    fn search_from_origin(&mut self) {
        let (row, col) = self.search_origin;
        self.apply_search(row, col);
    }

    /// Recherche à partir du caractère suivant le curseur (occurrence suivante).
    fn search_next(&mut self) {
        let ed = self.editor();
        let (row, col) = (ed.cursor_row, ed.cursor_col + 1);
        self.apply_search(row, col);
    }

    /// Déplace le curseur sur la prochaine occurrence du motif et met à jour le
    /// statut (position trouvée, ou « introuvable »).
    fn apply_search(&mut self, row: usize, col: usize) {
        if self.search_query.is_empty() {
            self.status = "recherche : ".to_string();
            return;
        }
        match self.editor().find_from(&self.search_query, row, col) {
            Some((r, c)) => {
                let ed = self.editor_mut();
                ed.cursor_row = r;
                ed.cursor_col = c;
                self.status = format!("/{}  →  {}:{}", self.search_query, r + 1, c + 1);
            }
            None => {
                self.status = format!("/{}  introuvable", self.search_query);
            }
        }
    }

    fn on_key_palette(&mut self, key: KeyEvent) {
        let len = self.specs.len();
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Up => self.palette_selected = self.palette_selected.saturating_sub(1),
            KeyCode::Down => {
                if len > 0 && self.palette_selected + 1 < len {
                    self.palette_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(spec) = self.specs.get(self.palette_selected) {
                    self.input = format!("{} ", spec.name);
                }
                self.mode = Mode::Command;
                self.status = "commande (:) — Échap pour annuler".to_string();
            }
            _ => {}
        }
    }

    /// Ouvre la sélection de l'arbre : déplie un répertoire, charge un fichier
    /// dans l'éditeur (focus → éditeur).
    fn open_selected(&mut self) {
        let visible = self.visible();
        let Some(row) = visible.get(self.selection) else {
            return;
        };
        if row.is_dir {
            self.set_expanded(true);
            return;
        }
        let path = row.path.clone();
        match Self::load_into(&path) {
            Ok(buffer) => {
                self.open_tab(buffer);
                self.status = format!("ouvert {}", path.display());
            }
            Err(err) => self.output.push(format!("error: {err}")),
        }
    }

    /// Charge un tampon depuis un chemin ; crée un tampon vide si le fichier
    /// n'existe pas encore.
    fn load_into(path: &Path) -> Result<Buffer, DevxError> {
        if path.is_file() {
            Buffer::load(path)
        } else {
            Ok(Buffer::empty_with_path(path))
        }
    }

    fn set_expanded(&mut self, expand: bool) {
        let visible = self.visible();
        let Some(row) = visible.get(self.selection) else {
            return;
        };
        if !row.is_dir {
            return;
        }
        let path = row.path.clone();
        set_expanded_by_path(&mut self.nodes, &path, expand);
    }

    fn autocomplete(&mut self) {
        let (command, args) = parse_command_line(&self.input);
        // Pour `:e` / `:new`, complète l'argument chemin dans le workspace.
        if matches!(command.as_str(), "e" | "new") && self.input.ends_with(|c: char| c != ' ') {
            let partial = args.first().map(String::as_str).unwrap_or("");
            if let Some(completed) = complete_path(&self.ctx.workspace.root, partial) {
                self.input = format!("{command} {completed}");
                return;
            }
        }
        // Sinon, complète le nom de commande via les suggestions.
        let Some(spec) = self.suggestions().first().copied().cloned() else {
            return;
        };
        self.input = format!("{} ", spec.name);
    }

    /// Soumet la commande de la barre `:`: commande éditeur d'abord, sinon
    /// dispatch vers les modules via l'exécuteur.
    fn submit(&mut self) {
        let line = self.input.trim().to_string();
        self.input.clear();
        self.mode = Mode::Normal;
        self.status = "prêt".to_string();
        if line.is_empty() {
            return;
        }
        self.output.push(format!("devx> :{line}"));

        let (command, args) = parse_command_line(&line);
        if self.handle_editor_command(&command, &args) {
            return;
        }

        let App {
            ctx,
            executor,
            output,
            ..
        } = self;
        match executor(ctx, &command, &args) {
            Ok(result) => output.extend(result.lines),
            Err(err) => output.push(format!("error: {err}")),
        }
    }

    /// Gère les commandes éditeur du mode `:`. Renvoie `true` si consommée.
    fn handle_editor_command(&mut self, command: &str, args: &[String]) -> bool {
        // `:<n>` (commande purement numérique) saute à la ligne n.
        if let Ok(n) = command.parse::<usize>() {
            self.goto_line(n);
            return true;
        }
        match command {
            "goto" => {
                match args.first().and_then(|a| a.parse::<usize>().ok()) {
                    Some(n) => self.goto_line(n),
                    None => self.output.push("usage: :goto <ligne>".to_string()),
                }
                true
            }
            "help" => {
                for line in self.help_text() {
                    self.output.push(line);
                }
                self.status = "aide affichée".to_string();
                true
            }
            "w" => {
                self.save_editor();
                true
            }
            "wq" => {
                self.save_editor();
                self.quit_if_clean();
                true
            }
            "wa" => {
                self.save_all();
                true
            }
            "wqa" => {
                self.save_all();
                self.quit_if_clean();
                true
            }
            "q" => {
                self.quit_if_clean();
                true
            }
            "q!" => {
                self.should_quit = true;
                true
            }
            "next" => {
                self.switch_tab(1);
                true
            }
            "prev" => {
                self.switch_tab(-1);
                true
            }
            "close" => {
                self.close_tab();
                true
            }
            "git" => {
                self.open_git_view();
                true
            }
            "code" => {
                self.open_code_view();
                true
            }
            "e" => {
                match args.first() {
                    Some(file) => {
                        let path = PathBuf::from(file);
                        let path = if path.is_absolute() {
                            path
                        } else {
                            self.ctx.workspace.root.join(path)
                        };
                        match Self::load_into(&path) {
                            Ok(buffer) => {
                                self.open_tab(buffer);
                                self.status = format!("ouvert {}", path.display());
                            }
                            Err(err) => self.output.push(format!("error: :e {err}")),
                        }
                    }
                    None => self.output.push("usage: :e <fichier>".to_string()),
                }
                true
            }
            "new" => {
                let Some(file) = args.first() else {
                    self.output.push("usage: :new <fichier>".to_string());
                    return true;
                };
                let raw = Path::new(file);
                if raw.is_absolute() {
                    self.output
                        .push("error: chemin absolu refusé (relatif au workspace)".to_string());
                    return true;
                }
                let path = self.ctx.workspace.root.join(raw);
                // Garde-fou : le chemin résolu ne doit pas sortir du workspace.
                if !path
                    .strip_prefix(&self.ctx.workspace.root)
                    .map(|p| {
                        !p.components()
                            .any(|c| matches!(c, std::path::Component::ParentDir))
                    })
                    .unwrap_or(false)
                {
                    self.output
                        .push("error: chemin invalide (fuite hors workspace)".to_string());
                    return true;
                }
                if let Some(parent) = path.parent() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        self.output.push(format!("error: :new {err}"));
                        return true;
                    }
                }
                self.open_tab(Buffer::empty_with_path(&path));
                self.refresh_tree();
                self.status = format!("nouveau fichier {} — :w pour sauvegarder", path.display());
                true
            }
            _ => false,
        }
    }

    fn save_editor(&mut self) {
        match self.editor_mut().save() {
            Ok(()) => {
                let path = self
                    .editor()
                    .path
                    .as_deref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                self.output.push(format!("saved {path}"));
                self.status = "sauvegardé".to_string();
                self.refresh_tree();
            }
            Err(err) => {
                self.output.push(format!("error: {err}"));
                self.status = "erreur".to_string();
            }
        }
    }

    /// Sauvegarde tous les onglets modifiés qui ont un fichier associé. Les
    /// onglets sans chemin (« sans-titre ») sont laissés tels quels. Met à jour
    /// l'arbre et le statut avec le bilan.
    fn save_all(&mut self) {
        let mut saved = 0usize;
        let mut errors = Vec::new();
        for tab in &mut self.tabs {
            if tab.path.is_none() || !tab.dirty {
                continue;
            }
            match tab.save() {
                Ok(()) => saved += 1,
                Err(err) => errors.push(format!("error: {err}")),
            }
        }
        for err in &errors {
            self.output.push(err.clone());
        }
        self.output
            .push(format!("{saved} fichier(s) sauvegardé(s)"));
        self.status = format!("{saved} sauvegardé(s)");
        self.refresh_tree();
    }

    /// Déplace le curseur sur la ligne `n` (1-based, bornée), bascule en vue
    /// code et donne le focus à l'éditeur.
    fn goto_line(&mut self, n: usize) {
        if self.view != View::Code {
            self.open_code_view();
        }
        let last = self.editor().lines.len();
        let target = n.clamp(1, last.max(1)) - 1;
        let ed = self.editor_mut();
        ed.cursor_row = target;
        ed.cursor_col = 0;
        self.focus = Focus::Editor;
        self.status = format!("ligne {}", target + 1);
    }
}

/// Vrai si la position `(x, y)` tombe dans `area` (bordure comprise).
fn inside(area: Rect, x: u16, y: u16) -> bool {
    area.width > 0
        && area.height > 0
        && x >= area.x
        && x < area.x + area.width
        && y >= area.y
        && y < area.y + area.height
}

/// Complète un chemin partiel (relatif à `root`) pour `:e`/`:new`. Renvoie le
/// chemin complété (nom unique, ou plus long préfixe commun) s'il progresse,
/// sinon `None`. Les répertoires sont suffixés d'un `/`.
fn complete_path(root: &Path, partial: &str) -> Option<String> {
    // Sépare la partie répertoire (avec son `/` final) du préfixe à compléter.
    let (dir_rel, prefix) = match partial.rfind('/') {
        Some(i) => (&partial[..=i], &partial[i + 1..]),
        None => ("", partial),
    };
    let dir = root.join(dir_rel);

    let mut matches: Vec<String> = fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(prefix) {
                return None;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            Some(if is_dir { format!("{name}/") } else { name })
        })
        .collect();

    if matches.is_empty() {
        return None;
    }
    matches.sort();

    let completion = if matches.len() == 1 {
        matches[0].clone()
    } else {
        common_prefix(&matches)
    };
    if completion.len() <= prefix.len() {
        return None; // aucune progression
    }
    Some(format!("{dir_rel}{completion}"))
}

/// Plus long préfixe commun (en `char`) d'un ensemble de chaînes non vide.
fn common_prefix(items: &[String]) -> String {
    let first: Vec<char> = items[0].chars().collect();
    let mut len = first.len();
    for item in &items[1..] {
        let common = first
            .iter()
            .zip(item.chars())
            .take_while(|(a, b)| **a == *b)
            .count();
        len = len.min(common);
    }
    first[..len].iter().collect()
}

/// Découpe une ligne de commande `:` en (commande, arguments).
pub fn parse_command_line(line: &str) -> (String, Vec<String>) {
    let mut parts = line.split_whitespace();
    let command = parts.next().unwrap_or_default().to_string();
    let args: Vec<String> = parts.map(str::to_string).collect();
    (command, args)
}

/// Filtre les commandes dont le nom (ou un alias) commence par le premier mot
/// de `input`.
pub fn matching_commands<'a>(specs: &'a [CommandSpec], input: &str) -> Vec<&'a CommandSpec> {
    let prefix = input.split_whitespace().next().unwrap_or("");
    if prefix.is_empty() {
        return specs.iter().collect();
    }
    specs
        .iter()
        .filter(|spec| {
            spec.name.starts_with(prefix)
                || spec.aliases.iter().any(|alias| alias.starts_with(prefix))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specs() -> Vec<CommandSpec> {
        vec![
            CommandSpec::new("find", "chercher un fichier", "find <query>"),
            CommandSpec::new("grep", "chercher du texte", "grep <pattern>").with_aliases(["g"]),
            CommandSpec::new("tree", "arborescence", "tree"),
        ]
    }

    fn app() -> App {
        let ctx = Context::with_root(".");
        let exec: Executor = Box::new(|_ctx, _cmd, _args| Ok(Output::empty()));
        App::new(ctx, specs(), exec, None)
    }

    /// Workspace temporaire isolé (avec `devx.toml` pour figer la détection).
    fn temp_workspace(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("devx-tui-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("devx.toml"), "").unwrap();
        dir
    }

    fn app_at(root: &Path) -> App {
        let ctx = Context::with_root(root);
        let exec: Executor = Box::new(|_ctx, _cmd, _args| Ok(Output::empty()));
        App::new(ctx, specs(), exec, None)
    }

    // --- parsing des commandes `:` ---

    #[test]
    fn parses_simple_command() {
        assert_eq!(parse_command_line("w"), ("w".to_string(), vec![]));
    }

    #[test]
    fn parses_command_with_file_arg() {
        assert_eq!(
            parse_command_line("e src/main.rs"),
            ("e".to_string(), vec!["src/main.rs".to_string()])
        );
    }

    #[test]
    fn parses_help() {
        assert_eq!(parse_command_line("help"), ("help".to_string(), vec![]));
    }

    #[test]
    fn empty_line_yields_empty_command() {
        assert_eq!(parse_command_line("  "), (String::new(), vec![]));
    }

    // --- commandes éditeur via le mode `:` ---

    #[test]
    fn force_quit_sets_quit_flag() {
        let mut app = app();
        app.editor_mut().insert_char('x');
        app.input = "q!".to_string();
        app.submit();
        assert!(app.should_quit);
    }

    #[test]
    fn quit_blocks_on_dirty_buffer() {
        let mut app = app();
        app.editor_mut().insert_char('x');
        app.input = "q".to_string();
        app.submit();
        assert!(!app.should_quit);
        assert!(app.output.iter().any(|l| l.contains("non sauvegardées")));
    }

    #[test]
    fn write_all_saves_every_dirty_tab() {
        let root = temp_workspace("write-all");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        app.editor_mut().insert_char('X');
        open_file(&mut app, &root, "b.txt");
        app.editor_mut().insert_char('Y');
        app.input = "wa".to_string();
        app.submit();
        // Les deux onglets sont propres et le disque reflète les modifs.
        assert!(app.tabs.iter().all(|t| !t.dirty));
        assert!(fs::read_to_string(root.join("a.txt"))
            .unwrap()
            .starts_with('X'));
        assert!(fs::read_to_string(root.join("b.txt"))
            .unwrap()
            .starts_with('Y'));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn write_quit_all_saves_then_quits() {
        let root = temp_workspace("write-quit-all");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        app.editor_mut().insert_char('Z');
        app.input = "wqa".to_string();
        app.submit();
        assert!(app.should_quit);
        assert!(fs::read_to_string(root.join("a.txt"))
            .unwrap()
            .starts_with('Z'));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn quit_blocks_on_dirty_inactive_tab() {
        let root = temp_workspace("quit-dirty-inactive");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        app.editor_mut().insert_char('X'); // onglet a.txt modifié
        open_file(&mut app, &root, "b.txt"); // bascule sur un onglet propre
        assert!(!app.editor().dirty);
        app.input = "q".to_string();
        app.submit();
        assert!(
            !app.should_quit,
            ":q doit refuser tant qu'un onglet est modifié"
        );
        assert!(app.output.iter().any(|l| l.contains("a.txt")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn clean_quit_sets_quit_flag() {
        let mut app = app();
        app.input = "q".to_string();
        app.submit();
        assert!(app.should_quit);
    }

    #[test]
    fn editor_open_command_loads_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("devx-tui-open-{}.rs", std::process::id()));
        std::fs::write(&path, "fn main() {}\n").unwrap();
        let mut app = app();
        app.input = format!("e {}", path.display());
        app.submit();
        assert_eq!(app.focus, Focus::Editor);
        assert_eq!(app.editor().path.as_deref(), Some(path.as_path()));
        assert_eq!(app.editor().lines[0], "fn main() {}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn editor_write_command_saves_buffer() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("devx-tui-write-{}.txt", std::process::id()));
        std::fs::write(&path, "avant\n").unwrap();
        let mut app = app();
        app.input = format!("e {}", path.display());
        app.submit();
        app.editor_mut().lines[0].push('!');
        app.editor_mut().dirty = true;
        app.input = "w".to_string();
        app.submit();
        assert!(!app.editor().dirty);
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "avant!\n");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ctrl_n_prefills_new_command() {
        let mut app = app();
        app.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, Mode::Command);
        assert_eq!(app.input, "new ");
    }

    #[test]
    fn new_command_without_arg_shows_usage() {
        let mut app = app();
        app.input = "new".to_string();
        app.submit();
        assert!(app.output.iter().any(|l| l.contains("usage: :new")));
        assert!(app.editor().path.is_none());
    }

    #[test]
    fn new_command_opens_empty_buffer_linked_to_path() {
        let root = temp_workspace("new-buffer");
        let mut app = app_at(&root);
        app.input = "new sub/hello.rs".to_string();
        app.submit();
        let expected = root.join("sub/hello.rs");
        assert_eq!(app.focus, Focus::Editor);
        assert_eq!(app.editor().path.as_deref(), Some(expected.as_path()));
        assert!(!app.editor().dirty);
        assert!(root.join("sub").is_dir());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_command_rejects_absolute_path() {
        let root = temp_workspace("new-abs");
        let mut app = app_at(&root);
        app.input = "new /etc/passwd".to_string();
        app.submit();
        assert!(app.output.iter().any(|l| l.contains("absolu")));
        assert!(app.editor().path.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_command_rejects_parent_escape() {
        let root = temp_workspace("new-escape");
        let mut app = app_at(&root);
        app.input = "new ../evil.txt".to_string();
        app.submit();
        assert!(app.output.iter().any(|l| l.contains("invalide")));
        assert!(app.editor().path.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_then_save_creates_file_and_refreshes_tree() {
        let root = temp_workspace("new-save");
        let mut app = app_at(&root);
        app.input = "new note.txt".to_string();
        app.submit();
        assert!(!app.visible().iter().any(|v| v.name == "note.txt"));
        app.input = "w".to_string();
        app.submit();
        assert!(root.join("note.txt").is_file());
        assert!(app.visible().iter().any(|v| v.name == "note.txt"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn refresh_tree_preserves_expanded_dirs() {
        let root = temp_workspace("refresh-expand");
        fs::create_dir_all(root.join("dir")).unwrap();
        fs::write(root.join("dir/f.txt"), "x").unwrap();
        let mut app = app_at(&root);
        let idx = app
            .visible()
            .iter()
            .position(|v| v.name == "dir")
            .expect("dir présent");
        app.selection = idx;
        app.set_expanded(true);
        assert!(app.visible().iter().any(|v| v.name == "f.txt"));
        app.refresh_tree();
        assert!(app.visible().iter().any(|v| v.name == "f.txt"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn help_command_lists_editor_and_modules() {
        let mut app = app();
        app.input = "help".to_string();
        app.submit();
        let joined = app.output.join("\n");
        assert!(joined.contains(":wq"));
        assert!(joined.contains(":e <fichier>"));
        assert!(joined.contains(":new <fichier>"));
        assert!(joined.contains("grep"));
    }

    // --- matching_commands (conservé) ---

    #[test]
    fn empty_input_returns_all() {
        assert_eq!(matching_commands(&specs(), "").len(), 3);
    }

    #[test]
    fn filters_by_name_prefix() {
        let specs = specs();
        let matched = matching_commands(&specs, "gr");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "grep");
    }

    #[test]
    fn filters_by_alias_prefix() {
        let specs = specs();
        let matched = matching_commands(&specs, "g");
        let names: Vec<&str> = matched.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"grep"));
    }

    #[test]
    fn no_match_yields_empty() {
        assert!(matching_commands(&specs(), "zzz").is_empty());
    }

    #[test]
    fn context_is_used_for_root() {
        let app = app();
        assert_eq!(app.ctx.workspace.root, PathBuf::from("."));
    }

    // --- vues code/git (`:git`, `:code`, Ctrl-G) ---

    #[test]
    fn git_command_switches_to_git_view() {
        let mut app = app();
        app.input = "git".to_string();
        app.submit();
        assert_eq!(app.view, View::Git);
    }

    #[test]
    fn code_command_returns_to_code_view() {
        let mut app = app();
        app.open_git_view();
        app.input = "code".to_string();
        app.submit();
        assert_eq!(app.view, View::Code);
    }

    #[test]
    fn toggle_view_round_trips() {
        let mut app = app();
        app.toggle_view();
        assert_eq!(app.view, View::Git);
        app.toggle_view();
        assert_eq!(app.view, View::Code);
    }

    #[test]
    fn load_git_reports_error_outside_repo() {
        let dir = std::env::temp_dir().join(format!("devx-tui-nogit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let state = load_git(&dir);
        // Hors dépôt : soit une erreur explicite, soit pas de branche utile.
        assert!(state.error.is_some() || state.branch == "HEAD");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- souris ---

    #[test]
    fn editor_click_places_cursor() {
        let mut app = app();
        app.editor_mut().lines = vec!["hello".to_string(), "world".to_string()];
        app.layout.editor = Rect::new(10, 5, 40, 10);
        app.layout.editor_y_offset = 0;
        app.layout.editor_x_offset = 0;
        // origin_x = 10 + 1 + 6 = 17 ; origin_y = 5 + 1 = 6.
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 19,
            row: 7,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.focus, Focus::Editor);
        assert_eq!(app.editor().cursor_row, 1);
        assert_eq!(app.editor().cursor_col, 2);
    }

    #[test]
    fn editor_click_clamps_past_end_of_line() {
        let mut app = app();
        app.editor_mut().lines = vec!["hi".to_string()];
        app.layout.editor = Rect::new(0, 0, 40, 10);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 30,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.editor().cursor_row, 0);
        assert_eq!(app.editor().cursor_col, 2);
    }

    #[test]
    fn scroll_over_editor_moves_cursor() {
        let mut app = app();
        app.editor_mut().lines = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        app.layout.editor = Rect::new(0, 0, 40, 10);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.editor().cursor_row, 1);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.editor().cursor_row, 0);
    }

    #[test]
    fn mouse_ignored_in_command_mode() {
        let mut app = app();
        app.mode = Mode::Command;
        app.editor_mut().lines = vec!["hello".to_string()];
        app.layout.editor = Rect::new(0, 0, 40, 10);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 8,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.focus, Focus::Tree);
    }

    // --- tabulations (indentation) ---

    #[test]
    fn tab_key_inserts_indentation_when_editor_focused() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines[0].push_str("fn main() {}");
        app.editor_mut().cursor_col = 12;
        app.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            app.editor().lines[0],
            "fn main() {}    ",
            "la touche Tab doit indenter"
        );
        assert_eq!(app.editor().cursor_col, 16);
        // Le focus n'a pas changé.
        assert_eq!(app.focus, Focus::Editor);
    }

    #[test]
    fn ctrl_z_undoes_and_ctrl_y_redoes_in_editor() {
        let mut app = app();
        app.focus = Focus::Editor;
        for c in "hey".chars() {
            app.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.editor().lines[0], "hey");
        app.on_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "");
        app.on_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "hey");
    }

    #[test]
    fn ctrl_z_is_inert_when_tree_focused() {
        let mut app = app();
        app.focus = Focus::Tree;
        app.editor_mut().lines[0].push_str("intact");
        app.on_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        // Hors focus éditeur, Ctrl-Z ne touche ni le tampon ni le statut undo.
        assert_eq!(app.editor().lines[0], "intact");
        assert_ne!(app.status, "annulé");
    }

    // --- recherche ---

    fn type_in_search(app: &mut App, text: &str) {
        for c in text.chars() {
            app.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }

    #[test]
    fn ctrl_f_searches_incrementally_and_jumps() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["alpha".to_string(), "beta gamma".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, Mode::Search);
        type_in_search(&mut app, "gamma");
        // Le curseur a sauté sur la 1re occurrence.
        assert_eq!((app.editor().cursor_row, app.editor().cursor_col), (1, 5));
    }

    #[test]
    fn search_enter_jumps_to_next_occurrence() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["x x x".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        type_in_search(&mut app, "x");
        assert_eq!(app.editor().cursor_col, 0);
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.editor().cursor_col, 2);
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.editor().cursor_col, 4);
        // Boucle au premier.
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.editor().cursor_col, 0);
    }

    #[test]
    fn search_escape_restores_origin_cursor() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["abc def".to_string()];
        app.editor_mut().cursor_col = 2; // origine
        app.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        type_in_search(&mut app, "def");
        assert_eq!(app.editor().cursor_col, 4); // a sauté sur "def"
        app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.editor().cursor_col, 2); // curseur restauré
    }

    #[test]
    fn search_absent_pattern_reports_not_found() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
        type_in_search(&mut app, "zzz");
        assert!(app.status.contains("introuvable"));
    }

    // --- sélection ---

    #[test]
    fn shift_right_extends_selection() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(app.editor().selection_range(), Some(((0, 0), (0, 2))));
        // Un déplacement sans Maj annule la sélection.
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert!(app.editor().selection_range().is_none());
    }

    #[test]
    fn typing_replaces_the_selection() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        // Sélectionne "he".
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        app.on_key(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE));
        assert_eq!(app.editor().lines[0], "Xllo");
        assert!(app.editor().selection_range().is_none());
    }

    #[test]
    fn copy_then_paste_duplicates_selection() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        // Sélectionne "hel".
        for _ in 0..3 {
            app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        }
        app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(
            !app.should_quit,
            "Ctrl-C avec sélection copie, ne quitte pas"
        );
        assert_eq!(app.editor().cursor_col, 3);
        // Colle à la position courante (après "hel").
        app.on_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "helhello");
    }

    #[test]
    fn cut_removes_selection_into_clipboard() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        for _ in 0..2 {
            app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        }
        app.on_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "llo");
        app.on_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        app.on_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "llohe");
    }

    #[test]
    fn ctrl_c_without_selection_quits() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_clears_selection_before_leaving_editor() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["hello".to_string()];
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        // 1er Échap : annule la sélection, garde le focus éditeur.
        assert!(app.editor().selection_range().is_none());
        assert_eq!(app.focus, Focus::Editor);
        // 2e Échap : rend le focus à l'arbre.
        app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.focus, Focus::Tree);
    }

    // --- opérations par mot ---

    #[test]
    fn ctrl_w_deletes_previous_word_in_editor() {
        let mut app = app();
        app.focus = Focus::Editor;
        for c in "foo bar".chars() {
            app.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.on_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        assert_eq!(app.editor().lines[0], "foo ");
    }

    #[test]
    fn alt_left_moves_by_word_in_editor() {
        let mut app = app();
        app.focus = Focus::Editor;
        for c in "foo bar".chars() {
            app.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT));
        assert_eq!(app.editor().cursor_col, 4); // début de "bar"
    }

    #[test]
    fn alt_down_moves_line_in_editor() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines = vec!["un".to_string(), "deux".to_string()];
        app.editor_mut().cursor_row = 0;
        app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::ALT));
        assert_eq!(
            app.editor().lines,
            vec!["deux".to_string(), "un".to_string()]
        );
        assert_eq!(app.editor().cursor_row, 1);
    }

    // --- complétion de chemin ---

    #[test]
    fn complete_path_completes_unique_match() {
        let root = temp_workspace("complete-unique");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "").unwrap();
        // "src/ma" → "src/main.rs"
        assert_eq!(
            complete_path(&root, "src/ma").as_deref(),
            Some("src/main.rs")
        );
        // Un répertoire est suffixé d'un '/'.
        assert_eq!(complete_path(&root, "sr").as_deref(), Some("src/"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn complete_path_uses_common_prefix_for_many() {
        let root = temp_workspace("complete-prefix");
        fs::write(root.join("alpha.rs"), "").unwrap();
        fs::write(root.join("alpine.rs"), "").unwrap();
        // Préfixe commun "alp" → "alp" (progresse depuis "al").
        assert_eq!(complete_path(&root, "al").as_deref(), Some("alp"));
        // Aucune progression possible : None.
        assert_eq!(complete_path(&root, "alp"), None);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn tab_completes_path_argument_for_e_command() {
        let root = temp_workspace("complete-tab");
        fs::write(root.join("readme.md"), "").unwrap();
        let mut app = app_at(&root);
        app.mode = Mode::Command;
        app.input = "e read".to_string();
        app.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.input, "e readme.md");
        let _ = fs::remove_dir_all(&root);
    }

    // --- aller à la ligne ---

    #[test]
    fn numeric_command_jumps_to_line() {
        let mut app = app();
        app.editor_mut().lines = (1..=10).map(|i| format!("l{i}")).collect();
        app.input = "3".to_string();
        app.submit();
        assert_eq!(app.editor().cursor_row, 2); // ligne 3 → index 2
        assert_eq!(app.editor().cursor_col, 0);
        assert_eq!(app.focus, Focus::Editor);
    }

    #[test]
    fn goto_clamps_out_of_range_line() {
        let mut app = app();
        app.editor_mut().lines = vec!["a".to_string(), "b".to_string()];
        app.input = "999".to_string();
        app.submit();
        assert_eq!(app.editor().cursor_row, 1); // borné à la dernière ligne
    }

    #[test]
    fn goto_command_from_git_view_returns_to_code() {
        let mut app = app();
        app.editor_mut().lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        app.view = View::Git;
        app.input = "goto 2".to_string();
        app.submit();
        assert_eq!(app.view, View::Code);
        assert_eq!(app.editor().cursor_row, 1);
    }

    #[test]
    fn shift_tab_dedents_when_editor_focused() {
        let mut app = app();
        app.focus = Focus::Editor;
        app.editor_mut().lines[0].push_str("    code");
        app.editor_mut().cursor_col = 4;
        app.on_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(app.editor().lines[0], "code");
        assert_eq!(app.editor().cursor_col, 0);
    }

    #[test]
    fn tab_key_still_toggles_focus_from_tree() {
        let mut app = app();
        app.focus = Focus::Tree;
        app.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, Focus::Editor);
    }

    // --- onglets ---

    fn open_file(app: &mut App, root: &Path, name: &str) {
        app.input = format!("e {}", root.join(name).display());
        app.submit();
    }

    #[test]
    fn opening_two_files_creates_two_tabs() {
        let root = temp_workspace("tabs-two");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        open_file(&mut app, &root, "b.txt");
        // Tampon initial vide + deux onglets.
        assert_eq!(app.tab_count(), 3);
        assert_eq!(app.active, 2);
        assert_eq!(app.editor().lines[0], "B");
        // L'onglet précédent a gardé son contenu.
        assert_eq!(app.tabs[1].lines[0], "A");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reopening_same_file_activates_existing_tab() {
        let root = temp_workspace("tabs-reopen");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        let ed = app.editor_mut();
        ed.move_end();
        ed.insert_char('!');
        open_file(&mut app, &root, "a.txt");
        // Aucun onglet supplémentaire : l'existant est réactivé, ses
        // modifications en attente préservées.
        assert_eq!(app.tab_count(), 2);
        assert_eq!(app.active, 1);
        assert!(app.editor().dirty);
        assert!(app.editor().lines[0].starts_with('A'));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn switch_tab_cycles_and_keeps_buffers() {
        let root = temp_workspace("tabs-cycle");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        open_file(&mut app, &root, "b.txt");
        switch_tab_prev(&mut app);
        assert_eq!(app.active, 1);
        assert_eq!(app.editor().lines[0], "A");
        let ed = app.editor_mut();
        ed.move_end();
        ed.insert_char('X');
        switch_tab_next(&mut app);
        assert_eq!(app.editor().lines[0], "B");
        // L'édition non sauvée de l'onglet A n'a pas été perdue.
        assert_eq!(app.tabs[1].lines[0], "AX");
        switch_tab_next(&mut app);
        // Bouclage : dernier onglet → tampon initial vide.
        assert_eq!(app.active, 0);
        assert!(app.editor().path.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    fn switch_tab_prev(app: &mut App) {
        app.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
    }

    fn switch_tab_next(app: &mut App) {
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
    }

    #[test]
    fn ctrl_arrows_cycle_tabs() {
        let root = temp_workspace("tabs-ctrl");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        open_file(&mut app, &root, "b.txt");
        app.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(app.active, 1);
        app.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(app.active, 2);
        let _ = fs::remove_dir_all(&root);
    }
    #[test]
    fn switch_tab_is_noop_with_single_tab() {
        let mut app = app();
        app.switch_tab(1);
        assert_eq!(app.active, 0);
        app.switch_tab(-1);
        assert_eq!(app.active, 0);
    }

    #[test]
    fn close_command_refuses_dirty_tab() {
        let mut app = app();
        app.editor_mut().insert_char('x');
        app.input = "close".to_string();
        app.submit();
        assert_eq!(app.tab_count(), 1);
        assert!(app.output.iter().any(|l| l.contains("onglet modifié")));
    }

    #[test]
    fn close_command_removes_tab() {
        let root = temp_workspace("tabs-close");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        open_file(&mut app, &root, "b.txt");
        app.input = "close".to_string();
        app.submit();
        assert_eq!(app.tab_count(), 2); // vide + a.txt
        assert_eq!(app.editor().lines[0], "A");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn close_last_tab_leaves_empty_buffer() {
        let mut app = app();
        app.input = "close".to_string();
        app.submit();
        assert_eq!(app.tab_count(), 1);
        assert!(app.editor().path.is_none());
        assert!(!app.editor().dirty);
    }

    #[test]
    fn click_on_tab_bar_switches_tab() {
        let mut app = app();
        // On simule la géométrie remplie par le rendu : deux onglets.
        app.layout.tabs = Rect::new(0, 0, 20, 1);
        app.layout.tab_spans = vec![Rect::new(0, 0, 4, 1), Rect::new(4, 0, 4, 1)];
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.active, 1);
    }

    #[test]
    fn next_and_prev_commands_cycle_tabs() {
        let root = temp_workspace("tabs-cmd");
        fs::write(root.join("a.txt"), "A\n").unwrap();
        fs::write(root.join("b.txt"), "B\n").unwrap();
        let mut app = app_at(&root);
        open_file(&mut app, &root, "a.txt");
        open_file(&mut app, &root, "b.txt");
        app.input = "prev".to_string();
        app.submit();
        assert_eq!(app.active, 1);
        app.input = "next".to_string();
        app.submit();
        assert_eq!(app.active, 2);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn help_lists_keyboard_shortcuts() {
        let mut app = app();
        app.input = "help".to_string();
        app.submit();
        let joined = app.output.join("\n");
        assert!(joined.contains("[raccourcis]"));
        assert!(joined.contains("Ctrl-Z"));
        assert!(joined.contains("Ctrl-F"));
        assert!(joined.contains("Alt-↑/↓"));
    }

    #[test]
    fn help_lists_tab_commands() {
        let mut app = app();
        app.input = "help".to_string();
        app.submit();
        let joined = app.output.join("\n");
        assert!(joined.contains(":next"));
        assert!(joined.contains(":prev"));
        assert!(joined.contains(":close"));
    }
}
