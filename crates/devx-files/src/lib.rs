//! Module `files` du projet `devx`.
//!
//! Expose [`FilesModule`], qui implémente [`devx_core::Module`] et fournit les
//! commandes orientées fichiers :
//!
//! - `tree` : arborescence de `ctx.workspace.root` (`.git`/`target` exclus,
//!   `.gitignore` respecté) ;
//! - `find <query>` : fuzzy finder par sous-séquence, top 20 par score ;
//! - `grep <pattern>` : recherche récursive, sortie `chemin:ligne:texte` ;
//! - `preview <file> [n]` : `n` premières lignes numérotées (défaut 40).

mod fuzzy;

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
        assert_eq!(names, ["tree", "find", "grep", "preview"]);
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
