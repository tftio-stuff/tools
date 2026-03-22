---
phase: 04-cli-surface
plan: 02
subsystem: bsky-comment-extractor/cli
tags: [cli, binary, clap, indicatif, xdg, tokio, bce]
dependency_graph:
  requires: ["04-01"]
  provides: ["bce binary", "CLI argument parsing", "spinner UX"]
  affects: ["crates/bsky-comment-extractor"]
tech_stack:
  added: [clap, dateparser, directories, indicatif, is-terminal, anyhow, tftio-cli-common]
  patterns: [sync-main-tokio-runtime, xdg-project-dirs, indicatif-stderr-spinner, progress-callback]
key_files:
  created:
    - crates/bsky-comment-extractor/src/cli.rs
    - crates/bsky-comment-extractor/src/main.rs
  modified:
    - crates/bsky-comment-extractor/Cargo.toml
    - crates/bsky-comment-extractor/src/lib.rs
    - crates/bsky-comment-extractor/src/client.rs
    - crates/bsky-comment-extractor/src/db.rs
decisions:
  - "make_spinner returns None when quiet=true OR stdout is not a TTY, matching workspace pattern"
  - "test_db_path_default checks path contains 'bce' and ends with 'bsky-posts.db' rather than asserting exact OS-specific path structure"
  - "Pre-existing rustfmt violations in client.rs and db.rs fixed as part of format commit to unblock just ci for this crate"
metrics:
  duration: "~3.5min"
  completed_date: "2026-03-22"
  tasks_completed: 3
  files_created: 2
  files_modified: 4
---

# Phase 04 Plan 02: CLI Binary Entry Point Summary

**One-liner:** `bce` binary wiring clap argument parsing, indicatif stderr spinner, XDG default db path, dateparser date parsing, and tokio runtime to the library's run_extraction function.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add CLI dependencies and clap Parser struct | 991215d | Cargo.toml, src/cli.rs, src/lib.rs |
| 2 | Create main.rs with tokio runtime, spinner, credential check | d0d3f04 | src/main.rs |
| 3 | Format fix and CI validation for bce crate | b5fd8e1 | src/client.rs, src/db.rs, src/lib.rs, src/main.rs |

## What Was Built

The `bce` binary (`crates/bsky-comment-extractor/src/main.rs`) wires together:

- **Credential gate:** Checks `BSKY_APP_PASSWORD` before any work; exits with clear setup message if missing.
- **DB path resolution:** `--db PATH` flag or XDG default via `ProjectDirs::from("com", "tftio", "bce")` -> `~/.local/share/bce/bsky-posts.db` on Linux/macOS.
- **Parent dir creation:** `create_dir_all` on db parent before opening, so first run always succeeds.
- **Date parsing:** `dateparser::parse_with_timezone` for `--since` with human-friendly formats.
- **Spinner UX:** `ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr())` on stderr; suppressed when `--quiet` or not a TTY.
- **Tokio runtime:** Manual `RuntimeBuilder::new_current_thread().enable_all().build()` (workspace pattern, no `#[tokio::main]`).
- **Summary line:** "Extracted N posts for handle to path (X new, Y existing)" using `FetchSummary.new_count` and `FetchSummary.existing_count` from plan 01.

### CLI struct (`src/cli.rs`)

```rust
pub struct Cli {
    pub handle: String,          // positional: alice.bsky.social or did:plc:abc123
    pub db: Option<PathBuf>,     // --db PATH
    pub since: Option<String>,   // --since DATE
    pub quiet: bool,             // -q / --quiet
}
```

## Verification

- `cargo run -p tftio-bsky-comment-extractor -- --help` shows all flags with app password setup help
- `cargo run -p tftio-bsky-comment-extractor -- alice.bsky.social` without env var: "Error: BSKY_APP_PASSWORD not set..."
- 32 tests pass (30 lib + 2 main binary tests)
- Format, clippy, release build all pass for the bce crate

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] test_db_path_default assertion wrong for macOS**
- **Found during:** Task 2 verification
- **Issue:** Test asserted `path.ends_with("bce/bsky-posts.db")` but on macOS `directories` crate returns `Application Support/com.tftio.bce/bsky-posts.db`
- **Fix:** Changed assertion to check `path_str.ends_with("bsky-posts.db")` and `path_str.contains("bce")` for cross-platform correctness
- **Files modified:** `crates/bsky-comment-extractor/src/main.rs`
- **Commit:** d0d3f04

**2. [Rule 3 - Blocking] Pre-existing rustfmt violations in bsky-comment-extractor crate**
- **Found during:** Task 3 (just ci)
- **Issue:** `client.rs` and `db.rs` from phase 03 had unformatted code that caused `cargo +nightly fmt --check` to fail for the whole crate, blocking CI validation of the new binary
- **Fix:** Applied `cargo +nightly fmt -p tftio-bsky-comment-extractor` to reformat the entire crate
- **Files modified:** `client.rs`, `db.rs`, `lib.rs`, `main.rs`
- **Commit:** b5fd8e1

### Out-of-Scope Issues (Logged, Not Fixed)

Pre-existing `rustfmt` violations in `gator`, `silent-critic`, and `prompter` crates cause `just ci` to fail workspace-wide. These are unrelated to the `bce` binary and are deferred.

## Self-Check: PASSED

- `crates/bsky-comment-extractor/src/cli.rs` - FOUND
- `crates/bsky-comment-extractor/src/main.rs` - FOUND
- Commit 991215d - FOUND
- Commit d0d3f04 - FOUND
- Commit b5fd8e1 - FOUND
