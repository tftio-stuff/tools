---
phase: 04-cli-surface
verified: 2026-03-22T18:30:00Z
status: passed
score: 12/12 must-haves verified
re_verification: false
gaps: []
human_verification:
  - test: "Run bce with a valid BSKY_APP_PASSWORD against a real handle"
    expected: "Spinner displays during extraction, then summary line shows 'Extracted N posts for handle to path (X new, Y existing)'"
    why_human: "Requires live BlueSky credentials and network; cannot verify spinner TTY rendering or actual extraction loop programmatically"
  - test: "Run bce with --since '3 months ago' and verify only recent posts appear"
    expected: "Date parsing succeeds; extraction stops at cutoff; count matches actual recent posts"
    why_human: "Requires live data to validate since-cutoff behavior end-to-end"
---

# Phase 4: CLI Surface Verification Report

**Phase Goal:** The extraction engine is usable as a first-class CLI tool following workspace conventions
**Verified:** 2026-03-22T18:30:00Z
**Status:** PASSED
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                            | Status     | Evidence                                                                      |
|----|---------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------|
| 1  | FetchSummary reports separate new_count and existing_count                       | VERIFIED   | models.rs lines 81-83: pub new_count: u64, pub existing_count: u64            |
| 2  | fetch_all_posts accepts optional progress callback and invokes it after each record | VERIFIED | client.rs line 233: on_progress: Option<&dyn Fn(u64)>; invoked at line 306-308 |
| 3  | run_extraction accepts optional progress callback and passes it through          | VERIFIED   | lib.rs line 34: on_progress: Option<&dyn Fn(u64)>; passed to fetch_all_posts at line 59 |
| 4  | Existing unit tests still pass after API changes                                 | VERIFIED   | cargo test: 30 lib tests + 2 binary tests, all pass                           |
| 5  | bce binary is runnable and accepts handle as positional argument                 | VERIFIED   | bce --help shows HANDLE as positional; bce binary builds and runs             |
| 6  | --db flag overrides XDG default database path                                    | VERIFIED   | cli.rs line 28: pub db: Option<PathBuf>; main.rs lines 38-41: match cli.db   |
| 7  | --since flag passes parsed date to run_extraction                                | VERIFIED   | main.rs lines 49-56: dateparser::parse_with_timezone wired to since param     |
| 8  | -q flag suppresses the progress spinner                                          | VERIFIED   | main.rs line 121: if quiet return None; test_make_spinner_quiet passes        |
| 9  | Missing BSKY_APP_PASSWORD shows clear error with setup instructions              | VERIFIED   | main.rs lines 30-35: bail! with app-passwords URL; verified via live run      |
| 10 | Completion summary line shows new and existing counts                            | VERIFIED   | main.rs lines 92-99: println! includes summary.new_count, summary.existing_count |
| 11 | First run creates parent directories for the database automatically              | VERIFIED   | main.rs lines 44-46: create_dir_all on db_path.parent()                      |
| 12 | Binary compiles and passes clippy with workspace pedantic lints                  | VERIFIED   | cargo build exits 0; cargo clippy -D warnings exits 0                        |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact                                          | Expected                                               | Status     | Details                                                                   |
|---------------------------------------------------|--------------------------------------------------------|------------|---------------------------------------------------------------------------|
| crates/bsky-comment-extractor/src/models.rs       | FetchSummary with new_count and existing_count         | VERIFIED   | Lines 81-83 contain both fields with u64 type and doc comments            |
| crates/bsky-comment-extractor/src/db.rs           | upsert_post returns Result<bool, ExtractorError>       | VERIFIED   | Line 74: returns Result<bool, ExtractorError>; true=new, false=existing   |
| crates/bsky-comment-extractor/src/client.rs       | fetch_all_posts with progress callback parameter       | VERIFIED   | Line 233: on_progress: Option<&dyn Fn(u64)>; invoked lines 306-308       |
| crates/bsky-comment-extractor/src/lib.rs          | run_extraction with progress callback parameter        | VERIFIED   | Line 34: on_progress: Option<&dyn Fn(u64)>; passed through at line 59    |
| crates/bsky-comment-extractor/src/cli.rs          | Clap Parser struct with handle, --db, --since, --quiet | VERIFIED   | pub struct Cli with all 4 fields; 4 unit tests pass                       |
| crates/bsky-comment-extractor/src/main.rs         | Binary entry point with sync main, tokio runtime, spinner, summary line | VERIFIED | fn main(), Cli::parse(), RuntimeBuilder, ProgressBar, summary println   |
| crates/bsky-comment-extractor/Cargo.toml          | Binary target and CLI dependencies                     | VERIFIED   | [[bin]] name="bce"; all 7 CLI deps present (anyhow, clap, dateparser, directories, indicatif, is-terminal, tftio-cli-common) |

