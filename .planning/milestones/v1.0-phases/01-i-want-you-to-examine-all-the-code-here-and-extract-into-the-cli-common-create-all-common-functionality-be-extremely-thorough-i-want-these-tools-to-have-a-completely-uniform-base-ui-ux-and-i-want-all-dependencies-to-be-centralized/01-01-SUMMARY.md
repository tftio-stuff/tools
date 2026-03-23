---
phase: 01-cli-common-unification
plan: 01
subsystem: cli
tags: [rust, clap, serde_json, indicatif, cli-common]
requires: []
provides:
  - shared tool metadata contract in `cli-common`
  - shared standard command dispatcher for metadata commands
  - shared JSON success and error envelopes
  - shared error printer and spinner helper
affects: [01-02, 01-03, 01-04, cli-common, unvenv, todoer, silent-critic, bsky-comment-extractor]
tech-stack:
  added: [serde_json, indicatif]
  patterns: [shared ToolSpec metadata, shared StandardCommand dispatcher, shared JSON envelope, shared progress spinner]
key-files:
  created: [crates/cli-common/src/app.rs, crates/cli-common/src/command.rs, crates/cli-common/src/json.rs, crates/cli-common/src/error.rs, crates/cli-common/src/progress.rs]
  modified: [crates/cli-common/Cargo.toml, crates/cli-common/src/lib.rs, crates/cli-common/src/doctor.rs, crates/cli-common/src/output.rs]
key-decisions:
  - "`ToolSpec` owns the shared binary metadata contract for workspace CLIs."
  - "`StandardCommand` centralizes version, license, completions, doctor, and update dispatch in `cli-common`."
  - "JSON envelopes match the existing `todoer` and `silent-critic` response shape exactly."
  - "Spinner creation checks stderr TTY state and renders on stderr with the shared template."
patterns-established:
  - "New shared CLI UX belongs in `cli-common` before per-tool migrations."
  - "Base metadata commands should route through one dispatcher instead of per-binary match arms."
  - "Machine-readable CLI responses use the shared `ok_response` and `err_response` envelope helpers."
requirements-completed: [CLI-UNIFY-01]
duration: 4 min
completed: 2026-03-22
---

# Phase 1 Plan 01: CLI Common Foundation Summary

**`cli-common` now provides shared tool metadata, metadata-command dispatch, JSON envelopes, error rendering, and spinner construction for workspace CLIs**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-22T19:59:08Z
- **Completed:** 2026-03-22T20:03:03Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Added `ToolSpec` and `StandardCommand` so workspace binaries can converge on one metadata-command contract.
- Added shared JSON envelope helpers and a shared top-level error printer that matches the current `todoer` and `silent-critic` shape.
- Added shared spinner construction and stderr TTY detection so progress UX can converge on one implementation.

## Task Commits

No commits created yet during this execution segment.

## Files Created/Modified
- `crates/cli-common/src/app.rs` - Shared CLI tool metadata contract.
- `crates/cli-common/src/command.rs` - Shared standard command dispatcher.
- `crates/cli-common/src/json.rs` - Shared JSON success and error envelopes.
- `crates/cli-common/src/error.rs` - Shared plain-text and JSON error printer.
- `crates/cli-common/src/progress.rs` - Shared spinner constructor for interactive stderr progress.
- `crates/cli-common/src/lib.rs` - Module exports and public re-exports for the new shared surface.
- `crates/cli-common/src/doctor.rs` - Custom-header doctor entrypoint and shared header rendering.
- `crates/cli-common/src/output.rs` - Added stderr TTY detection alongside existing stdout TTY detection.
- `crates/cli-common/Cargo.toml` - Centralized `serde_json` and `indicatif` usage in `cli-common`.

## Decisions Made
- `ToolSpec` stores the shared bin metadata that later plan migrations can pass by reference.
- `run_standard_command` remains generic over both the clap command type and doctor provider so binaries can reuse it without changing their clap setup.
- `ok_response` and `err_response` preserve the exact current JSON object shape already used by `todoer` and `silent-critic`.
- `make_spinner` gates on stderr TTY state because the spinner draws on stderr, not stdout.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered
- `cargo clippy -p tftio-cli-common -- -D warnings` flagged missing `#[must_use]` annotations and `needless_pass_by_value` on the plan-specified JSON helper signatures. Added `#[must_use]` where required and a narrow allow on the two JSON helper functions to preserve the exact public API required by the plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
Plan 01 is verified. Wave 2 can now migrate consumer crates onto the shared `cli-common` surface.

---
*Phase: 01-cli-common-unification*
*Completed: 2026-03-22*
