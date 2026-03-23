//! Integration tests for the restricted `gator` agent surface.

use std::process::{Command, Output};

const TEST_TOKEN: &str = "phase7-test-token";

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_gator")
}

fn agent_command() -> Command {
    let mut command = Command::new(bin_path());
    command
        .env("TFTIO_AGENT_TOKEN", TEST_TOKEN)
        .env("TFTIO_AGENT_TOKEN_EXPECTED", TEST_TOKEN);
    command
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn agent_surface_help_exposes_only_run_agent_capability() {
    let output = agent_command()
        .arg("--agent-help")
        .output()
        .expect("run gator --agent-help");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains("run-agent"), "stdout: {stdout}");
    assert!(!stdout.contains("meta"), "stdout: {stdout}");
}

#[test]
fn agent_surface_skill_renders_run_agent_contract() {
    let output = agent_command()
        .args(["--agent-skill", "run-agent"])
        .output()
        .expect("run gator --agent-skill run-agent");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains("capability:\n- run-agent"), "stdout: {stdout}");
    assert!(!stdout.contains("meta"), "stdout: {stdout}");
}

#[test]
fn agent_surface_hides_meta_subcommands() {
    let output = agent_command()
        .args(["meta", "version"])
        .output()
        .expect("run gator meta version");

    assert!(!output.status.success(), "stdout: {}", stdout(&output));

    let stderr = stderr(&output);
    assert!(
        stderr.contains("unrecognized subcommand 'meta'"),
        "stderr: {stderr}"
    );
}

#[test]
fn agent_surface_rejects_hidden_privileged_flags() {
    let session = agent_command()
        .args(["claude", "--session", "abc-123"])
        .output()
        .expect("run gator with hidden session flag");

    assert!(!session.status.success(), "stdout: {}", stdout(&session));
    assert!(stderr(&session).contains("--session"), "stderr: {}", stderr(&session));

    let help = agent_command()
        .arg("--help")
        .output()
        .expect("run gator --help");

    assert!(help.status.success(), "stderr: {}", stderr(&help));

    let stdout = stdout(&help);
    for hidden in [
        "--add-dirs",
        "--add-dirs-ro",
        "--share-worktrees",
        "--session",
        "--no-yolo",
    ] {
        assert!(!stdout.contains(hidden), "stdout: {stdout}");
    }
}
