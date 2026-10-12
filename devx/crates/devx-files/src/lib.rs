//! Module `files` du projet `devx`.
//!
//! Expose [`FilesModule`], qui implémente [`devx_core::Module`] et fournit les
//! commandes orientées fichiers :
//!
//! - `tree` : arborescence de `ctx.workspace.root` (`.git`/`target` exclus,
//!   `.gitignore` respecté) ;
//! - `find <query>` : fuzzy finder par sous-séquence, top 20 par score ;
//! - `grep <pattern>` : recherche récursive, sortie `chemin:ligne:texte` ;
//! - `preview <file> [n]` : `n` premières lignes numérotées (défaut 40) ;
//! - `new <file>` : crée un fichier vide (dossiers parents créés au besoin) ;
//! - `write <file> <content...>` : écrit/écrase un fichier ;
//! - `rm <file>` : supprime un fichier ;
//! - `mkdir <dir>` : crée un dossier (récursif) ;
//! - `ls [dir]` : liste un dossier (dossier courant : racine du workspace).
//!
//! Tous les chemins sont résolus relativement à `ctx.workspace.root` et refusés
//! s'ils tentent de sortir de la racine.

mod fuzzy;

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

use devx_core::{CommandSpec, Context, DevxError, Module, Output};
use ignore::WalkBuilder;
use regex::Regex;

/// Nombre maximal de résultats renvoyés par `find`.
pub const FIND_LIMIT: usize = 20;

/// Nombre de lignes renvoyées par `preview` par défaut.
pub const DEFAULT_PREVIEW_LINES: usize = 40;

/// Module de commandes orientées fichiers.
#[derive(Debug, Default, Clone, Copy)]
pub struct FilesModule;

impl FilesModule {
    /// Crée un nouveau module `files`.
    pub fn new() -> Self {
        Self
    }

