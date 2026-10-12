//! Contrat figé du projet `devx`.
//!
//! Ce crate expose les types partagés ([`Workspace`], [`Config`], [`Context`],
//! [`CommandSpec`], [`Output`], [`DevxError`]) et le trait [`Module`] que chaque
//! module de commandes doit implémenter.
//!
//! ATTENTION : ce fichier est le contrat commun aux agents. Sa surface publique
//! est gelée. Les modules ne doivent l'utiliser que pour ces types, jamais le
//! modifier.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Racine de travail sur laquelle s'appliquent les opérations des modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// Répertoire racine du workspace.
    pub root: PathBuf,
}

impl Workspace {
    /// Construit un workspace à partir d'une racine.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Détecte la racine à partir d'un point de départ en remontant les parents
    /// jusqu'à trouver un fichier `.git` ou `devx.toml`. À défaut, renvoie
    /// `start` (canonisé si possible).
    pub fn discover(start: impl AsRef<Path>) -> Self {
        let start = start.as_ref();
        let mut current = Some(start);
        while let Some(dir) = current {
            if dir.join(".git").exists() || dir.join("devx.toml").is_file() {
                return Self::new(dir);
            }
            current = dir.parent();
        }
        Self::new(start.to_path_buf())
    }
}

/// Configuration chargée depuis `devx.toml` (aucun fichier requis).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    /// Thème d'affichage de la TUI.
    pub theme: String,
    /// Commande d'ouverture d'un fichier dans l'éditeur (ex: `code`).
    pub editor_cmd: String,
    /// Tâches nommées disponibles via la commande `run`.
    pub tasks: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            editor_cmd: "code".to_string(),
            tasks: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Charge `devx.toml` depuis la racine du workspace, ou renvoie les valeurs
    /// par défaut si le fichier est absent ou invalide.
    pub fn load(root: impl AsRef<Path>) -> Self {
        let path = root.as_ref().join("devx.toml");
        match std::fs::read_to_string(&path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

/// Contexte d'exécution transmis à chaque commande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    /// Workspace courant.
    pub workspace: Workspace,
    /// Configuration chargée.
    pub config: Config,
}

impl Context {
    /// Construit un contexte à partir d'un workspace et d'une configuration.
    pub fn new(workspace: Workspace, config: Config) -> Self {
        Self { workspace, config }
    }

    /// Construit un contexte à partir d'une racine : détecte le workspace et
    /// charge sa configuration.
    pub fn with_root(root: impl AsRef<Path>) -> Self {
        let workspace = Workspace::discover(root);
        let config = Config::load(&workspace.root);
        Self { workspace, config }
    }
}

/// Description déclarative d'une commande exposée par un module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    /// Nom de la commande (ex: `grep`).
    pub name: String,
    /// Description courte affichée dans la palette.
    pub description: String,
    /// Ligne d'usage (ex: `grep <pattern>`).
    pub usage: String,
    /// Alias acceptés (ex: `["f"]`).
    pub aliases: Vec<String>,
}

impl CommandSpec {
    /// Crée une nouvelle spécification de commande sans alias.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: usage.into(),
            aliases: Vec::new(),
        }
    }

    /// Ajoute/remplace les alias de la commande.
    pub fn with_aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }
}

/// Résultat textuel d'une commande.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    /// Lignes de sortie, dans l'ordre d'affichage.
    pub lines: Vec<String>,
}

impl Output {
    /// Crée une sortie à partir d'une liste de lignes.
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Crée une sortie vide.
    pub fn empty() -> Self {
        Self { lines: Vec::new() }
    }
}

/// Erreur remontée par l'exécution d'une commande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevxError {
    /// Erreur d'entrée/sortie.
    Io(String),
    /// Erreur générique portant un message.
    Message(String),
    /// Arguments invalides ou manquants.
    InvalidArgs(String),
    /// Commande inconnue du module.
    UnknownCommand(String),
    /// Erreur fourre-tout (ex: erreurs de rendu TUI).
    Other(String),
}

impl DevxError {
    /// Construit une erreur `Io`.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    /// Construit une erreur `Message`.
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    /// Construit une erreur `InvalidArgs`.
    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self::InvalidArgs(message.into())
    }

    /// Construit une erreur `UnknownCommand`.
    pub fn unknown_command(command: impl Into<String>) -> Self {
        Self::UnknownCommand(command.into())
    }

    /// Construit une erreur `Other`.
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

impl fmt::Display for DevxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DevxError::Io(msg) => write!(f, "io error: {msg}"),
            DevxError::Message(msg) => write!(f, "{msg}"),
            DevxError::InvalidArgs(msg) => write!(f, "invalid arguments: {msg}"),
            DevxError::UnknownCommand(cmd) => write!(f, "unknown command: {cmd}"),
            DevxError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DevxError {}

impl From<std::io::Error> for DevxError {
    fn from(err: std::io::Error) -> Self {
        DevxError::Io(err.to_string())
    }
}

/// Contrat implémenté par chaque module de commandes.
pub trait Module: Send + Sync {
    /// Nom du module (ex: `files`).
    fn name(&self) -> &'static str;

    /// Commandes exposées par le module.
    fn commands(&self) -> Vec<CommandSpec>;

    /// Exécute une commande avec ses arguments dans un contexte donné.
    fn execute(&self, ctx: &Context, command: &str, args: &[String]) -> Result<Output, DevxError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_without_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(dir.path());
        assert_eq!(config.theme, "dark");
        assert!(config.tasks.is_empty());
    }

    #[test]
    fn config_reads_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("devx.toml"),
            "theme = \"light\"\n[tasks]\nbuild = \"cargo build\"\n",
        )
        .unwrap();
        let config = Config::load(dir.path());
        assert_eq!(config.theme, "light");
        assert_eq!(
            config.tasks.get("build").map(String::as_str),
            Some("cargo build")
        );
    }

    #[test]
    fn workspace_discover_finds_marker() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("devx.toml"), "").unwrap();
        let nested = dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        let ws = Workspace::discover(&nested);
        assert_eq!(ws.root, dir.path());
    }
}
