//! Agent-mode surface smoke tests for `silent-critic`.

use std::path::PathBuf;
use std::process::{Command, Output};

const AGENT_TOKEN_ENV: &str = "TFTIO_AGENT_TOKEN";
const AGENT_TOKEN_EXPECTED_ENV: &str = "TFTIO_AGENT_TOKEN_EXPECTED";

fn bin_path() -> String {
    if let Some(value) = std::env::vars().find_map(|(key, value)| {
        if key.starts_with("CARGO_BIN_EXE_silent_critic")
            || key.starts_with("CARGO_BIN_EXE_silent-critic")
        {
            Some(value)
        } else {
            None
        }
    }) {
        return value;
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push(if cfg!(windows) {
        "silent-critic.exe"
    } else {
        "silent-critic"
    });

    if path.exists() {
        return path.to_string_lossy().into_owned();
    }

    panic!("silent-critic binary path environment variable not set");
}

fn run_agent_command(args: &[&str]) -> Output {
    Command::new(bin_path())
        .args(args)
        .env(AGENT_TOKEN_ENV, "shared-test-token")
        .env(AGENT_TOKEN_EXPECTED_ENV, "shared-test-token")
        .env_remove("SILENT_CRITIC_TOKEN")
        .output()
        .expect("failed to execute silent-critic binary")
}

#[test]
fn agent_surface_help_lists_only_worker_safe_capabilities() {
    let output = run_agent_command(&["--agent-help"]);
    assert!(
        output.status.success(),
        "agent help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for capability in ["session-status", "session-manifest", "session-submit"] {
        assert!(
            stdout.contains(capability),
            "missing capability {capability} from output: {stdout}"
        );
    }

    for hidden in [
        "- project",
        "- criterion",
        "- contract",
        "- decide",
        "- log",
        "command project",
        "command criterion",
        "command contract",
        "command decide",
        "command log",
        "command session new",
        "command session discover",
        "command session sandbox",
        "command session compose-from",
        "command session go",
    ] {
        assert!(
            !stdout.contains(hidden),
            "unexpected hidden entry {hidden} in output: {stdout}"
        );
    }
}

#[test]
fn agent_surface_skill_session_manifest_is_capability_scoped() {
    let output = run_agent_command(&["--agent-skill", "session-manifest"]);
    assert!(
        output.status.success(),
        "agent skill failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session-manifest"));
    assert!(stdout.contains("- session manifest"));
    assert!(
        !stdout.contains("session-status"),
        "unexpected extra capability: {stdout}"
    );
    assert!(
        !stdout.contains("- session status"),
        "unexpected extra command path: {stdout}"
    );
}

#[test]
fn agent_surface_hidden_higher_role_commands_parse_as_nonexistent() {
    let hidden_commands: &[&[&str]] = &[
        &["project", "init"],
        &["criterion", "create"],
        &["contract", "show", "contract-id"],
        &[
            "decide",
            "--contract",
            "contract-id",
            "--type",
            "accept",
            "--basis",
            "reason",
        ],
        &["log", "contract-id"],
        &["session", "new", "--worktree", "."],
        &["session", "discover"],
        &["session", "sandbox"],
        &["session", "compose-from"],
        &["session", "go"],
    ];

    for args in hidden_commands {
        let output = run_agent_command(args);
        assert!(
            !output.status.success(),
            "command should be hidden but succeeded: {}",
            args.join(" ")
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unrecognized subcommand"),
            "expected clap unknown-subcommand error for {}: {stderr}",
            args.join(" ")
        );
        assert!(
            !stderr.contains("Did you mean"),
            "hidden command error leaked suggestion for {}: {stderr}",
            args.join(" ")
        );
    }
}