    fn tree(&self, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        if !root.is_dir() {
            return Err(DevxError::message(format!(
                "racine introuvable ou non répertoire: {}",
                root.display()
            )));
        }

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .require_git(false)
            .filter_entry(|entry| {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                !(is_dir && is_pruned_dir(&entry.file_name().to_string_lossy()))
            })
            .build();

        let mut entries: Vec<(PathBuf, usize, bool, String)> = Vec::new();
        for entry in walker {
            let entry = entry.map_err(|err| DevxError::message(err.to_string()))?;
            if entry.depth() == 0 {
                continue;
            }
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let name = entry.file_name().to_string_lossy().into_owned();
            entries.push((entry.path().to_path_buf(), entry.depth(), is_dir, name));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut lines = vec![format!("{}/", root.display())];
        for (_, depth, is_dir, name) in entries {
            let indent = "  ".repeat(depth.saturating_sub(1));
            if is_dir {
                lines.push(format!("{indent}{name}/"));
            } else {
                lines.push(format!("{indent}{name}"));
            }
        }

        Ok(Output::new(lines))
    }

    fn find(&self, query: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let mut scored: Vec<(i64, String)> = Vec::new();

        for path in collect_files(root)? {
            let relative = relative_path(root, &path);
            if let Some(value) = fuzzy::score(query, &relative) {
                scored.push((value, relative));
            }
        }

        scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.len().cmp(&b.1.len()))
                .then_with(|| a.1.cmp(&b.1))
        });

        let lines = scored
            .into_iter()
            .take(FIND_LIMIT)
            .map(|(_, path)| path)
            .collect();
        Ok(Output::new(lines))
    }

    fn grep(&self, pattern: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let regex = Regex::new(pattern)
            .map_err(|err| DevxError::invalid_args(format!("regex invalide: {err}")))?;

        let mut lines = Vec::new();
        for path in collect_files(root)? {
            let Ok(file) = fs::File::open(&path) else {
                continue;
            };
            let relative = relative_path(root, &path);
            for (index, line) in BufReader::new(file).lines().enumerate() {
                let Ok(line) = line else { break };
                if regex.is_match(&line) {
                    lines.push(format!("{relative}:{}:{line}", index + 1));
                }
            }
        }

        Ok(Output::new(lines))
    }

    fn preview(
        &self,
        file: &str,
        limit: Option<usize>,
        ctx: &Context,
    ) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let candidate = root.join(file);
        let path = if candidate.is_file() {
            candidate
        } else {
            PathBuf::from(file)
        };

        let handle = fs::File::open(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;

        let limit = limit.unwrap_or(DEFAULT_PREVIEW_LINES);
        let mut lines = Vec::new();
        for (index, line) in BufReader::new(handle).lines().take(limit).enumerate() {
            let line = line.map_err(|err| DevxError::io(err.to_string()))?;
            lines.push(format!("{}: {line}", index + 1));
        }

        Ok(Output::new(lines))
    }

    fn new_file(&self, file: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let path = resolve_under_root(root, file)?;
        if path.exists() {
            return Err(DevxError::message(format!(
                "le fichier existe déjà: {}",
                relative_path(root, &path)
            )));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| DevxError::io(format!("{}: {err}", parent.display())))?;
        }
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        Ok(Output::new(vec![format!(
            "créé: {}",
            relative_path(root, &path)
        )]))
    }

    fn write_file(&self, file: &str, content: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let path = resolve_under_root(root, file)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| DevxError::io(format!("{}: {err}", parent.display())))?;
        }
        fs::write(&path, content)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        Ok(Output::new(vec![format!(
            "écrit: {} ({} octets)",
            relative_path(root, &path),
            content.len()
        )]))
    }

    fn remove(&self, file: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let path = resolve_under_root(root, file)?;
        if !path.exists() {
            return Err(DevxError::message(format!(
                "fichier absent: {}",
                relative_path(root, &path)
            )));
        }
        if path.is_dir() {
            return Err(DevxError::message(format!(
                "est un dossier (utiliser une commande dédiée): {}",
                relative_path(root, &path)
            )));
        }
        fs::remove_file(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        Ok(Output::new(vec![format!(
            "supprimé: {}",
            relative_path(root, &path)
        )]))
    }

    fn make_dir(&self, dir: &str, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let path = resolve_under_root(root, dir)?;
        fs::create_dir_all(&path)
            .map_err(|err| DevxError::io(format!("{}: {err}", path.display())))?;
        Ok(Output::new(vec![format!(
            "dossier créé: {}",
            relative_path(root, &path)
        )]))
    }

    fn list(&self, dir: Option<&str>, ctx: &Context) -> Result<Output, DevxError> {
        let root = &ctx.workspace.root;
        let target = match dir {
            Some(raw) => resolve_under_root(root, raw)?,
            None => root.clone(),
        };
        if !target.is_dir() {
            return Err(DevxError::message(format!(
                "dossier introuvable: {}",
                relative_path(root, &target)
            )));
        }

        let mut entries: Vec<(String, bool)> = Vec::new();
        for entry in fs::read_dir(&target)
            .map_err(|err| DevxError::io(format!("{}: {err}", target.display())))?
        {
            let entry = entry.map_err(|err| DevxError::io(err.to_string()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            entries.push((name, is_dir));
        }
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let lines = entries
            .into_iter()
            .map(|(name, is_dir)| if is_dir { format!("{name}/") } else { name })
            .collect();
        Ok(Output::new(lines))
    }
}

impl Module for FilesModule {
    fn name(&self) -> &'static str {
        "files"
    }

    fn commands(&self) -> Vec<CommandSpec> {
        vec![
            CommandSpec::new("tree", "Affiche l'arborescence du workspace", "tree")
                .with_aliases(["t"]),
            CommandSpec::new(
                "find",
                "Fuzzy finder de fichiers (top 20 par score)",
                "find <query>",
            )
            .with_aliases(["f"]),
            CommandSpec::new(
                "grep",
                "Recherche texte récursive respectant .gitignore",
                "grep <pattern>",
            )
            .with_aliases(["g"]),
            CommandSpec::new(
                "preview",
                "Affiche les N premières lignes d'un fichier",
                "preview <file> [n]",
            )
            .with_aliases(["p", "cat"]),
            CommandSpec::new(
                "new",
                "Crée un fichier vide (dossiers parents créés au besoin)",
                "new <file>",
            )
            .with_aliases(["touch"]),
            CommandSpec::new(
                "write",
                "Écrit (crée/écrase) le contenu dans un fichier",
                "write <file> <content...>",
            )
            .with_aliases(["w"]),
            CommandSpec::new("rm", "Supprime un fichier", "rm <file>")
                .with_aliases(["del", "remove"]),
            CommandSpec::new("mkdir", "Crée un dossier (récursif)", "mkdir <dir>")
                .with_aliases(["md"]),
            CommandSpec::new(
                "ls",
                "Liste les entrées d'un dossier (dossier courant: racine)",
                "ls [dir]",
            )
            .with_aliases(["list"]),
        ]
    }

    fn execute(&self, ctx: &Context, command: &str, args: &[String]) -> Result<Output, DevxError> {
        match command {
            "tree" | "t" => self.tree(ctx),
            "find" | "f" => {
                let query = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("find requiert <query>"))?;
                self.find(query, ctx)
            }
            "grep" | "g" => {
                let pattern = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("grep requiert <pattern>"))?;
                self.grep(pattern, ctx)
            }
            "preview" | "p" | "cat" => {
                let file = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("preview requiert <file>"))?;
                let limit = match args.get(1) {
                    Some(raw) => Some(raw.parse::<usize>().map_err(|_| {
                        DevxError::invalid_args(format!("nombre de lignes invalide: {raw}"))
                    })?),
                    None => None,
                };
                self.preview(file, limit, ctx)
            }
            "new" | "touch" => {
                let file = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("new requiert <file>"))?;
                self.new_file(file, ctx)
            }
            "write" | "w" => {
                let file = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("write requiert <file> <content...>"))?;
                let content = args.get(1..).map(|rest| rest.join(" ")).unwrap_or_default();
                self.write_file(file, &content, ctx)
            }
            "rm" | "del" | "remove" => {
                let file = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("rm requiert <file>"))?;
                self.remove(file, ctx)
            }
            "mkdir" | "md" => {
                let dir = args
                    .first()
                    .ok_or_else(|| DevxError::invalid_args("mkdir requiert <dir>"))?;
                self.make_dir(dir, ctx)
            }
            "ls" | "list" => self.list(args.first().map(String::as_str), ctx),
            other => Err(DevxError::unknown_command(other)),
        }
    }
}

