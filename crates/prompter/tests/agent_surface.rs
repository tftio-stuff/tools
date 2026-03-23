//! Integration tests for the restricted `prompter` agent surface.

use std::process::{Command, Output};

const TEST_TOKEN: &str = "phase7-test-token";

fn prompter_bin() -> &'static str {
    env!("CARGO_BIN_EXE_prompter")
}

fn agent_command(args: &[&str]) -> Command {
    let mut command = Command::new(prompter_bin());
    command.args(args);
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
fn agent_surface_help_lists_only_visible_capabilities() {
    let output = agent_command(&["--agent-help"])
        .output()
        .expect("run prompter --agent-help");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    for capability in [
        "render-prompts",
        "list-profiles",
        "tree-profiles",
        "validate-profiles",
    ] {
        assert!(stdout.contains(capability), "stdout: {stdout}");
    }

    for hidden in ["doctor", "license", "version", "completions", "init"] {
        assert!(!stdout.contains(hidden), "stdout: {stdout}");
    }
}

#[test]
fn agent_surface_skill_renders_render_prompts_contract() {
    let output = agent_command(&["--agent-skill", "render-prompts"])
        .output()
        .expect("run prompter --agent-skill render-prompts");

    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let stdout = stdout(&output);
    assert!(
        stdout.contains("capability:\n- render-prompts"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("commands:\n- run"), "stdout: {stdout}");

    for hidden in ["doctor", "license", "version", "completions", "init"] {
        assert!(!stdout.contains(hidden), "stdout: {stdout}");
    }
}

#[test]
fn agent_surface_hides_privileged_commands() {
    for args in [
        &["doctor"][..],
        &["license"][..],
        &["version"][..],
        &["completions", "bash"][..],
        &["init"][..],
    ] {
        let output = agent_command(args).output().expect("run hidden command");

        assert!(!output.status.success(), "stdout: {}", stdout(&output));

        let stderr = stderr(&output);
        assert!(
            stderr.contains("unrecognized subcommand") || stderr.contains("invalid value"),
            "stderr: {stderr}"
        );
    }
}
