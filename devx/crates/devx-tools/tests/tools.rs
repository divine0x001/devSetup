//! Tests d'intégration du module `tools` via sa surface publique.

use devx_core::{Context, Module};
use devx_tools::ToolsModule;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn run_reads_tasks_from_devx_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("devx.toml"),
        "[tasks]\nhello = \"echo bonjour\"\n",
    )
    .unwrap();

    let ctx = Context::with_root(dir.path());
    let out = ToolsModule::new()
        .execute(&ctx, "run", &args(&["hello"]))
        .unwrap();
    assert_eq!(out.lines, vec!["bonjour"]);
}

#[test]
fn snip_round_trip_via_module() {
    let dir = tempfile::tempdir().unwrap();
    let module = ToolsModule::new();
    let ctx = Context::with_root(dir.path());

    module
        .execute(&ctx, "snip", &args(&["add", "greet", "hello\nworld"]))
        .unwrap();

    let got = module
        .execute(&ctx, "snip", &args(&["get", "greet"]))
        .unwrap();
    assert_eq!(got.lines, vec!["hello", "world"]);
}

#[test]
fn session_round_trip_via_module() {
    let dir = tempfile::tempdir().unwrap();
    let module = ToolsModule::new();
    let ctx = Context::with_root(dir.path());

    module
        .execute(&ctx, "session", &args(&["save", "s1", "main.rs"]))
        .unwrap();

    let out = module
        .execute(&ctx, "session", &args(&["load", "s1"]))
        .unwrap();
    assert!(out.lines.iter().any(|l| l == "file: main.rs"));
}
