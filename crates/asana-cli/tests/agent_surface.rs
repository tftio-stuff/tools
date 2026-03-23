//! Agent-mode surface smoke tests for `asana-cli`.

use std::path::PathBuf;
use std::process::{Command, Output};

const AGENT_TOKEN_ENV: &str = "TFTIO_AGENT_TOKEN";
const AGENT_TOKEN_EXPECTED_ENV: &str = "TFTIO_AGENT_TOKEN_EXPECTED";

fn bin_path() -> String {
    if let Some(value) = std::env::vars().find_map(|(key, value)| {
        if key.starts_with("CARGO_BIN_EXE_asana") {
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
        "asana-cli.exe"
    } else {
        "asana-cli"
    });

    if path.exists() {
        return path.to_string_lossy().into_owned();
    }

    panic!("asana-cli binary path environment variable not set");
}

fn run_agent_command(args: &[&str]) -> Output {
    Command::new(bin_path())
        .args(args)
        .env(AGENT_TOKEN_ENV, "shared-test-token")
        .env(AGENT_TOKEN_EXPECTED_ENV, "shared-test-token")
        .output()
        .expect("failed to execute asana-cli binary")
}

#[test]
fn agent_surface_help_lists_only_declared_capabilities() {
    let output = run_agent_command(&["--agent-help"]);
    assert!(
        output.status.success(),
        "agent help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for capability in [
        "manage-config",
        "manage-tasks",
        "manage-projects",
        "manage-sections",
        "manage-tags",
        "manage-custom-fields",
        "manage-workspaces",
        "manage-users",
    ] {
        assert!(
            stdout.contains(capability),
            "missing capability {capability} from output: {stdout}"
        );
    }

    for hidden in [
        "- doctor",
        "- update",
        "- manpage",
        "- completions",
        "- version",
        "- license",
        "command doctor",
        "command update",
        "command manpage",
        "command completions",
        "command version",
        "command license",
    ] {
        assert!(
            !stdout.contains(hidden),
            "unexpected hidden entry {hidden} in output: {stdout}"
        );
    }
}

#[test]
fn agent_surface_skill_manage_tasks_is_capability_scoped() {
    let output = run_agent_command(&["--agent-skill", "manage-tasks"]);
    assert!(
        output.status.success(),
        "agent skill failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("manage-tasks"));
    assert!(stdout.contains("- task"));
    assert!(
        !stdout.contains("manage-projects"),
        "unexpected extra capability: {stdout}"
    );
    assert!(
        !stdout.contains("- project"),
        "unexpected extra command path: {stdout}"
    );
}

#[test]
fn agent_surface_hidden_top_level_commands_parse_as_nonexistent_in_agent_mode() {
    for args in [&["doctor"][..], &["update"][..]] {
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