/// Indique si un répertoire doit être exclu de l'arborescence.
fn is_pruned_dir(name: &str) -> bool {
    name == ".git" || name == "target"
}

/// Collecte les fichiers sous `root` via `ignore` (respecte `.gitignore` et
/// ignore les entrées cachées), triés par chemin pour une sortie déterministe.
fn collect_files(root: &Path) -> Result<Vec<PathBuf>, DevxError> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in WalkBuilder::new(root).require_git(false).build() {
        let entry = entry.map_err(|err| DevxError::message(err.to_string()))?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// Convertit un chemin en chemin relatif à `root`, séparateurs `/`.
fn relative_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.to_string_lossy().replace('\\', "/")
}

/// Résout un chemin saisi par l'utilisateur relativement à `root` en refusant
/// les chemins absolus et toute tentative de sortie de la racine (`..`).
fn resolve_under_root(root: &Path, input: &str) -> Result<PathBuf, DevxError> {
    if input.trim().is_empty() {
        return Err(DevxError::invalid_args("chemin vide"));
    }

    let raw = Path::new(input);
    if raw.is_absolute() {
        return Err(DevxError::invalid_args(format!(
            "chemin absolu refusé: {input}"
        )));
    }

    let mut normalized = PathBuf::new();
    let mut depth: i64 = 0;
    for component in raw.components() {
        match component {
            Component::Normal(part) => {
                normalized.push(part);
                depth += 1;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(DevxError::invalid_args(format!(
                        "chemin hors de la racine refusé: {input}"
                    )));
                }
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(DevxError::invalid_args(format!(
                    "chemin absolu refusé: {input}"
                )));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        Ok(root.to_path_buf())
    } else {
        Ok(root.join(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::FilesModule;
    use devx_core::{Context, Module};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn exposes_expected_commands() {
        let names: Vec<String> = FilesModule::new()
            .commands()
            .into_iter()
            .map(|spec| spec.name)
            .collect();
        assert_eq!(
            names,
            ["tree", "find", "grep", "preview", "new", "write", "rm", "mkdir", "ls"]
        );
    }

    #[test]
    fn module_name_is_files() {
        assert_eq!(FilesModule::new().name(), "files");
    }

    #[test]
    fn unknown_command_is_rejected() {
        let ctx = Context::with_root(".");
        let err = FilesModule::new().execute(&ctx, "nope", &[]).unwrap_err();
        assert!(matches!(err, devx_core::DevxError::UnknownCommand(_)));
    }

    #[test]
    fn find_requires_query() {
        let ctx = Context::with_root(".");
        assert!(FilesModule::new()
            .execute(&ctx, "find", &args(&[]))
            .is_err());
    }
}
