---
phase: 03-extraction-engine
plan: 01
subsystem: database
tags: [rust, rusqlite, sqlite, serde, thiserror, at-protocol, bluesky]

# Dependency graph
requires: []
provides:
  - tftio-bsky-comment-extractor workspace crate with Cargo.toml
  - ExtractorError enum (thiserror) with From impls for rusqlite, serde_json, io
  - AT Protocol serde structs: CreateSessionRequest/Response, ResolveHandleResponse, ListRecordsResponse, RepoRecord, PostValue, FetchSummary
  - SQLite storage layer: open_db (WAL + foreign keys), init_db (posts + extractions tables), upsert_post, db_has_uri, save_cursor, load_resume_cursor, complete_extraction
  - 9 unit tests covering schema creation, UPSERT idempotency, cursor round-trip, URI checks, and parent dir creation
affects: [03-02, 04-cli]

# Tech tracking
tech-stack:
  added: [tftio-bsky-comment-extractor crate (new), tempfile (dev-dep)]
  patterns:
    - open_db with create_dir_all + WAL pragma (modeled on silent-critic)
    - INSERT OR REPLACE for idempotent UPSERT on AT URI primary key
    - in-memory SQLite tests via Connection::open_in_memory()
    - missing_docs compliance with backtick-wrapped technical terms in doc comments

key-files:
  created:
    - crates/bsky-comment-extractor/Cargo.toml
    - crates/bsky-comment-extractor/src/lib.rs
    - crates/bsky-comment-extractor/src/error.rs
    - crates/bsky-comment-extractor/src/models.rs
    - crates/bsky-comment-extractor/src/db.rs
    - crates/bsky-comment-extractor/src/client.rs
  modified:
    - Cargo.toml (added crates/bsky-comment-extractor to members)
    - release-please-config.json (added bsky-comment-extractor package entry)

key-decisions:
  - "db.rs save_cursor uses UPDATE for existing incomplete extraction rows and INSERT for new ones (two-step SELECT + UPDATE/INSERT pattern, not INSERT OR REPLACE, because the extractions table has AUTOINCREMENT id)"
  - "u64 record_count stored via cast_signed() per clippy pedantic cast_possible_wrap lint requirement"
  - "doc_markdown lint: SQLite and BlueSky require backtick wrapping in doc comments under pedantic clippy"

patterns-established:
  - "Pattern: open_db creates parent dirs, enables WAL mode and foreign keys"
  - "Pattern: init_db uses execute_batch with CREATE TABLE IF NOT EXISTS for idempotent schema init"
  - "Pattern: UPSERT via INSERT OR REPLACE on TEXT PRIMARY KEY"
  - "Pattern: cursor persistence with incomplete-extraction row tracking via completed_at IS NULL filter"

requirements-completed: [STOR-01, STOR-02, STOR-03]

# Metrics
duration: 4min
completed: 2026-03-22
---

# Phase 03 Plan 01: Crate Scaffold and SQLite Storage Layer Summary

**`tftio-bsky-comment-extractor` workspace crate with AT Protocol serde structs, typed error hierarchy, and fully tested `SQLite` storage layer (UPSERT-idempotent posts, cursor-resumable extractions)**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-03-22T17:43:13Z
- **Completed:** 2026-03-22T17:47:00Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- New Rust library crate added to workspace with all AT Protocol model types as serde structs
- Complete `SQLite` DB layer: `open_db` (WAL + foreign keys + parent dir creation), `init_db` (posts + extractions tables + 3 indexes), `upsert_post` (INSERT OR REPLACE), `db_has_uri`, `save_cursor`, `load_resume_cursor`, `complete_extraction`
- 9 unit tests green: schema creation, UPSERT idempotency, cursor round-trip, missing cursor, URI presence, and nested-path parent dir creation

## Task Commits

Each task was committed atomically:

1. **Task 1: Create crate scaffold, error types, and AT Protocol models** - `79d17b1` (feat)
2. **Task 2: Implement SQLite storage layer with tested UPSERT and extraction metadata** - `269d609` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/bsky-comment-extractor/Cargo.toml` - Crate manifest with workspace deps (chrono, reqwest, rusqlite, serde, serde_json, thiserror, tokio:time, tracing) + tempfile dev-dep
- `crates/bsky-comment-extractor/src/lib.rs` - Crate root declaring pub mod client, db, error, models
- `crates/bsky-comment-extractor/src/error.rs` - ExtractorError enum with thiserror derives and From impls for rusqlite::Error, serde_json::Error, std::io::Error
- `crates/bsky-comment-extractor/src/models.rs` - AT Protocol serde structs: CreateSessionRequest/Response, ResolveHandleResponse, ListRecordsResponse, RepoRecord, PostValue, FetchSummary
- `crates/bsky-comment-extractor/src/db.rs` - Full SQLite layer with 9 unit tests
- `crates/bsky-comment-extractor/src/client.rs` - Stub for Plan 02 implementation
- `Cargo.toml` - Added crates/bsky-comment-extractor to workspace members
- `release-please-config.json` - Added tftio-bsky-comment-extractor package entry

## Decisions Made

- `save_cursor` uses a SELECT then UPDATE/INSERT pattern rather than INSERT OR REPLACE because the extractions table uses AUTOINCREMENT; INSERT OR REPLACE would delete and re-insert, losing the auto-generated id
- `u64` record_count is stored using `.cast_signed()` (not `as i64`) to satisfy the `cast_possible_wrap` clippy pedantic lint
- Technical nouns (SQLite, BlueSky) must be wrapped in backticks in doc comments to satisfy the `doc_markdown` clippy pedantic lint in this workspace

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed 7 clippy pedantic lint errors blocking compilation**
- **Found during:** Task 2 verification (clippy run)
- **Issue:** `doc_markdown` lint required backticks around `SQLite` and `BlueSky` in doc comments; `cast_possible_wrap` lint rejected `record_count as i64`
- **Fix:** Wrapped technical terms in backticks across lib.rs, db.rs, error.rs; replaced `as i64` with `.cast_signed()` in complete_extraction
- **Files modified:** crates/bsky-comment-extractor/src/lib.rs, src/db.rs, src/error.rs
- **Verification:** `cargo clippy -p tftio-bsky-comment-extractor` exits 0 with no warnings
- **Committed in:** `269d609` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 lint/bug)
**Impact on plan:** Fix was required for clippy compliance under workspace pedantic lints. No scope creep.

## Issues Encountered

- Environment has a stale `/tmp/cc-wrapper` Nix linker reference in `.cargo/config.toml`. Builds required `CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc` env var override. This is a pre-existing environment issue unrelated to this plan; not modified.

## Next Phase Readiness

- Plan 02 can build on stable `db.rs`, `error.rs`, and `models.rs` foundations
- `client.rs` stub is in place for Plan 02 to implement the AT Protocol HTTP client
- All public types are documented and pass clippy pedantic

## Self-Check: PASSED

- FOUND: crates/bsky-comment-extractor/Cargo.toml
- FOUND: crates/bsky-comment-extractor/src/lib.rs
- FOUND: crates/bsky-comment-extractor/src/error.rs
- FOUND: crates/bsky-comment-extractor/src/models.rs
- FOUND: crates/bsky-comment-extractor/src/db.rs
- FOUND: .planning/phases/03-extraction-engine/03-01-SUMMARY.md
- FOUND: commit 79d17b1 (Task 1)
- FOUND: commit 269d609 (Task 2)
