//! Agent-surface smoke tests for the unvenv binary.

use std::process::Command;

fn unvenv_bin() -> &'static str {
    env!("CARGO_BIN_EXE_unvenv")
}

fn agent_command(args: &[&str]) -> Command {
    let mut command = Command::new(unvenv_bin());
    command.args(args);
    command.env("TFTIO_AGENT_TOKEN", "phase7-test-token");
    command.env("TFTIO_AGENT_TOKEN_EXPECTED", "phase7-test-token");
    command
}

#[test]
fn agent_surface_help_lists_only_scan_capability() {
    let output = agent_command(&["--agent-help"])
        .output()
        .expect("run unvenv --agent-help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scan-venvs"));
    assert!(!stdout.contains("doctor"));
    assert!(!stdout.contains("update"));
    assert!(!stdout.contains("completions"));
}

#[test]
fn agent_surface_skill_lists_scan_contract() {
    let output = agent_command(&["--agent-skill", "scan-venvs"])
        .output()
        .expect("run unvenv --agent-skill scan-venvs");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("capability:\n- scan-venvs"));
    assert!(stdout.contains("commands:\n- scan"));
    assert!(!stdout.contains("doctor"));
    assert!(!stdout.contains("update"));
}

#[test]
fn agent_surface_rejects_hidden_doctor_command() {
    let output = agent_command(&["doctor"])
        .output()
        .expect("run unvenv doctor");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand 'doctor'"));
    assert!(!stderr.contains("update"));
    assert!(!stderr.contains("completions"));
}
