use std::process::Command;

fn run_todoer(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_todoer"))
        .args(args)
        .output()
        .expect("todoer binary should run for integration tests")
}

#[test]
fn agent_help_top_level_request_succeeds_without_subcommand() {
    let output = run_todoer(&["--agent-help"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"todoer\""));
    assert!(stdout.contains(".todoer.toml"));
    assert!(stdout.contains("task update status"));
    assert!(stdout.contains("stdin"));
    assert!(stdout.contains("json"));
}

#[test]
fn agent_help_skill_top_level_request_succeeds_without_subcommand() {
    let output = run_todoer(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: \"todoer\""));
    assert!(stdout.contains("# todoer"));
    assert!(stdout.contains("project resolution"));
    assert!(stdout.contains("task update status"));
    assert!(stdout.contains("JSON"));
}

#[test]
fn agent_help_hidden_flags_stay_out_of_normal_help_output() {
    let output = run_todoer(&["list", "--help"]);

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
    let output = run_todoer(&["task", "--agent-help"]);

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"todoer\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("Usage:"));
}
