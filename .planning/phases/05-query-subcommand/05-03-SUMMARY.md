---
phase: 05-query-subcommand
plan: 03
subsystem: cli
tags: [rust, clap, sqlite, jsonl, integration-tests]
requires:
  - phase: 05-01
    provides: query envelope/query post models and read-only SQLite pagination helpers
  - phase: 05-02
    provides: fetch/query subcommand parser with top-level agent-help flag
provides:
  - runtime dispatch for `bce fetch`, `bce query`, no-subcommand help, and `bce --agent-help`
  - envelope-first JSONL query output with structured JSON stderr failures
  - binary-level integration coverage for query pagination, db override, and no-password query mode
affects: [05-query-subcommand, 06-agent-help, bsky-comment-extractor, query-subcommand]
tech-stack:
  added: []
  patterns: [envelope-first jsonl stdout, structured stderr json errors, hidden global clap flag for subcommand help]
key-files:
  created: [crates/bsky-comment-extractor/tests/query_cli.rs, .planning/phases/05-query-subcommand/05-03-SUMMARY.md]
  modified:
    - crates/bsky-comment-extractor/src/main.rs
    - crates/bsky-comment-extractor/src/cli.rs
key-decisions:
  - "Query execution stays fully synchronous and never reads `BSKY_APP_PASSWORD` or starts a tokio runtime."
  - "Query failures emit structured stderr JSON with `db_not_found` for missing databases and `query_failed` for other runtime errors."
  - "The global `--agent-help` flag remains parseable but hidden from subcommand help so `bce query --help` exposes only query options."
patterns-established:
  - "Fetch/runtime concerns are split into `execute_fetch` and `execute_query` so networked and local code paths stay isolated."
  - "Query CLI integration tests seed real SQLite databases through crate DB helpers instead of mocking command output."
requirements-completed: [QUERY-01, QUERY-02, QUERY-03, QUERY-04, AGENT-02]
duration: 3m 23s
completed: 2026-03-22
---

# Phase 05 Plan 03: Query runtime and JSONL output Summary

**`bce query` now streams envelope-first JSONL from SQLite with deterministic pagination, structured stderr errors, and binary-level contract coverage while fetch keeps its existing spinner-based path**

## Performance

- **Duration:** 3m 23s
- **Started:** 2026-03-22T22:12:26Z
- **Completed:** 2026-03-22T22:15:55Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added `tests/query_cli.rs` with seeded-database integration tests for JSONL output, limit/offset paging, db override, missing-db JSON errors, and query mode without credentials.
- Refactored `main.rs` into explicit fetch and query execution paths, with envelope-first JSONL stdout and structured JSON stderr for query failures.
- Completed crate and workspace verification, including a targeted parser/help fix so `bce query --help` shows only `--db`, `--limit`, and `--offset`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing integration tests for the full query CLI contract** - `400d81e` (test)
2. **Task 2: Implement fetch/query runtime dispatch and JSONL query output in main.rs** - `1d3e521` (feat)
3. **Task 3: Run crate and workspace verification for the completed query subcommand** - `b5455a3` (fix)

## Files Created/Modified
- `crates/bsky-comment-extractor/tests/query_cli.rs` - Adds integration coverage for the end-to-end `bce query` stdout/stderr contract.
- `crates/bsky-comment-extractor/src/main.rs` - Splits fetch/query dispatch, prints top-level agent help/no-subcommand help, and streams query JSONL.
- `crates/bsky-comment-extractor/src/cli.rs` - Hides the global `--agent-help` flag from subcommand help while keeping it parseable at the top level.
- `.planning/phases/05-query-subcommand/05-03-SUMMARY.md` - Records execution, verification, and deviations for this plan.

## Decisions Made
- Kept fetch error printing as human-readable stderr while query mode uses structured JSON stderr so agent consumers can parse failures.
- Used `serde_json::to_writer` over locked stdout/stderr handles to stream query output and errors without building full buffers.
- Hid `--agent-help` from clap help output rather than changing runtime semantics, preserving `bce --agent-help` while satisfying the query help contract.

## Deviations from Plan

### Execution Deviations

1. Task 3 required a minimal approved scope expansion to `crates/bsky-comment-extractor/src/cli.rs` because clap’s global flag handling surfaced `--agent-help` in `bce query --help`, which violated the plan’s acceptance criteria. The fix hid the flag from help output and added a parser test without changing runtime behavior.

## Issues Encountered
- `git add` initially failed with a transient worktree index-lock error during Task 2 staging. A retry succeeded without manual cleanup.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 5 query behavior is complete and verified across crate-level and workspace-level test runs.
- Phase 6 can focus on expanding the `--agent-help` output into the full structured reference document without revisiting query runtime behavior.

## Self-Check: PASSED

- FOUND: `.planning/phases/05-query-subcommand/05-03-SUMMARY.md`
- FOUND: `400d81e`
- FOUND: `1d3e521`
- FOUND: `b5455a3`

---
*Phase: 05-query-subcommand*
*Completed: 2026-03-22*
