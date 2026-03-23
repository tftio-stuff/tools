//! Integration tests for asana-cli agent-facing documentation entrypoints.

use std::process::Command;

fn run_asana_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_asana-cli"))
        .args(args)
        .output()
        .expect("asana-cli binary should run for integration tests")
}

#[test]
fn agent_help_top_level_request_succeeds_without_subcommand() {
    let output = run_asana_cli(&["--agent-help"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"asana-cli\""));
    assert!(stdout.contains("ASANA_PAT"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("task"));
    assert!(stdout.contains("project"));
    assert!(stdout.contains("section"));
    assert!(stdout.contains("tag"));
    assert!(stdout.contains("custom-field"));
    assert!(stdout.contains("workspace"));
    assert!(stdout.contains("user"));
    assert!(stdout.contains("completions"));
    assert!(stdout.contains("manpage"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("json"));
    assert!(stdout.contains("cache"));
}

#[test]
fn agent_help_skill_top_level_request_succeeds_without_subcommand() {
    let output = run_asana_cli(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: \"asana-cli\""));
    assert!(stdout.contains("# asana-cli"));
    assert!(stdout.contains("task subtasks"));
    assert!(stdout.contains("project members"));
    assert!(stdout.contains("network"));
    assert!(stdout.contains("Operator mistakes"));
}

#[test]
fn agent_help_hidden_flags_stay_out_of_normal_help_output() {
    let output = run_asana_cli(&["task", "--help"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("--agent-help"));
    assert!(!stdout.contains("--agent-skill"));
}

#[test]
fn agent_help_flags_are_rejected_when_placed_after_a_subcommand() {
    let output = run_asana_cli(&["task", "list", "--agent-help"]);

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"asana-cli\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("Usage:"));
}
