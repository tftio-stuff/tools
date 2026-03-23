//! Integration tests for `unvenv` agent-facing documentation flags.

use std::process::{Command, Output};

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_unvenv")
}

fn run_unvenv(args: &[&str]) -> Output {
    Command::new(bin_path())
        .args(args)
        .output()
        .expect("run unvenv")
}

#[test]
fn agent_help_top_level_prints_exhaustive_yaml() {
    let output = run_unvenv(&["--agent-help"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"unvenv\""));
    assert!(stdout.contains("name: \"scan\""));
    assert!(stdout.contains("name: \"doctor\""));
    assert!(stdout.contains("name: \"completions\""));
    assert!(stdout.contains("name: \"update\""));
    assert!(stdout.contains("exit code 2"));
    assert!(stdout.contains("pyvenv.cfg"));
    assert!(stdout.contains(".gitignore"));
    assert!(stdout.contains("Running `unvenv scan --agent-help`"));
}

#[test]
fn agent_help_top_level_agent_skill_prints_skill_document() {
    let output = run_unvenv(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("name: \"unvenv\""));
    assert!(stdout.contains("# unvenv"));
    assert!(stdout.contains("## Commands"));
    assert!(stdout.contains("`unvenv scan`"));
    assert!(stdout.contains("`unvenv update --version"));
}

#[test]
fn agent_help_subcommand_does_not_trigger_top_level_doc_flow() {
    let output = run_unvenv(&["scan", "--agent-help"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"unvenv\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("unexpected argument"));
}

#[test]
fn agent_help_human_help_hides_agent_flags() {
    let output = run_unvenv(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("--agent-help"));
    assert!(!stdout.contains("--agent-skill"));
    assert!(stdout.contains("scan"));
    assert!(stdout.contains("doctor"));
}
