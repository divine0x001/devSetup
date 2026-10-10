//! Commande `replace` : remplacement regex projet-wide respectant `.gitignore`.

use std::path::Path;

use devx_core::{Context, DevxError, Output};
use ignore::WalkBuilder;
use regex::Regex;

/// Exécute `replace <pattern> <replacement>` sur tous les fichiers texte du
/// workspace.
///
/// Le parcours respecte `.gitignore` (via la crate `ignore`) et ignore les
/// fichiers binaires (présence d'un octet NUL) ou non-UTF-8. La sortie liste les
/// fichiers modifiés avec leur nombre de remplacements, puis un total. Les
/// fichiers inchangés ne sont pas réécrits.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let (pattern, replacement) = match args {
        [pattern, replacement] => (pattern, replacement),
        _ => {
            return Err(DevxError::invalid_args(
                "usage: replace <pattern> <replacement>",
            ))
        }
    };

    let regex =
        Regex::new(pattern).map_err(|e| DevxError::invalid_args(format!("regex invalide: {e}")))?;

    let mut changed: Vec<(String, usize)> = Vec::new();
    let mut total = 0usize;

    let walker = WalkBuilder::new(&ctx.workspace.root)
        .require_git(false)
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path();
        let Some(content) = read_text(path) else {
            continue;
        };

        let matches = regex.find_iter(&content).count();
        if matches == 0 {
            continue;
        }

        let replaced = regex.replace_all(&content, replacement.as_str());
        if replaced.as_ref() == content {
            continue;
        }

        std::fs::write(path, replaced.as_ref())
            .map_err(|e| DevxError::io(format!("écriture de {}: {e}", path.display())))?;

        let rel = path.strip_prefix(&ctx.workspace.root).unwrap_or(path);
        changed.push((rel.display().to_string(), matches));
        total += matches;
    }

    changed.sort();
    let mut lines: Vec<String> = changed
        .iter()
        .map(|(path, count)| format!("{path}: {count} remplacement(s)"))
        .collect();
    lines.push(format!(
        "{total} remplacement(s) dans {} fichier(s)",
        changed.len()
    ));
    Ok(Output::new(lines))
}

/// Lit un fichier s'il s'agit de texte UTF-8 sans octet NUL.
fn read_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.contains(&0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::Workspace;

    fn ctx(dir: &Path) -> Context {
        Context::new(Workspace::new(dir), devx_core::Config::default())
    }

    #[test]
    fn replaces_and_counts() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "foo foo\nbar\n").unwrap();
        std::fs::write(dir.path().join("b.txt"), "foo\n").unwrap();

        let out = execute(&ctx(dir.path()), &["foo".to_string(), "qux".to_string()]).unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "qux qux\nbar\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("b.txt")).unwrap(),
            "qux\n"
        );
        assert!(out.lines.iter().any(|l| l == "a.txt: 2 remplacement(s)"));
        assert!(out.lines.iter().any(|l| l == "b.txt: 1 remplacement(s)"));
        assert_eq!(
            out.lines.last().unwrap(),
            "3 remplacement(s) dans 2 fichier(s)"
        );
    }

    #[test]
    fn respects_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "ignored.txt\n").unwrap();
        std::fs::write(dir.path().join("ignored.txt"), "foo\n").unwrap();
        std::fs::write(dir.path().join("kept.txt"), "foo\n").unwrap();

        execute(&ctx(dir.path()), &["foo".to_string(), "qux".to_string()]).unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.path().join("kept.txt")).unwrap(),
            "qux\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("ignored.txt")).unwrap(),
            "foo\n"
        );
    }

    #[test]
    fn skips_binary_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bin.dat"), b"foo\0foo").unwrap();

        let out = execute(&ctx(dir.path()), &["foo".to_string(), "qux".to_string()]).unwrap();

        assert_eq!(
            std::fs::read(dir.path().join("bin.dat")).unwrap(),
            b"foo\0foo"
        );
        assert_eq!(
            out.lines.last().unwrap(),
            "0 remplacement(s) dans 0 fichier(s)"
        );
    }

    #[test]
    fn invalid_regex_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["(".to_string(), "x".to_string()]),
            Err(DevxError::InvalidArgs(_))
        ));
    }

    #[test]
    fn missing_args_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["only".to_string()]),
            Err(DevxError::InvalidArgs(_))
        ));
    }
}
