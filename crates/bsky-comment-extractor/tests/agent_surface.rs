//! Integration tests for the shared restricted `bce` agent surface.

use std::process::{Command, Output};

const TEST_TOKEN: &str = "phase7-test-token";

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_bce")
}

fn agent_command() -> Command {
    let mut command = Command::new(bin_path());
    command
        .env("TFTIO_AGENT_TOKEN", TEST_TOKEN)
        .env("TFTIO_AGENT_TOKEN_EXPECTED", TEST_TOKEN)
        .env_remove("BSKY_APP_PASSWORD");
    command
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn agent_help_shows_only_query_posts_capability() {
    let output = agent_command()
        .arg("--agent-help")
        .output()
        .expect("run bce --agent-help");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains("query-posts"), "stdout: {stdout}");
    assert!(!stdout.contains("fetch"), "stdout: {stdout}");
}

#[test]
fn agent_skill_query_posts_renders_single_capability_contract() {
    let output = agent_command()
        .args(["--agent-skill", "query-posts"])
        .output()
        .expect("run bce --agent-skill query-posts");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(stdout.contains("capability:\n- query-posts"), "stdout: {stdout}");
    assert!(stdout.contains("commands:\n- query"), "stdout: {stdout}");
    assert!(!stdout.contains("fetch"), "stdout: {stdout}");
}

#[test]
fn hidden_fetch_is_rejected_in_agent_mode() {
    let output = agent_command()
        .args(["fetch", "alice.bsky.social"])
        .output()
        .expect("run bce fetch in agent mode");

    assert!(!output.status.success(), "stdout: {}", stdout(&output));

    let stderr = stderr(&output);
    assert!(
        stderr.contains("unrecognized subcommand 'fetch'"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("query"), "stderr: {stderr}");
}
