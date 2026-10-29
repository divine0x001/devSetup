//! Détection du projet C et génération d'un Makefile 42 par défaut.
//!
//! Sert à la commande `:test` : repérer les sources `.c`, et — seulement s'il
//! n'existe pas déjà de Makefile — en écrire un au format 42 (NAME, CC, CFLAGS
//! `-Wall -Wextra -Werror`, règles `all/clean/fclean/re`). Un Makefile présent
//! n'est jamais écrasé. [`binary_target`] déduit le binaire à lancer depuis la
//! variable `NAME` du Makefile (défaut : `a.out`).

use std::path::{Path, PathBuf};

/// Profondeur maximale de recherche des sources.
const MAX_DEPTH: usize = 8;
/// Nombre maximal d'entrées parcourues (garde-fou).
const MAX_ENTRIES: usize = 10_000;

/// Répertoires ignorés lors de la détection des sources.
fn is_skipped(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "target" | "build" | "node_modules")
}

/// Liste les sources `.c` du projet, en chemins relatifs à `root`, triées.
pub(crate) fn detect_sources(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut budget = MAX_ENTRIES;
    walk(root, root, MAX_DEPTH, &mut budget, &mut out);
    out.sort();
    out
}

fn walk(root: &Path, dir: &Path, depth: usize, budget: &mut usize, out: &mut Vec<String>) {
    if depth == 0 || *budget == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *budget == 0 {
            return;
        }
        *budget -= 1;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !is_skipped(&name) {
                walk(root, &path, depth - 1, budget, out);
            }
        } else if name.ends_with(".c") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Nom de projet dérivé du dossier racine (caractères sûrs), défaut `program`.
fn project_name(root: &Path) -> String {
    let raw = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "program".to_string()
    } else {
        cleaned
    }
}

/// Texte d'un Makefile 42 standard pour `name` et les sources `srcs`.
pub(crate) fn generate_makefile(name: &str, srcs: &[String]) -> String {
    format!(
        "NAME = {name}\n\
         CC = cc\n\
         CFLAGS = -Wall -Wextra -Werror\n\
         SRCS = {srcs}\n\
         OBJS = $(SRCS:.c=.o)\n\
         \n\
         all: $(NAME)\n\
         \n\
         $(NAME): $(OBJS)\n\
         \t$(CC) $(CFLAGS) -o $(NAME) $(OBJS)\n\
         \n\
         %.o: %.c\n\
         \t$(CC) $(CFLAGS) -c $< -o $@\n\
         \n\
         clean:\n\
         \trm -f $(OBJS)\n\
         \n\
         fclean: clean\n\
         \trm -f $(NAME)\n\
         \n\
         re: fclean all\n\
         \n\
         .PHONY: all clean fclean re\n",
        name = name,
        srcs = srcs.join(" "),
    )
}

/// Chemin d'un Makefile existant dans `root`, le cas échéant.
fn existing_makefile(root: &Path) -> Option<PathBuf> {
    for candidate in ["Makefile", "makefile", "GNUmakefile"] {
        let p = root.join(candidate);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Garantit un Makefile dans `root` : réutilise celui qui existe, sinon en
/// génère un à partir des sources détectées. Renvoie `(chemin, généré)`.
pub(crate) fn ensure_makefile(root: &Path) -> Result<(PathBuf, bool), String> {
    if let Some(path) = existing_makefile(root) {
        return Ok((path, false));
    }
    let srcs = detect_sources(root);
    if srcs.is_empty() {
        return Err("aucune source .c trouvée — impossible de générer un Makefile".to_string());
    }
    let path = root.join("Makefile");
    let text = generate_makefile(&project_name(root), &srcs);
    std::fs::write(&path, text).map_err(|e| format!("écriture du Makefile: {e}"))?;
    Ok((path, true))
}

/// Nom du binaire à lancer : la variable `NAME` du Makefile, sinon `a.out`.
pub(crate) fn binary_target(root: &Path) -> String {
    let Some(path) = existing_makefile(root) else {
        return "a.out".to_string();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return "a.out".to_string();
    };
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("NAME") {
            let rest = rest.trim_start();
            if let Some(value) = rest
                .strip_prefix('=')
                .or_else(|| rest.strip_prefix(":=").or_else(|| rest.strip_prefix("?=")))
            {
                if let Some(token) = value.split_whitespace().next() {
                    return token.to_string();
                }
            }
        }
    }
    "a.out".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn detect_sources_finds_nested_and_skips_hidden() {
        let dir = tmp();
        std::fs::write(dir.path().join("main.c"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/util.c"), "").unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/hook.c"), "").unwrap();
        let srcs = detect_sources(dir.path());
        assert_eq!(srcs, vec!["main.c".to_string(), "src/util.c".to_string()]);
    }

    #[test]
    fn generated_makefile_has_42_flags_and_rules() {
        let mk = generate_makefile("philo", &["main.c".to_string(), "src/a.c".to_string()]);
        assert!(mk.contains("CFLAGS = -Wall -Wextra -Werror"));
        assert!(mk.contains("NAME = philo"));
        assert!(mk.contains("SRCS = main.c src/a.c"));
        assert!(mk.contains(".PHONY: all clean fclean re"));
        assert!(mk.contains("\t$(CC) $(CFLAGS) -o $(NAME) $(OBJS)"));
    }

    #[test]
    fn ensure_generates_when_absent_and_keeps_existing() {
        let dir = tmp();
        std::fs::write(dir.path().join("main.c"), "int main(void){return 0;}\n").unwrap();
        let (path, generated) = ensure_makefile(dir.path()).unwrap();
        assert!(generated);
        assert!(path.is_file());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("main.c"));

        // Un second appel ne régénère pas : le Makefile est préservé tel quel.
        std::fs::write(&path, "custom\n").unwrap();
        let (again, generated2) = ensure_makefile(dir.path()).unwrap();
        assert!(!generated2);
        assert_eq!(again, path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "custom\n");
    }

    #[test]
    fn ensure_errors_without_sources() {
        let dir = tmp();
        assert!(ensure_makefile(dir.path()).is_err());
    }

    #[test]
    fn binary_target_reads_name_or_defaults() {
        let dir = tmp();
        assert_eq!(binary_target(dir.path()), "a.out");
        std::fs::write(dir.path().join("Makefile"), "NAME = push_swap\nall:\n").unwrap();
        assert_eq!(binary_target(dir.path()), "push_swap");
    }
}
