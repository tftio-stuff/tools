//! Integration tests for `bce` agent-facing documentation flags.

use std::process::{Command, Output};

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_bce")
}

fn run_bce(args: &[&str]) -> Output {
    Command::new(bin_path())
        .args(args)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run bce")
}

#[test]
fn top_level_agent_help_prints_exhaustive_yaml() {
    let output = run_bce(&["--agent-help"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version:"));
    assert!(stdout.contains("binary: \"bce\""));
    assert!(stdout.contains("name: \"fetch\""));
    assert!(stdout.contains("name: \"query\""));
    assert!(stdout.contains("BSKY_APP_PASSWORD"));
    assert!(stdout.contains("bsky-posts.db"));
    assert!(stdout.contains("has_more"));
    assert!(stdout.contains("db_not_found"));
    assert!(stdout.contains("App password missing"));
    assert!(stdout.contains("Running query without fetching first"));
}

#[test]
fn top_level_agent_skill_prints_skill_document() {
    let output = run_bce(&["--agent-skill"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("---\n"));
    assert!(stdout.contains("name: \"bce\""));
    assert!(stdout.contains("# BlueSky Comment Extractor"));
    assert!(stdout.contains("## Commands"));
    assert!(stdout.contains("`bce fetch <HANDLE>`"));
    assert!(stdout.contains("`bce query --limit 50 --offset 0`"));
    assert!(stdout.contains("BSKY_APP_PASSWORD"));
}

#[test]
fn subcommand_agent_help_does_not_trigger_top_level_doc_flow() {
    let output = run_bce(&["query", "--agent-help"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("schema_version:"));
    assert!(!stdout.contains("pending_phase_06"));
    assert!(stderr.contains("--agent-help") || stderr.contains("unexpected argument"));
}

#[test]
fn human_help_hides_agent_flags() {
    let output = run_bce(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("--agent-help"));
    assert!(!stdout.contains("--agent-skill"));
    assert!(stdout.contains("fetch"));
    assert!(stdout.contains("query"));
}
