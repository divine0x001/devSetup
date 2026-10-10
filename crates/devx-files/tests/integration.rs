use std::fs;
use std::path::Path;

use devx_core::{Context, Module};
use devx_files::FilesModule;
use tempfile::TempDir;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

/// Arborescence temporaire représentative, avec un marqueur `devx.toml` pour
/// figer la racine du workspace.
fn fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    write(root, "devx.toml", "");
    write(
        root,
        "src/main.rs",
        "fn main() {\n    println!(\"hi\");\n}\n",
    );
    write(root, "src/lib.rs", "let needle = 1;\nother line\n");
    write(root, "docs/readme.md", "# docs\n");
    write(root, "ignored.txt", "needle should be ignored\n");
    write(root, "target/junk.txt", "needle in target\n");
    write(root, ".git/config", "[core]\n");
    write(root, ".gitignore", "ignored.txt\ntarget/\n");

    dir
}

fn execute(root: &Path, command: &str, items: &[&str]) -> Vec<String> {
    let ctx = Context::with_root(root);
    FilesModule::new()
        .execute(&ctx, command, &args(items))
        .unwrap()
        .lines
}

#[test]
fn tree_lists_files_and_skips_git_target_and_gitignored() {
    let dir = fixture();
    let lines = execute(dir.path(), "tree", &[]);

    assert!(lines.iter().any(|line| line.trim() == "src/"));
    assert!(lines.iter().any(|line| line.trim() == "main.rs"));
    assert!(lines.iter().any(|line| line.trim() == "lib.rs"));
    assert!(!lines.iter().any(|line| line.contains(".git/")));
    assert!(!lines.iter().any(|line| line.trim() == ".git"));
    assert!(!lines.iter().any(|line| line.trim() == "target/"));
    assert!(!lines.iter().any(|line| line.contains("junk.txt")));
    assert!(!lines.iter().any(|line| line.contains("ignored.txt")));
}

#[test]
fn find_returns_fuzzy_match_first() {
    let dir = fixture();
    let lines = execute(dir.path(), "find", &["main"]);

    assert_eq!(lines.first().map(String::as_str), Some("src/main.rs"));
    assert!(lines.len() <= devx_files::FIND_LIMIT);
}

#[test]
fn grep_reports_file_line_text_and_respects_gitignore() {
    let dir = fixture();
    let lines = execute(dir.path(), "grep", &["needle"]);

    assert_eq!(lines, vec!["src/lib.rs:1:let needle = 1;".to_string()]);
}

#[test]
fn grep_rejects_invalid_regex() {
    let dir = fixture();
    let ctx = Context::with_root(dir.path());
    let err = FilesModule::new()
        .execute(&ctx, "grep", &args(&["("]))
        .unwrap_err();
    assert!(matches!(err, devx_core::DevxError::InvalidArgs(_)));
}

#[test]
fn preview_numbers_lines_and_honours_limit() {
    let dir = fixture();
    let lines = execute(dir.path(), "preview", &["src/main.rs", "2"]);

    assert_eq!(
        lines,
        vec![
            "1: fn main() {".to_string(),
            "2:     println!(\"hi\");".to_string()
        ]
    );
}

#[test]
fn preview_defaults_to_forty_lines() {
    let dir = TempDir::new().unwrap();
    let content: String = (1..=60).map(|n| format!("line {n}\n")).collect();
    write(dir.path(), "big.txt", &content);

    let lines = execute(dir.path(), "preview", &["big.txt"]);
    assert_eq!(lines.len(), devx_files::DEFAULT_PREVIEW_LINES);
    assert_eq!(lines[0], "1: line 1");
}

#[test]
fn preview_missing_file_errors() {
    let dir = fixture();
    let ctx = Context::with_root(dir.path());
    assert!(FilesModule::new()
        .execute(&ctx, "preview", &args(&["does/not/exist.txt"]))
        .is_err());
}

