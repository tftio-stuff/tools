use std::process::Command;

fn todoer_bin() -> &'static str {
    env!("CARGO_BIN_EXE_todoer")
}

fn agent_command(args: &[&str]) -> Command {
    let mut command = Command::new(todoer_bin());
    command.args(args);
    command.env("TFTIO_AGENT_TOKEN", "phase7-test-token");
    command.env("TFTIO_AGENT_TOKEN_EXPECTED", "phase7-test-token");
    command
}

#[test]
fn agent_surface_help_lists_declared_todoer_capabilities() {
    let output = agent_command(&["--agent-help"])
        .output()
        .expect("run todoer --agent-help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    for capability in [
        "init-project",
        "create-task",
        "list-tasks",
        "task-status",
        "task-show",
        "task-note",
        "update-task-status",
    ] {
        assert!(stdout.contains(capability), "missing capability {capability}");
    }
}

#[test]
fn agent_surface_skill_lists_only_requested_capability_contract() {
    let output = agent_command(&["--agent-skill", "list-tasks"])
        .output()
        .expect("run todoer --agent-skill list-tasks");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("capability:\n- list-tasks"));
    assert!(stdout.contains("commands:\n- list"));
    assert!(!stdout.contains("task-show"));
    assert!(!stdout.contains("update-task-status"));
}

#[test]
fn agent_surface_rejects_hidden_meta_commands() {
    let output = agent_command(&["meta", "version"])
        .output()
        .expect("run todoer meta version");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand 'meta'"));
    assert!(!stderr.contains("version"));
    assert!(!stderr.contains("license"));
    assert!(!stderr.contains("completions"));
}
