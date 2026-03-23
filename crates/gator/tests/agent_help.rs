//! Integration tests for `gator` agent-facing documentation flags.

use std::process::{Command, Output};

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_gator")
}

fn run_gator(args: &[&str]) -> Output {
    Command::new(bin_path()).args(args).output().expect("run gator")
}

#[test]
fn agent_help_top_level_prints_exhaustive_yaml() {
    let output = run_gator(&["--agent-help"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"gator\""));
    assert!(stdout.contains("claude"));
    assert!(stdout.contains("codex"));
    assert!(stdout.contains("gemini"));
    assert!(stdout.contains("--policy"));
    assert!(stdout.contains("--session"));
    assert!(stdout.contains("--share-worktrees"));
    assert!(stdout.contains("--no-yolo"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("sandbox-exec"));
    assert!(stdout.contains("silent-critic"));
}

#[test]
fn agent_help_top_level_agent_skill_prints_skill_document() {
    let output = run_gator(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("name: \"gator\""));
    assert!(stdout.contains("# gator"));
    assert!(stdout.contains("## Commands"));
    assert!(stdout.contains("`gator claude rust.full --dry-run`"));
    assert!(stdout.contains("session mode"));
}

#[test]
fn agent_help_positional_invocation_does_not_trigger_top_level_doc_flow() {
    let output = run_gator(&["claude", "--agent-help"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"gator\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("unexpected argument"));
}

#[test]
fn agent_help_human_help_hides_agent_flags() {
    let output = run_gator(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("--agent-help"));
    assert!(!stdout.contains("--agent-skill"));
    assert!(stdout.contains("AGENT") || stdout.contains("<AGENT>"));
    assert!(stdout.contains("--session"));
}
