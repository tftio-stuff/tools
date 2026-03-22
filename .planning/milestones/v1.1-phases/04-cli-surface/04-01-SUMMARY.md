---
phase: 04-cli-surface
plan: 01
subsystem: api
tags: [rust, bsky, sqlite, progress-callback, fetch-summary]

# Dependency graph
requires:
  - phase: 03-extraction-engine
    provides: BskyClient, fetch_all_posts, run_extraction, FetchSummary, upsert_post, db_has_uri
provides:
  - FetchSummary with new_count and existing_count fields
  - upsert_post returns Result<bool> (true=new, false=existing)
  - fetch_all_posts with Option<&dyn Fn(u64)> on_progress callback
  - run_extraction with Option<&dyn Fn(u64)> on_progress callback
affects: [04-02-cli-binary, future-callers-of-run-extraction]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Progress callback via Option<&dyn Fn(u64)> -- non-Send safe for single-threaded async"
    - "upsert returns bool: db_has_uri check before INSERT OR REPLACE distinguishes new vs existing"

key-files:
  created: []
  modified:
    - crates/bsky-comment-extractor/src/models.rs
    - crates/bsky-comment-extractor/src/db.rs
    - crates/bsky-comment-extractor/src/client.rs
    - crates/bsky-comment-extractor/src/lib.rs

key-decisions:
  - "on_progress uses Option<&dyn Fn(u64)> (not a type alias) -- keeps signature explicit and non-Send compatible"
  - "upsert_post checks db_has_uri before INSERT OR REPLACE rather than relying on conn.changes() -- semantically clearer"

patterns-established:
  - "Progress callbacks: Option<&dyn Fn(u64)> ref pattern for non-Send single-threaded async contexts"

requirements-completed: [CLI-03, CLI-04]

# Metrics
duration: 4min
completed: 2026-03-22
---

# Phase 4 Plan 1: CLI Surface API Extensions Summary

**Library API extended with new/existing count tracking in FetchSummary and optional progress callbacks on fetch_all_posts/run_extraction, enabling the CLI spinner and completion summary.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-03-22T17:43:44Z
- **Completed:** 2026-03-22T17:47:06Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- FetchSummary now carries `new_count` and `existing_count` alongside the existing `count` and `done` fields
- `upsert_post` returns `Result<bool, ExtractorError>` (true=new insert, false=existing update) using a pre-check via `db_has_uri`
- `fetch_all_posts` and `run_extraction` both accept `Option<&dyn Fn(u64)>` progress callbacks, invoked with running total after each record
- All 26 existing tests pass; 2 new tests added for upsert return value semantics

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend FetchSummary and track new vs existing counts** - `3bd3058` (feat)
2. **Task 2: Add progress callback to fetch_all_posts and run_extraction** - `7660a50` (feat)

**Plan metadata:** (docs commit below)

_Note: Task 1 was TDD (RED then GREEN); Task 2 was auto._

## Files Created/Modified
- `crates/bsky-comment-extractor/src/models.rs` - Added `new_count: u64` and `existing_count: u64` to FetchSummary
- `crates/bsky-comment-extractor/src/db.rs` - Changed `upsert_post` return type to `Result<bool, ExtractorError>`; added 2 new unit tests
- `crates/bsky-comment-extractor/src/client.rs` - Added `on_progress` param, new/existing counters, callback invocation; updated all 3 FetchSummary return sites; added 1 new compile test
- `crates/bsky-comment-extractor/src/lib.rs` - Added `on_progress` param to `run_extraction`; passed through to `fetch_all_posts`

## Decisions Made
- `on_progress: Option<&dyn Fn(u64)>` (reference to trait object, not boxed) -- avoids heap allocation, compatible with non-Send single-threaded async runtime; matches the `#[allow(clippy::future_not_send)]` pattern already established in Phase 03
- `upsert_post` uses `db_has_uri` pre-check rather than `conn.changes()` -- semantically unambiguous; `changes()` semantics with `INSERT OR REPLACE` can be surprising (counts as 2 changes: delete + insert)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed pre-existing clippy::option_if_let_else in lib.rs**
- **Found during:** Task 2 (adding on_progress to run_extraction)
- **Issue:** `if let Some(ref pw) = password { BskyClient::new(...) } else { BskyClient::new(None) }` triggered `clippy::option_if_let_else` which causes `-D warnings` build failure
- **Fix:** Rewrote as `password.as_ref().map_or_else(|| BskyClient::new(None), |pw| BskyClient::new(Some((handle, pw.as_str()))))`
- **Files modified:** crates/bsky-comment-extractor/src/lib.rs
- **Verification:** `cargo clippy -p tftio-bsky-comment-extractor -- -D warnings` exits 0
- **Committed in:** 7660a50 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - pre-existing clippy bug)
**Impact on plan:** Fix was necessary for the plan's own acceptance criterion (`cargo clippy -- -D warnings` exits 0). No scope creep.

## Issues Encountered
- Stale `/tmp/cc-wrapper` Nix linker wrapper was missing, causing initial compile failures. Created a minimal `/tmp/cc-wrapper` wrapper script pointing to `/usr/bin/clang` to restore builds. This is an environment issue unrelated to code changes.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Library API is ready for Plan 02 (CLI binary)
- `run_extraction` signature updated -- Plan 02 caller must pass `on_progress: Some(&|n| spinner.set_message(...))` or `None`
- FetchSummary now provides `new_count`/`existing_count` for the CLI summary line: "Fetched N posts (X new, Y updated)"

---
*Phase: 04-cli-surface*
*Completed: 2026-03-22*

## Self-Check: PASSED

- models.rs contains `pub new_count: u64` -- FOUND
- models.rs contains `pub existing_count: u64` -- FOUND
- db.rs `upsert_post` returns `Result<bool, ExtractorError>` -- FOUND
- client.rs contains `let is_new = crate::db::upsert_post` -- FOUND
- client.rs `fetch_all_posts` has `on_progress: Option<&dyn Fn(u64)>` -- FOUND
- lib.rs `run_extraction` has `on_progress: Option<&dyn Fn(u64)>` -- FOUND
- Task 1 commit 3bd3058 -- FOUND
- Task 2 commit 7660a50 -- FOUND
