//! Commande `norm` : vérifie la Norme 42 via `norminette`.
//!
//! Lance l'outil officiel `norminette` (installable avec `pip install
//! norminette`) sur un fichier, un dossier ou le projet entier, puis normalise
//! sa sortie en lignes `fichier:ligne:col: CODE — message`. Ce format est à la
//! fois lisible dans le panneau de sortie et reparsable par la TUI pour sauter
//! directement à la ligne fautive (voir `parse_norm_location` côté `devx-tui`).
//!
//! Un code de sortie non nul de `norminette` est normal (il signale des
//! violations) : seul l'échec de lancement du binaire produit une erreur.

use std::process::Command;

use devx_core::{Context, DevxError, Output};
use regex::Regex;

use crate::util::bytes_to_lines;

/// Une violation (`Error`) ou remarque (`Notice`) sur une ligne précise.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    line: usize,
    col: usize,
    code: String,
    message: String,
    notice: bool,
}

/// Rapport norminette pour un fichier donné.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileReport {
    file: String,
    ok: bool,
    entries: Vec<Entry>,
}

/// Lance `norminette <args...>` dans la racine du workspace et renvoie sa
/// sortie normalisée. Les `args` sont transmis tels quels (chemins et options),
/// sans shell : aucune injection possible.
pub fn execute(ctx: &Context, args: &[String]) -> Result<Output, DevxError> {
    let output = Command::new("norminette")
        .args(args)
        .current_dir(&ctx.workspace.root)
        .output()
        .map_err(|e| {
            DevxError::message(format!(
                "impossible de lancer `norminette` ({e}) — installe-le avec `pip install norminette`"
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report = parse_report(&stdout);

    // Sortie non reconnue : ne rien masquer.
    if report.is_empty() {
        if stdout.trim().is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.trim();
            if !output.status.success() {
                return Err(DevxError::message(if detail.is_empty() {
                    "`norminette` a échoué sans sortie".to_string()
                } else {
                    format!("`norminette` a échoué: {detail}")
                }));
            }
            return Ok(Output::new(vec!["norminette: aucune sortie".to_string()]));
        }
        return Ok(Output::new(bytes_to_lines(&output.stdout)));
    }

    Ok(Output::new(render(&report)))
}

/// Analyse la sortie brute de norminette en une liste de rapports par fichier.
fn parse_report(stdout: &str) -> Vec<FileReport> {
    let header = Regex::new(r"^(?P<file>.+?):\s+(?P<status>OK!|Error!)\s*$")
        .expect("regex d'en-tête norminette valide");
    let entry = Regex::new(
        r"^(?P<level>Error|Notice):\s+(?P<code>\S+)\s+\(line:\s*(?P<line>\d+),\s*col:\s*(?P<col>\d+)\):\s*(?P<msg>.*)$",
    )
    .expect("regex d'entrée norminette valide");

    let mut files: Vec<FileReport> = Vec::new();
    for raw in stdout.lines() {
        let line = raw.trim_end();
        if let Some(cap) = header.captures(line) {
            files.push(FileReport {
                file: cap["file"].to_string(),
                ok: &cap["status"] == "OK!",
                entries: Vec::new(),
            });
            continue;
        }
        if let Some(cap) = entry.captures(line) {
            let e = Entry {
                line: cap["line"].parse().unwrap_or(0),
                col: cap["col"].parse().unwrap_or(0),
                code: cap["code"].to_string(),
                message: cap["msg"].trim().to_string(),
                notice: &cap["level"] == "Notice",
            };
            match files.last_mut() {
                Some(last) => {
                    last.ok = false;
                    last.entries.push(e);
                }
                // Entrée sans en-tête préalable : fichier inconnu.
                None => files.push(FileReport {
                    file: String::new(),
                    ok: false,
                    entries: vec![e],
                }),
            }
        }
    }
    files
}

/// Rend le rapport en lignes affichables et reparsables.
fn render(files: &[FileReport]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut errors = 0usize;
    let mut notices = 0usize;
    let mut ok_files = 0usize;

    for fr in files {
        if fr.entries.is_empty() {
            if fr.ok {
                ok_files += 1;
                lines.push(format!("{}: OK", fr.file));
            } else {
                lines.push(format!("{}: Error!", fr.file));
            }
            continue;
        }
        for e in &fr.entries {
            if e.notice {
                notices += 1;
            } else {
                errors += 1;
            }
            let tag = if e.notice { "[notice] " } else { "" };
            let sep = if e.message.is_empty() { "" } else { " — " };
            lines.push(format!(
                "{}:{}:{}: {tag}{}{sep}{}",
                fr.file, e.line, e.col, e.code, e.message
            ));
        }
    }

    lines.push(String::new());
    if errors == 0 && notices == 0 {
        lines.push(format!(
            "norminette: OK — {ok_files} fichier(s) conforme(s)"
        ));
    } else {
        lines.push(format!(
            "norminette: {errors} erreur(s), {notices} notice(s) — {} fichier(s)",
            files.len()
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use devx_core::{Config, Workspace};

    fn norminette_available() -> bool {
        Command::new("norminette")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    const SAMPLE: &str = "\
main.c: Error!
Error: TOO_MANY_LINES          (line:   1, col:   0):\tFunction has more than 25 lines
Notice: EMPTY_LINE_FILE_START  (line:   1, col:   1):\tsome notice
ft_putchar.c: OK!";

    #[test]
    fn parse_splits_files_and_entries() {
        let report = parse_report(SAMPLE);
        assert_eq!(report.len(), 2);
        assert_eq!(report[0].file, "main.c");
        assert!(!report[0].ok);
        assert_eq!(report[0].entries.len(), 2);
        assert_eq!(report[0].entries[0].code, "TOO_MANY_LINES");
        assert_eq!(report[0].entries[0].line, 1);
        assert!(!report[0].entries[0].notice);
        assert!(report[0].entries[1].notice);
        assert_eq!(report[1].file, "ft_putchar.c");
        assert!(report[1].ok);
    }

    #[test]
    fn render_emits_locatable_lines_and_summary() {
        let lines = render(&parse_report(SAMPLE));
        assert!(lines
            .iter()
            .any(|l| l == "main.c:1:0: TOO_MANY_LINES — Function has more than 25 lines"));
        assert!(lines
            .iter()
            .any(|l| l.starts_with("main.c:1:1: [notice] EMPTY_LINE_FILE_START")));
        assert!(lines.iter().any(|l| l == "ft_putchar.c: OK"));
        assert_eq!(
            lines.last().unwrap(),
            "norminette: 1 erreur(s), 1 notice(s) — 2 fichier(s)"
        );
    }

    #[test]
    fn render_all_ok_summary() {
        let lines = render(&parse_report("a.c: OK!\nb.c: OK!"));
        assert_eq!(
            lines.last().unwrap(),
            "norminette: OK — 2 fichier(s) conforme(s)"
        );
    }

    #[test]
    fn parse_ignores_unrelated_text() {
        assert!(parse_report("hello world\nnot norminette output").is_empty());
    }

    #[test]
    fn execute_handles_binary_presence_or_absence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.c"), "int main(void){return (0);}\n").unwrap();
        let ctx = Context::new(Workspace::new(dir.path()), Config::default());
        let res = execute(&ctx, &["a.c".to_string()]);
        if norminette_available() {
            assert!(res.is_ok(), "norminette présent mais erreur: {res:?}");
        } else {
            assert!(
                matches!(res, Err(DevxError::Message(_))),
                "binaire absent: {res:?}"
            );
        }
    }
}
