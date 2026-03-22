---
phase: 05-query-subcommand
plan: 02
subsystem: cli
tags: [rust, clap, sqlite, subcommands]
requires:
  - phase: 04-cli-surface
    provides: existing bce clap parser structure and database flag conventions
provides:
  - top-level Cli parser with optional fetch/query subcommands
  - top-level --agent-help flag reserved for future runtime handling
  - query-only pagination flags with default limit 50 and offset 0
affects: [05-query-subcommand, 06-agent-help]
tech-stack:
  added: []
  patterns: [clap subcommands, global top-level flags, parser-first TDD]
key-files:
  created: [.planning/phases/05-query-subcommand/05-02-SUMMARY.md]
  modified: [crates/bsky-comment-extractor/src/cli.rs]
key-decisions:
  - "Kept Cli.command optional so `bce` can show help instead of failing parse with no subcommand."
  - "Reserved `--agent-help` as a global top-level flag instead of a subcommand to match the phase contract."
  - "Locked query parsing to only `--db`, `--limit`, and `--offset` with clap defaults 50 and 0."
patterns-established:
  - "Parser migrations start with failing clap parse tests that exercise the exact command surface."
  - "Top-level global flags live on Cli while subcommand-specific flags stay isolated in typed args structs."
requirements-completed: [QUERY-02, QUERY-03, QUERY-04]
duration: 2m
completed: 2026-03-22
---

# Phase 05 Plan 02: Query Subcommand Summary

**Clap parser now exposes explicit `fetch` and `query` subcommands with a top-level `--agent-help` flag and locked query pagination defaults**

## Performance

- **Duration:** 2m
- **Started:** 2026-03-22T18:05:52-04:00
- **Completed:** 2026-03-22T18:07:42-04:00
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Replaced flat parser coverage with failing tests for `fetch`, `query`, top-level `--agent-help`, and flat invocation rejection.
- Reworked `cli.rs` into a top-level parser with `Command`, `FetchArgs`, and `QueryArgs`.
- Locked query parsing to `--db`, `--limit`, and `--offset` with defaults `50` and `0`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace flat-parser tests with failing subcommand migration tests** - `9bc2523` (test)
2. **Task 2: Implement the subcommand parser and locked query flag surface** - `a126ab7` (feat)

## Files Created/Modified
- `crates/bsky-comment-extractor/src/cli.rs` - Defines the top-level clap parser, fetch/query subcommands, and parser tests for the migrated CLI surface.
- `.planning/phases/05-query-subcommand/05-02-SUMMARY.md` - Records plan execution details, decisions, and verification evidence.

## Decisions Made
- Kept the top-level `command` field optional so `bce` can defer no-subcommand behavior to runtime help handling.
- Reserved `--agent-help` on `Cli` with `global = true` so it parses before any subcommand-specific arguments.
- Split fetch and query args into separate structs so query rejects fetch-only flags like `--since`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Task 2 verification initially failed because `cargo test -p tftio-bsky-comment-extractor test_cli_parse_ --lib --verbose` compiles all lib tests, and Plan 05-01 symbols in `db.rs` and `models.rs` were not present yet. After those changes landed, the required verification command passed without further parser changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `main.rs` can now dispatch on `Command::Fetch` vs `Command::Query`.
- Phase 06 can reuse the top-level `agent_help` flag without further parser restructuring.

## Self-Check: PASSED

- FOUND: `.planning/phases/05-query-subcommand/05-02-SUMMARY.md`
- FOUND: `9bc2523`
- FOUND: `a126ab7`

---
*Phase: 05-query-subcommand*
*Completed: 2026-03-22*
