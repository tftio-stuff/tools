---
phase: 01-cli-common-unification
plan: 02
subsystem: cli
tags: [rust, clap, cli-common, todoer, silent-critic, gator]
requires:
  - phase: 01-01
    provides: shared CLI metadata, command, JSON, error, and progress primitives
provides:
  - machine-oriented CLIs use shared JSON envelopes
  - machine-oriented CLIs use shared top-level error rendering
  - gator, todoer, and silent-critic expose uniform metadata commands
affects: [01-04, gator, todoer, silent-critic]
tech-stack:
  added: [clap_complete, tftio-cli-common]
  patterns: [shared meta subcommand, shared print_error, shared ok_response/err_response]
key-files:
  created: []
  modified: [crates/gator/src/cli.rs, crates/gator/src/main.rs, crates/gator/src/lib.rs, crates/todoer/src/cli.rs, crates/todoer/src/main.rs, crates/todoer/src/output.rs, crates/todoer/tests/cli_parse.rs, crates/todoer/tests/json_output.rs, crates/silent-critic/src/cli.rs, crates/silent-critic/src/main.rs, crates/silent-critic/src/lib.rs]
key-decisions:
  - "All three machine-oriented CLIs use a `meta` command path for shared metadata commands."
  - "Shared JSON envelopes now come only from `cli-common`, not per-crate helper duplication."
patterns-established:
  - "Machine-facing CLIs should route top-level failures through `cli-common::error::print_error`."
requirements-completed: [CLI-UNIFY-02]
duration: 5 min
completed: 2026-03-22
---

# Phase 1 Plan 02: Machine-Oriented CLI Migration Summary

**`gator`, `todoer`, and `silent-critic` now share one metadata-command surface, one JSON envelope implementation, and one top-level error renderer**

## Performance
- **Duration:** 5 min
- **Started:** 2026-03-22T20:03:03Z
- **Completed:** 2026-03-22T20:08:44Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Removed duplicated JSON envelope helpers from `todoer` and `silent-critic`.
- Added shared `meta` command wiring to `gator`, `todoer`, and `silent-critic`.
- Normalized top-level machine-readable error output through `cli-common`.

## Task Commits
No commits created yet during this execution segment.

## Files Created/Modified
- `crates/todoer/src/main.rs` - Shared JSON and error helpers now come from `cli-common`.
- `crates/todoer/src/output.rs` - Reduced to crate-specific table rendering only.
- `crates/todoer/src/cli.rs` - Added shared metadata subcommands.
- `crates/silent-critic/src/main.rs` - Shared JSON and error helpers now come from `cli-common`.
- `crates/silent-critic/src/lib.rs` - Removed obsolete local output module export.
- `crates/gator/src/cli.rs` - Added optional metadata subcommand path while preserving existing agent invocation.
- `crates/gator/src/main.rs` - Shared metadata and error handling.

## Decisions Made
- `gator` keeps its positional agent invocation and adds metadata via an optional top-level subcommand field.
- `todoer` retains crate-specific table rendering while delegating JSON envelopes to `cli-common`.
- `silent-critic` removed its local output module entirely because the remaining functionality is shared.

## Deviations from Plan
None - plan executed as written.

## Issues Encountered
- `gator` needed additional clap and missing-docs fixes after adding the metadata subcommand path.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Plan 02 is verified. User-facing CLIs can now migrate onto the same shared base contract.
