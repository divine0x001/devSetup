//! État applicatif de la TUI : arbre de fichiers, éditeur, sortie, événements.

use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use devx_core::{CommandSpec, Context, DevxError, Output};

use crate::editor::Buffer;

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
const EDITOR_COMMANDS: [(&str, &str); 6] = [
    (":help", "liste toutes les commandes disponibles"),
    (":w", "sauvegarde le fichier courant"),
    (":wq", "sauvegarde puis quitte"),
    (":q", "quitte (refusé si modifications en attente)"),
    (":q!", "quitte sans sauvegarder"),
    (
        ":e <fichier>",
        "ouvre un fichier et passe le focus à l'éditeur",
    ),
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
            }
            Err(err) => {
                self.output.push(format!("error: {err}"));
                self.status = "erreur".to_string();
            }
        }
    }
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
    fn help_command_lists_editor_and_modules() {
        let mut app = app();
        app.input = "help".to_string();
        app.submit();
        let joined = app.output.join("\n");
        assert!(joined.contains(":wq"));
        assert!(joined.contains(":e <fichier>"));
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
}