### Key Link Verification

| From      | To           | Via                                              | Status   | Evidence                                                              |
|-----------|--------------|--------------------------------------------------|----------|-----------------------------------------------------------------------|
| client.rs | db.rs        | db_has_uri check before upsert to distinguish new vs existing | VERIFIED | db.rs line 75: let is_new = !db_has_uri(conn, uri)?; returned from upsert_post |
| lib.rs    | client.rs    | run_extraction passes progress callback to fetch_all_posts | VERIFIED | lib.rs line 59: fetch_all_posts(&did, since, &conn, on_progress)      |
| main.rs   | lib.rs       | run_extraction call with progress callback        | VERIFIED | main.rs lines 77-82: runtime.block_on(bsky_comment_extractor::run_extraction(..., Some(&progress_cb))) |
| main.rs   | cli.rs       | Cli::parse() for argument handling                | VERIFIED | main.rs line 13: let cli = Cli::parse()                               |
| main.rs   | indicatif    | ProgressBar::new_spinner with TTY guard and quiet flag | VERIFIED | main.rs lines 124-133: ProgressBar::with_draw_target + tftio_cli_common::output::is_tty() |
| main.rs   | filesystem   | create_dir_all before opening database            | VERIFIED | main.rs lines 44-46: create_dir_all called before run_extraction      |

### Requirements Coverage

| Requirement | Source Plan | Description                                      | Status    | Evidence                                                              |
|-------------|------------|--------------------------------------------------|-----------|-----------------------------------------------------------------------|
| CLI-01      | 04-02       | Accept user handle or DID as positional argument | SATISFIED | cli.rs: pub handle: String (positional clap arg); --help shows <HANDLE> |
| CLI-02      | 04-02       | --db flag for database path                      | SATISFIED | cli.rs: pub db: Option<PathBuf>; main.rs: db_path resolution + create_dir_all |
| CLI-03      | 04-01, 04-02 | Progress indicator during extraction             | SATISFIED | on_progress callback in client/lib; ProgressBar spinner in main.rs   |
| CLI-04      | 04-01, 04-02 | Follow workspace conventions                     | SATISFIED | clap Parser, sync main + RuntimeBuilder, tftio-cli-common::output::is_tty, clippy clean |

No orphaned requirements: REQUIREMENTS.md maps CLI-01 through CLI-04 to Phase 4, all four accounted for across the two plans.

### Anti-Patterns Found

No anti-patterns detected in the modified files. Scanned for:
- TODO/FIXME/HACK/PLACEHOLDER comments: none found
- Empty implementations (return null/return {}): none found
- Stub handlers: none found
- console.log / eprintln-only implementations: none found

The `#[allow(clippy::future_not_send)]` attributes are intentional and documented -- the functions are single-threaded by design due to rusqlite::Connection not being Send.

### Human Verification Required

#### 1. Live extraction with credentials

**Test:** Set BSKY_APP_PASSWORD to a valid app password and run `bce alice.bsky.social` (or another real handle).
**Expected:** Spinner appears on stderr during extraction; after completion, prints "Extracted N posts for alice.bsky.social to /path/bsky-posts.db (X new, Y existing)".
**Why human:** Requires live BlueSky credentials and network. Cannot verify spinner TTY rendering or actual extraction loop behavior programmatically.

#### 2. --since flag with live data

**Test:** Run `bce alice.bsky.social --since '3 months ago'` with valid credentials.
**Expected:** Date parses without error; extraction stops at the cutoff; reported count matches only posts from the last 3 months.
**Why human:** Since-cutoff logic depends on real post timestamps. The code path is wired correctly but end-to-end correctness requires live data.

### Gaps Summary

No gaps. All 12 truths verified, all 7 artifacts confirmed substantive and wired, all 4 key links confirmed, all 4 requirement IDs satisfied. The `bce` binary builds, passes clippy, and all 32 tests pass (30 lib + 2 binary).

The one note from the summary -- that `just ci` fails workspace-wide due to pre-existing rustfmt violations in unrelated crates (`gator`, `silent-critic`, `prompter`) -- is out of scope for this phase. The bce crate itself is CI-clean.

---

_Verified: 2026-03-22T18:30:00Z_
_Verifier: Claude (gsd-verifier)_
