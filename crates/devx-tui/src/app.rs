//! État applicatif de la TUI : arbre de fichiers, sortie, saisie, événements.

use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use devx_core::{CommandSpec, Context, DevxError, Output};

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

/// Mode d'interaction courant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Navigation / raccourcis.
    Normal,
    /// Saisie dans la barre de commande.
    Command,
    /// Palette de commandes (`Ctrl-P`).
    Palette,
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

/// État complet de l'application TUI.
pub struct App {
    /// Contexte d'exécution (workspace + config).
    pub ctx: Context,
    /// Commandes disponibles (tous modules confondus).
    pub specs: Vec<CommandSpec>,
    /// Mode courant.
    pub mode: Mode,
    /// Demande de sortie de la boucle.
    pub should_quit: bool,
    /// Arbre de fichiers du workspace.
    pub nodes: Vec<Node>,
    /// Index de la ligne sélectionnée.
    pub selection: usize,
    /// Lignes de la zone de sortie.
    pub output: Vec<String>,
    /// Contenu de la barre de commande.
    pub input: String,
    /// Index sélectionné dans la palette.
    pub palette_selected: usize,
    /// Message de statut.
    pub status: String,
    executor: Executor,
}

impl App {
    /// Crée une application à partir du contexte, des specs et de l'exécuteur.
    pub fn new(ctx: Context, specs: Vec<CommandSpec>, executor: Executor) -> Self {
        let mut budget = MAX_ENTRIES;
        let nodes = scan_dir(&ctx.workspace.root, MAX_DEPTH, &mut budget);
        let output = vec![
            "devx — VS Code dans le terminal".to_string(),
            "Ctrl-P : palette · : commande · q / Ctrl-C : quitter".to_string(),
        ];
        Self {
            ctx,
            specs,
            mode: Mode::Normal,
            should_quit: false,
            nodes,
            selection: 0,
            output,
            input: String::new(),
            palette_selected: 0,
            status: "prêt".to_string(),
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
                _ => {}
            }
        }

        match self.mode {
            Mode::Normal => self.on_key_normal(key),
            Mode::Command => self.on_key_command(key),
            Mode::Palette => self.on_key_palette(key),
        }
    }

    fn on_key_normal(&mut self, key: KeyEvent) {
        let len = self.visible().len();
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(':') | KeyCode::Char('/') => {
                self.mode = Mode::Command;
                self.status = "saisie de commande".to_string();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selection = self.selection.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.selection + 1 < len {
                    self.selection += 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => self.set_expanded(true),
            KeyCode::Left | KeyCode::Char('h') => self.set_expanded(false),
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
                self.status = "saisie de commande".to_string();
            }
            _ => {}
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

    fn submit(&mut self) {
        let line = self.input.trim().to_string();
        self.input.clear();
        self.mode = Mode::Normal;
        self.status = "prêt".to_string();
        if line.is_empty() {
            return;
        }

        let mut parts = line.split_whitespace();
        let command = parts.next().unwrap_or_default().to_string();
        let args: Vec<String> = parts.map(str::to_string).collect();
        self.output.push(format!("devx> {line}"));

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
    use devx_core::{Config, Workspace};

    fn specs() -> Vec<CommandSpec> {
        vec![
            CommandSpec::new("find", "chercher un fichier", "find <query>"),
            CommandSpec::new("grep", "chercher du texte", "grep <pattern>").with_aliases(["g"]),
            CommandSpec::new("tree", "arborescence", "tree"),
        ]
    }

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
        let ctx = Context::new(Workspace::new("."), Config::default());
        let exec: Executor = Box::new(|_ctx, _cmd, _args| Ok(Output::empty()));
        let app = App::new(ctx, specs(), exec);
        assert_eq!(app.ctx.workspace.root, PathBuf::from("."));
    }
}
