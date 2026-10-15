//! État applicatif de la TUI : arbre de fichiers, éditeur, sortie, événements.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use devx_core::{CommandSpec, Context, DevxError, Output};
use ratatui::layout::Rect;

use crate::editor::Buffer;

/// Largeur (en cellules) de la gouttière de numéros de ligne dans l'éditeur.
pub const EDITOR_GUTTER: usize = 6;

/// Callback d'exécution de commande fourni par l'hôte (le binaire `devx`).
///
/// La TUI reste découplée des modules : elle délègue l'exécution.
pub type Executor = Box<dyn FnMut(&Context, &str, &[String]) -> Result<Output, DevxError>>;

/// Profondeur maximale de scan de l'arbre de fichiers.
const MAX_DEPTH: usize = 6;

/// Nombre maximal d'entrées scannées (garde-fou contre les arbres géants).
const MAX_ENTRIES: usize = 20_000;

/// Répertoires ignorés lors du scan.
const IGNORED_DIRS: [&str; 3] = [".git", "target", "node_modules"];

/// Commandes éditeur disponibles dans le mode `:` (nom, description).
const EDITOR_COMMANDS: [(&str, &str); 9] = [
    (":help", "liste toutes les commandes disponibles"),
    (":w", "sauvegarde le fichier courant"),
    (":wq", "sauvegarde puis quitte"),
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
    (":git", "affiche un rapport git à la place du code"),
    (":code", "revient à la vue code (arbre + éditeur)"),
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
}

/// Vue principale affichée par l'IDE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Vue « code » : arbre de fichiers + éditeur (vue par défaut).
    Code,
    /// Vue « git » : branche, statut et derniers commits.
    Git,
}

/// Informations git affichées dans la vue « git ».
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitState {
    /// Branche courante (ou « HEAD » si détaché).
    pub branch: String,
    /// Lignes de `git status --short --branch`.
    pub status: Vec<String>,
    /// Lignes de `git log --oneline`.
    pub log: Vec<String>,
    /// Message d'erreur éventuel (hors dépôt git, git absent…).
    pub error: Option<String>,
}

/// Nœud de l'arbre de fichiers du workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Nom affiché.
    pub name: String,
    /// Chemin absolu.
    pub path: PathBuf,
    /// Vrai s'il s'agit d'un répertoire.
    pub is_dir: bool,
    /// Répertoire déplié.
    pub expanded: bool,
    /// Enfants (vide pour un fichier).
    pub children: Vec<Node>,
}

/// Ligne aplatie prête au rendu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visible {
    /// Nom affiché.
    pub name: String,
    /// Chemin associé.
    pub path: PathBuf,
    /// Vrai s'il s'agit d'un répertoire.
    pub is_dir: bool,
    /// Profondeur d'indentation.
    pub depth: usize,
    /// Répertoire déplié.
    pub expanded: bool,
    /// Le répertoire contient au moins un enfant.
    pub has_children: bool,
}