#[test]
fn aliases_are_supported() {
    let dir = fixture();
    assert!(!execute(dir.path(), "t", &[]).is_empty());
    assert_eq!(
        execute(dir.path(), "f", &["main"])
            .first()
            .map(String::as_str),
        Some("src/main.rs")
    );
}

fn run(root: &Path, command: &str, items: &[&str]) -> Result<Vec<String>, devx_core::DevxError> {
    let ctx = Context::with_root(root);
    FilesModule::new()
        .execute(&ctx, command, &args(items))
        .map(|output| output.lines)
}

#[test]
fn new_creates_file_and_creates_parents() {
    let dir = fixture();
    let lines = execute(dir.path(), "new", &["a/b/c.txt"]);

    assert_eq!(lines, vec!["créé: a/b/c.txt".to_string()]);
    assert!(dir.path().join("a/b/c.txt").is_file());
}

#[test]
fn new_errors_when_file_exists() {
    let dir = fixture();
    let err = run(dir.path(), "new", &["src/main.rs"]).unwrap_err();
    assert!(matches!(err, devx_core::DevxError::Message(_)));
}

#[test]
fn write_then_preview_roundtrip() {
    let dir = fixture();
    execute(dir.path(), "write", &["notes.txt", "hello", "world"]);

    assert_eq!(
        fs::read_to_string(dir.path().join("notes.txt")).unwrap(),
        "hello world"
    );
    let lines = execute(dir.path(), "preview", &["notes.txt"]);
    assert_eq!(lines, vec!["1: hello world".to_string()]);
}

#[test]
fn rm_removes_file_and_errors_when_absent() {
    let dir = fixture();
    let lines = execute(dir.path(), "rm", &["ignored.txt"]);
    assert_eq!(lines, vec!["supprimé: ignored.txt".to_string()]);
    assert!(!dir.path().join("ignored.txt").exists());

    let err = run(dir.path(), "rm", &["ignored.txt"]).unwrap_err();
    assert!(matches!(err, devx_core::DevxError::Message(_)));
}

#[test]
fn mkdir_creates_nested_directories() {
    let dir = fixture();
    let lines = execute(dir.path(), "mkdir", &["x/y/z"]);
    assert_eq!(lines, vec!["dossier créé: x/y/z".to_string()]);
    assert!(dir.path().join("x/y/z").is_dir());
}

#[test]
fn ls_lists_directories_first() {
    let dir = fixture();
    let lines = execute(dir.path(), "ls", &[]);

    let idx = |needle: &str| lines.iter().position(|line| line == needle);
    let docs = idx("docs/").unwrap();
    let src = idx("src/").unwrap();
    let toml = idx("devx.toml").unwrap();
    assert!(
        docs < src && src < toml,
        "ordre attendu (dossiers d'abord): {lines:?}"
    );
}

#[test]
fn ls_defaults_to_workspace_root_and_accepts_subdir() {
    let dir = fixture();
    let sub = execute(dir.path(), "ls", &["src"]);
    assert!(sub.iter().any(|line| line == "main.rs"));
    assert!(sub.iter().any(|line| line == "lib.rs"));
    assert!(!sub.iter().any(|line| line == "src/"));
}

#[test]
fn path_traversal_out_of_root_is_refused() {
    let dir = fixture();
    let err = run(dir.path(), "new", &["../escape.txt"]).unwrap_err();
    assert!(matches!(err, devx_core::DevxError::InvalidArgs(_)));
    let err = run(dir.path(), "write", &["a/../../etc/passwd", "x"]).unwrap_err();
    assert!(matches!(err, devx_core::DevxError::InvalidArgs(_)));
}

#[test]
fn absolute_path_is_refused() {
    let dir = fixture();
    let err = run(dir.path(), "new", &["/tmp/escape.txt"]).unwrap_err();
    assert!(matches!(err, devx_core::DevxError::InvalidArgs(_)));
}
