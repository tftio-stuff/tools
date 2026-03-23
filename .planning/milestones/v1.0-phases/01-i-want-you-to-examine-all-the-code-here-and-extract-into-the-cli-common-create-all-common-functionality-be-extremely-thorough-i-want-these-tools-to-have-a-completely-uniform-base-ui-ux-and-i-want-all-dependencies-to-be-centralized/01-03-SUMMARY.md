---
phase: 01-cli-common-unification
plan: 03
subsystem: cli
tags: [rust, clap, cli-common, unvenv, bce, asana-cli]
requires:
  - phase: 01-01
    provides: shared CLI metadata, command, error, and progress primitives
provides:
  - user-facing CLIs use shared metadata and error primitives
  - `bce` uses the shared spinner helper
  - `asana-cli` routes base commands through the shared standard dispatcher
affects: [01-04, unvenv, bsky-comment-extractor, asana-cli]
tech-stack:
  added: [clap_complete]
  patterns: [shared standard command dispatcher for user-facing CLIs]
key-files:
  created: []
  modified: [crates/unvenv/src/main.rs, crates/bsky-comment-extractor/src/cli.rs, crates/bsky-comment-extractor/src/main.rs, crates/asana-cli/src/cli/mod.rs, crates/asana-cli/src/main.rs, crates/asana-cli/tests/cli.rs]
key-decisions:
  - "`bce` preserves its positional extraction path while adding shared metadata subcommands."
  - "`asana-cli` now uses the shared update and doctor wiring from `cli-common`."
patterns-established:
  - "User-facing CLIs still preserve crate-specific invocation ergonomics while sharing base UX internals."
requirements-completed: [CLI-UNIFY-03]
duration: 7 min
completed: 2026-03-22
---

# Phase 1 Plan 03: User-Facing CLI Migration Summary

**`unvenv`, `bce`, and `asana-cli` now share the same metadata and error primitives while keeping their crate-specific invocation flows intact**

## Performance
- **Duration:** 7 min
- **Started:** 2026-03-22T20:08:44Z
- **Completed:** 2026-03-22T20:16:03Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Replaced `unvenv` inline metadata-command glue with `ToolSpec` and `StandardCommand` dispatch.
- Removed the private `bce` spinner constructor in favor of `cli-common::progress::make_spinner` and normalized its top-level errors.
- Routed `asana-cli` version, license, completions, doctor, and update commands through the shared dispatcher.

## Task Commits
No commits created yet during this execution segment.

## Files Created/Modified
- `crates/unvenv/src/main.rs` - Shared metadata-command dispatch.
- `crates/bsky-comment-extractor/src/cli.rs` - Added shared metadata subcommands without breaking `bce <handle>`.
- `crates/bsky-comment-extractor/src/main.rs` - Shared spinner and shared top-level error handling.
- `crates/asana-cli/src/cli/mod.rs` - Shared metadata-command dispatch with `ToolSpec`.
- `crates/asana-cli/src/main.rs` - Shared plain-text error rendering.
- `crates/asana-cli/tests/cli.rs` - Added completions smoke coverage.

## Decisions Made
- `bce` uses `subcommand_negates_reqs` so metadata commands and extraction mode can coexist without changing the primary extraction syntax.
- `asana-cli` keeps `Manpage` as crate-local behavior while delegating base commands to `cli-common`.

## Deviations from Plan
None - plan executed as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Plan 03 is verified. The remaining holdout is `prompter`, plus workspace-wide enforcement and documentation.