/// Géométrie du dernier rendu, utilisée pour la souris (carte de zones).
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutInfo {
    /// Zone de l'arbre de fichiers.
    pub tree: Rect,
    /// Défilement vertical appliqué à l'arbre.
    pub tree_offset: usize,
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
    /// Tampon de l'éditeur.
    pub editor: Buffer,
    /// Lignes de la zone de sortie.
    pub output: Vec<String>,
    /// Contenu de la barre de commande.
    pub input: String,
    /// Index sélectionné dans la palette.
    pub palette_selected: usize,
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
            "Tab: changer de panneau · : commande · Ctrl-P: palette · q: quitter".to_string(),
        ];

        let editor = match initial_file {
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
        let focus = if editor.path.is_some() {
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
            editor,
            output,
            input: String::new(),
            palette_selected: 0,
            status: "prêt".to_string(),
            view: View::Code,
            git: GitState::default(),
            layout: LayoutInfo::default(),
            executor,
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
                    self.should_quit = true;
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
                _ => {}
            }
        }

        match self.mode {
            Mode::Command => self.on_key_command(key),
            Mode::Palette => self.on_key_palette(key),
            Mode::Normal => match key.code {
                KeyCode::Char(':') | KeyCode::Char('/') => {
                    self.mode = Mode::Command;
                    self.input.clear();
                    self.status = "commande (:) — Échap pour annuler".to_string();
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
        if inside(self.layout.tree, x, y) {
            self.on_tree_click(x, y);
        } else if inside(self.layout.editor, x, y) {
            self.on_editor_click(x, y);
        }
    }

    pub fn on_scroll(&mut self, x: u16, y: u16, delta: i32) {
        if inside(self.layout.editor, x, y) {
            for _ in 0..delta.abs() {
                if delta < 0 {
                    self.editor.move_up();
                } else {
                    self.editor.move_down();
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
        let last_row = self.editor.lines.len().saturating_sub(1);
        let row = ((rel_y as usize) + self.layout.editor_y_offset).min(last_row);
        let col_vis = if rel_x < 0 { 0 } else { rel_x as usize };
        let line_len = self.editor.lines[row].chars().count();
        let col = (col_vis + self.layout.editor_x_offset).min(line_len);
        self.editor.cursor_row = row;
        self.editor.cursor_col = col;
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

    fn on_key_editor(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Home => self.editor.move_home(),
            KeyCode::End => self.editor.move_end(),
            KeyCode::Backspace => self.editor.backspace(),
            KeyCode::Delete => self.editor.delete(),
            KeyCode::Enter => self.editor.insert_newline(),
            KeyCode::Esc => {
                self.focus = Focus::Tree;
                self.status = "focus: arbre de fichiers".to_string();
            }
            KeyCode::Char(c) => self.editor.insert_char(c),
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
                self.editor = buffer;
                self.focus = Focus::Editor;
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
        match command {
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
                if !self.editor.dirty {
                    self.should_quit = true;
                }
                true
            }
            "q" => {
                if self.editor.dirty {
                    let msg = "modifications non sauvegardées — :w puis :q, ou :q!".to_string();
                    self.output.push(msg.clone());
                    self.status = msg;
                } else {
                    self.should_quit = true;
                }
                true
            }
            "q!" => {
                self.should_quit = true;
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
                                self.editor = buffer;
                                self.focus = Focus::Editor;
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
                self.editor = Buffer::empty_with_path(&path);
                self.focus = Focus::Editor;
                self.refresh_tree();
                self.status = format!("nouveau fichier {} — :w pour sauvegarder", path.display());
                true
            }
            _ => false,
        }
    }

    fn save_editor(&mut self) {
        match self.editor.save() {
            Ok(()) => {
                let path = self
                    .editor
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

/// Lance `git <args>` dans `root` et renvoie la sortie standard (texte).
fn git_capture(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git introuvable : {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        return Err(if detail.is_empty() {
            format!("`git {}` a échoué", args.join(" "))
        } else {
            detail.to_string()
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Construit l'état git (branche, statut, derniers commits) pour `root`.
pub fn load_git(root: &Path) -> GitState {
    let mut state = GitState {
        branch: "HEAD".to_string(),
        ..GitState::default()
    };

    let status = match git_capture(root, &["status", "--short", "--branch"]) {
        Ok(text) => text,
        Err(err) => {
            state.error = Some(err);
            return state;
        }
    };

    let mut lines = status.lines();
    if let Some(first) = lines.next() {
        let head = first.trim_start_matches("## ").trim();
        let name = head.split("...").next().unwrap_or(head);
        let name = name.split_whitespace().next().unwrap_or("HEAD");
        state.branch = if name.is_empty() {
            "HEAD".to_string()
        } else {
            name.to_string()
        };
    }
    state.status = lines.map(|l| l.to_string()).collect();

    state.log = git_capture(root, &["log", "--oneline", "-n", "20"])
        .map(|text| text.lines().map(str::to_string).collect())
        .unwrap_or_default();

    state
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

fn flatten(nodes: &[Node], level: usize, out: &mut Vec<Visible>) {
    for node in nodes {
        out.push(Visible {
            name: node.name.clone(),
            path: node.path.clone(),
            is_dir: node.is_dir,
            depth: level,
            expanded: node.expanded,
            has_children: !node.children.is_empty(),
        });
        if node.is_dir && node.expanded {
            flatten(&node.children, level + 1, out);
        }
    }
}

fn set_expanded_by_path(nodes: &mut [Node], path: &Path, expand: bool) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            if node.is_dir {
                node.expanded = expand;
            }
            return true;
        }
        if set_expanded_by_path(&mut node.children, path, expand) {
            return true;
        }
    }
    false
}

fn scan_dir(dir: &Path, depth: usize, budget: &mut usize) -> Vec<Node> {
    if depth == 0 || *budget == 0 {
        return Vec::new();
    }
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut entries: Vec<(bool, String, PathBuf)> = read
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_dir && IGNORED_DIRS.contains(&name.as_str()) {
                return None;
            }
            Some((is_dir, name, path))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut nodes = Vec::new();
    for (is_dir, name, path) in entries {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let children = if is_dir {
            scan_dir(&path, depth - 1, budget)
        } else {
            Vec::new()
        };
        nodes.push(Node {
            name,
            path,
            is_dir,
            expanded: false,
            children,
        });
    }
    nodes
}

/// Collecte les chemins des répertoires dépliés (récursif).
fn expanded_paths(node: &Node) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if node.is_dir && node.expanded {
        out.push(node.path.clone());
    }
    for child in &node.children {
        out.extend(expanded_paths(child));
    }
    out
}

/// Ré-applique l'état déplié à partir d'un jeu de chemins.
fn restore_expanded(nodes: &mut [Node], expanded: &HashSet<PathBuf>) {
    for node in nodes {
        if node.is_dir && expanded.contains(&node.path) {
            node.expanded = true;
        }
        restore_expanded(&mut node.children, expanded);
    }
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
        app.editor.insert_char('x');
        app.input = "q!".to_string();
        app.submit();
        assert!(app.should_quit);
    }

    #[test]
    fn quit_blocks_on_dirty_buffer() {
        let mut app = app();
        app.editor.insert_char('x');
        app.input = "q".to_string();
        app.submit();
        assert!(!app.should_quit);
        assert!(app.output.iter().any(|l| l.contains("non sauvegardées")));
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
        assert_eq!(app.editor.path.as_deref(), Some(path.as_path()));
        assert_eq!(app.editor.lines[0], "fn main() {}");
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
        app.editor.lines[0].push('!');
        app.editor.dirty = true;
        app.input = "w".to_string();
        app.submit();
        assert!(!app.editor.dirty);
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
        assert!(app.editor.path.is_none());
    }

    #[test]
    fn new_command_opens_empty_buffer_linked_to_path() {
        let root = temp_workspace("new-buffer");
        let mut app = app_at(&root);
        app.input = "new sub/hello.rs".to_string();
        app.submit();
        let expected = root.join("sub/hello.rs");
        assert_eq!(app.focus, Focus::Editor);
        assert_eq!(app.editor.path.as_deref(), Some(expected.as_path()));
        assert!(!app.editor.dirty);
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
        assert!(app.editor.path.is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_command_rejects_parent_escape() {
        let root = temp_workspace("new-escape");
        let mut app = app_at(&root);
        app.input = "new ../evil.txt".to_string();
        app.submit();
        assert!(app.output.iter().any(|l| l.contains("invalide")));
        assert!(app.editor.path.is_none());
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
        app.editor.lines = vec!["hello".to_string(), "world".to_string()];
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
        assert_eq!(app.editor.cursor_row, 1);
        assert_eq!(app.editor.cursor_col, 2);
    }

    #[test]
    fn editor_click_clamps_past_end_of_line() {
        let mut app = app();
        app.editor.lines = vec!["hi".to_string()];
        app.layout.editor = Rect::new(0, 0, 40, 10);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 30,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.editor.cursor_row, 0);
        assert_eq!(app.editor.cursor_col, 2);
    }

    #[test]
    fn scroll_over_editor_moves_cursor() {
        let mut app = app();
        app.editor.lines = vec![
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
        assert_eq!(app.editor.cursor_row, 1);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.editor.cursor_row, 0);
    }

    #[test]
    fn mouse_ignored_in_command_mode() {
        let mut app = app();
        app.mode = Mode::Command;
        app.editor.lines = vec!["hello".to_string()];
        app.layout.editor = Rect::new(0, 0, 40, 10);
        app.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 8,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.focus, Focus::Tree);
    }
}
