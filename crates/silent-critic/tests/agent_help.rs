use std::process::Command;

fn run_silent_critic(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_silent-critic"))
        .args(args)
        .output()
        .expect("silent-critic binary should run for integration tests")
}

#[test]
fn agent_help_top_level_request_succeeds_without_subcommand() {
    let output = run_silent_critic(&["--agent-help"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"silent-critic\""));
    assert!(stdout.contains("SILENT_CRITIC_TOKEN"));
    assert!(stdout.contains("awaiting_adjudication"));
    assert!(stdout.contains("db.sqlite"));
    assert!(stdout.contains("markdown"));
}

#[test]
fn agent_help_skill_top_level_request_succeeds_without_subcommand() {
    let output = run_silent_critic(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: \"silent-critic\""));
    assert!(stdout.contains("# silent-critic"));
    assert!(stdout.contains("session compose-from"));
    assert!(stdout.contains("contract show"));
    assert!(stdout.contains("operator mistakes"));
}

#[test]
fn agent_help_hidden_flags_stay_out_of_normal_help_output() {
    let output = run_silent_critic(&["session", "--help"]);

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
    let output = run_silent_critic(&["session", "--agent-help"]);

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("name: \"silent-critic\""));
    assert!(stderr.contains("--agent-help") || stderr.contains("Usage:"));
}
