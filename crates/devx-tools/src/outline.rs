//! Commande `outline` : symboles de premier niveau d'un fichier source.

use std::path::{Path, PathBuf};

use devx_core::{Context, DevxError, Output};
use regex::Regex;

/// Exécute `outline <file>`.
///
/// Détecte les symboles de premier niveau (colonne 0) selon l'extension du
/// fichier et renvoie une ligne `ligne: symbole` par symbole trouvé. Les
/// symboles imbriqués (indentés) sont ignorés.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let file = match args {
        [file] => file,
        _ => return Err(DevxError::invalid_args("usage: outline <file>")),
    };

    let path = resolve(&ctx.workspace.root, file);
    if !path.is_file() {
        return Err(DevxError::message(format!("fichier introuvable: `{file}`")));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let patterns = patterns_for(&extension).ok_or_else(|| {
        DevxError::message(format!(
            "extension non supportée pour l'outline: `{extension}`"
        ))
    })?;

    let content = std::fs::read_to_string(&path)
        .map_err(|e| DevxError::io(format!("lecture de {}: {e}", path.display())))?;

    let mut lines = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if let Some(symbol) = first_symbol(line, &patterns) {
            lines.push(format!("{}: {symbol}", index + 1));
        }
    }
    Ok(Output::new(lines))
}

/// Résout `file` relativement à la racine du workspace.
fn resolve(root: &Path, file: &str) -> PathBuf {
    let candidate = Path::new(file);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    }
}

/// Renvoie le premier symbole capturé sur une ligne, motifs testés dans l'ordre.
fn first_symbol(line: &str, patterns: &[Regex]) -> Option<String> {
    for pattern in patterns {
        if let Some(captures) = pattern.captures(line) {
            if let Some(symbol) = captures.get(1) {
                return Some(symbol.as_str().to_string());
            }
        }
    }
    None
}

/// Motifs de premier niveau pour une extension donnée.
///
/// Chaque motif est ancré en début de ligne : cela restreint la détection aux
/// éléments non indentés (premier niveau).
fn patterns_for(extension: &str) -> Option<Vec<Regex>> {
    let raw: &[&str] = match extension {
        "rs" => &[
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?(?:extern\s+\S+\s+)?fn\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?struct\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?enum\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?trait\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?const\s+([A-Za-z_]\w*)",
            r"^(?:pub(?:\s*\([^)]*\))?\s+)?static\s+(?:mut\s+)?([A-Za-z_]\w*)",
            r"^impl(?:<[^>]*>)?\s+(?:[\w:<>,\s]+?\s+for\s+)?([A-Za-z_]\w*)",
        ],
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => &[
            r"^(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*([A-Za-z_$][\w$]*)",
            r"^(?:export\s+)?(?:default\s+)?(?:abstract\s+)?class\s+([A-Za-z_$][\w$]*)",
            r"^(?:export\s+)?(?:declare\s+)?interface\s+([A-Za-z_$][\w$]*)",
            r"^(?:export\s+)?type\s+([A-Za-z_$][\w$]*)",
            r"^(?:export\s+)?(?:const|let)\s+([A-Za-z_$][\w$]*)",
        ],
        "py" => &[
            r"^(?:async\s+)?def\s+([A-Za-z_]\w*)",
            r"^class\s+([A-Za-z_]\w*)",
        ],
        _ => return None,
    };

    Some(
        raw.iter()
            .map(|pattern| Regex::new(pattern).expect("motif d'outline interne valide"))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::Workspace;

    fn ctx(dir: &Path) -> Context {
        Context::new(Workspace::new(dir), devx_core::Config::default())
    }

    #[test]
    fn outlines_rust_top_level_symbols() {
        let dir = tempfile::tempdir().unwrap();
        let source = "\
fn main() {}
struct Foo;
enum E { A }
trait T {}
impl Display for Foo {}
mod m {}
const C: u8 = 1;
static S: u8 = 1;
";
        std::fs::write(dir.path().join("lib.rs"), source).unwrap();

        let out = execute(&ctx(dir.path()), &["lib.rs".to_string()]).unwrap();
        assert_eq!(
            out.lines,
            vec!["1: main", "2: Foo", "3: E", "4: T", "5: Foo", "6: m", "7: C", "8: S",]
        );
    }

    #[test]
    fn outlines_python_top_level_only() {
        let dir = tempfile::tempdir().unwrap();
        let source = "\
def hello():
    pass
class World:
    def nested(self):
        pass
";
        std::fs::write(dir.path().join("app.py"), source).unwrap();

        let out = execute(&ctx(dir.path()), &["app.py".to_string()]).unwrap();
        assert_eq!(out.lines, vec!["1: hello", "3: World"]);
    }

    #[test]
    fn outlines_javascript() {
        let dir = tempfile::tempdir().unwrap();
        let source = "\
export function run() {}
class App {}
const value = 1;
interface Shape {}
type Alias = string;
";
        std::fs::write(dir.path().join("app.ts"), source).unwrap();

        let out = execute(&ctx(dir.path()), &["app.ts".to_string()]).unwrap();
        assert_eq!(
            out.lines,
            vec!["1: run", "2: App", "3: value", "4: Shape", "5: Alias"]
        );
    }

    #[test]
    fn missing_file_is_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["nope.rs".to_string()]),
            Err(DevxError::Message(_))
        ));
    }

    #[test]
    fn unsupported_extension_is_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data.bin"), "x").unwrap();
        assert!(matches!(
            execute(&ctx(dir.path()), &["data.bin".to_string()]),
            Err(DevxError::Message(_))
        ));
    }
}
