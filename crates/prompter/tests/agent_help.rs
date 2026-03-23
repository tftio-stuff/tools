//! Integration tests for prompter agent-facing documentation entrypoints.

use std::process::Command;

fn run_prompter(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_prompter"))
        .args(args)
        .output()
        .expect("prompter binary should run for integration tests")
}

#[test]
fn agent_help_top_level_request_succeeds_without_subcommand() {
    let output = run_prompter(&["--agent-help"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"prompter\""));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("tree"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("completions"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("json"));
    assert!(stdout.contains("library"));
    assert!(stdout.contains("duplicate"));
    assert!(stdout.contains("profile"));
}

#[test]
fn agent_help_skill_top_level_request_succeeds_without_subcommand() {
    let output = run_prompter(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: \"prompter\""));
    assert!(stdout.contains("# prompter"));
    assert!(stdout.contains("recursive"));
    assert!(stdout.contains("JSON"));
    assert!(stdout.contains("operator mistakes") || stdout.contains("Operator mistakes"));
    assert!(stdout.contains("missing library files") || stdout.contains("library"));
}

#[test]
fn agent_help_hidden_flags_stay_out_of_normal_help_output() {
    let output = run_prompter(&["run", "--help"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("--agent-help"));
    assert!(!combined.contains("--agent-skill"));
}

#[test]
fn agent_help_flags_are_rejected_when_placed_after_a_subcommand() {
    let output = run_prompter(&["run", "profile", "--agent-help"]);

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"prompter\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("Usage:"));
}
