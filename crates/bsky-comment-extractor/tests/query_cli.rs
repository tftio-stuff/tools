//! Integration tests for the `bce query` CLI contract.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

use bsky_comment_extractor::db::{init_db, open_db, upsert_post};

const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_bce")
}

fn seed_db(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_db(path)?;
    init_db(&conn)?;

    for (uri, created_at, text) in [
        (
            "at://did:plc:alice/app.bsky.feed.post/004",
            "2025-01-04T00:00:00Z",
            "post 004",
        ),
        (
            "at://did:plc:alice/app.bsky.feed.post/003",
            "2025-01-03T00:00:00Z",
            "post 003",
        ),
        (
            "at://did:plc:alice/app.bsky.feed.post/002",
            "2025-01-02T00:00:00Z",
            "post 002",
        ),
        (
            "at://did:plc:alice/app.bsky.feed.post/001",
            "2025-01-01T00:00:00Z",
            "post 001",
        ),
    ] {
        upsert_post(
            &conn,
            uri,
            "did:plc:alice",
            text,
            created_at,
            &format!(r#"{{"uri":"{uri}"}}"#),
        )?;
    }

    Ok(())
}

fn stdout_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

fn seeded_db() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("temp dir");
    let db_path = tmp.path().join("seeded.db");
    seed_db(&db_path).expect("seed database");
    (tmp, db_path)
}

fn parse_json(line: &str) -> Value {
    serde_json::from_str(line).expect("valid json")
}

#[test]
fn query_outputs_jsonl() {
    let (_tmp, db_path) = seeded_db();

    let output = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = stdout_lines(&output);
    assert_eq!(lines.len(), 5);

    let envelope = parse_json(&lines[0]);
    assert_eq!(envelope["total"], 4);
    assert_eq!(envelope["offset"], 0);
    assert_eq!(envelope["limit"], 50);
    assert_eq!(envelope["has_more"], false);

    let expected_keys = BTreeSet::from([
        "author_did".to_string(),
        "created_at".to_string(),
        "text".to_string(),
        "uri".to_string(),
    ]);

    for line in &lines[1..] {
        let post = parse_json(line);
        let keys = post
            .as_object()
            .expect("post object")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(keys, expected_keys);
    }
}

#[test]
fn query_limit_controls_page_size() {
    let (_tmp, db_path) = seeded_db();

    let output = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .args(["--limit", "2"])
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(output.status.success());

    let lines = stdout_lines(&output);
    assert_eq!(lines.len(), 3);

    let envelope = parse_json(&lines[0]);
    assert_eq!(envelope["limit"], 2);
    assert_eq!(envelope["has_more"], true);
}

#[test]
fn query_offset_skips_rows() {
    let (_tmp, db_path) = seeded_db();

    let output = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .args(["--limit", "2", "--offset", "1"])
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(output.status.success());

    let lines = stdout_lines(&output);
    assert_eq!(lines.len(), 3);

    let envelope = parse_json(&lines[0]);
    assert_eq!(envelope["offset"], 1);
    assert_eq!(envelope["limit"], 2);
    assert_eq!(envelope["has_more"], true);
    assert!(lines[1].contains("at://did:plc:alice/app.bsky.feed.post/003"));
    assert!(lines[2].contains("at://did:plc:alice/app.bsky.feed.post/002"));
}

#[test]
fn query_db_override_and_missing_db() {
    let (_tmp, db_path) = seeded_db();

    let success = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");
    assert!(success.status.success());

    let missing_path = db_path.with_file_name("missing.db");
    let failure = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&missing_path)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(!failure.status.success());
    let error = parse_json(
        String::from_utf8_lossy(&failure.stderr)
            .trim()
            .trim_end_matches('\n'),
    );
    assert_eq!(error["error"], "db_not_found");
}

#[test]
fn query_does_not_require_bsky_app_password() {
    let (_tmp, db_path) = seeded_db();

    let output = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
