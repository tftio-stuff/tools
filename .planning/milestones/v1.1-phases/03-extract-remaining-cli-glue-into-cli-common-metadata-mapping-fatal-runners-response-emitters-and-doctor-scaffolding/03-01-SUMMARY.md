---
phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding
plan: 01
subsystem: cli
tags: [rust, cli-common, clap, json, doctor]
requires:
  - phase: 02-maximize-cli-common-sharing
    provides: shared adapters, response envelopes, and structured doctor reports
provides:
  - metadata-command mapping helpers for crate-local enums
  - shared fatal runner helpers for parse/run/exit plumbing
  - lazy success-response rendering for JSON vs text output
  - doctor report scaffolding from shared tool metadata
affects: [03-02, 03-03, cli-common, workspace-tools]
tech-stack:
  added: []
  patterns:
    - metadata mapping via StandardCommandMap plus impl_standard_command_map!
    - fatal CLI handling via FatalCliError and parse_and_run
    - lazy text rendering via render_response_with
    - doctor report scaffolding via DoctorReport::for_tool and emit
key-files:
  created:
    - crates/cli-common/src/command.rs
    - crates/cli-common/src/error.rs
    - crates/cli-common/src/json.rs
    - crates/cli-common/src/runner.rs
  modified:
    - crates/cli-common/src/doctor.rs
    - crates/cli-common/src/lib.rs
key-decisions:
  - "Metadata helpers support global JSON flags, version-local JSON flags, and doctor/update variants through one shared mapping trait and macro."
  - "Fatal CLI handling stays closure-based so tools can keep their clap layouts while centralizing error printing and exit-code behavior."
  - "Response and doctor helpers stay infrastructure-only: text formatting remains caller-owned while cli-common owns the JSON/text branch and doctor scaffolding."
patterns-established:
  - "StandardCommandMap: local metadata enums convert into StandardCommand without repeating match logic in main.rs files."
  - "FatalCliError runner flow: parse_and_run and run_with_fatal_handler turn fallible CLI closures into shared exit semantics."
  - "Doctor scaffolding: DoctorReport::for_tool preloads header, version, and checks from DoctorChecks before tool-specific details are appended."
requirements-completed: [CLI-SHARE-05, CLI-SHARE-06]
duration: 3 min
completed: 2026-03-22
---

# Phase 03 Plan 01: Shared cli-common helper layer Summary

Reusable metadata mapping, fatal runner handling, lazy success rendering, and doctor report scaffolding now live in `tftio-cli-common`.

## Execution

- **Start:** 2026-03-22T22:14:52Z
- **End:** 2026-03-22T22:18:41Z
- **Tasks completed:** 2
- **Files changed:** 6

## Task Results

### Task 1: Add shared metadata-mapping and runner helpers to `cli-common`

- Added `StandardCommandMap`, `map_standard_command`, and optional runner helpers for mapped metadata commands.
- Added `impl_standard_command_map!` to cover the metadata enum shapes already present across workspace tools.
- Added `FatalCliError`, `run_with_fatal_handler`, `parse_and_run`, and `parse_and_exit`.
- Routed `print_error` through the shared fatal error type.

**Commits**
- `250e68a` — `test(03-01): add failing tests for cli-common helper layer`
- `07bac72` — `feat(03-01): add shared metadata and runner helpers`

### Task 2: Add richer shared response emission and doctor-provider scaffolding

- Added `render_response_with` so callers can defer text construction until plain-text mode is selected.
- Made `render_response` delegate through the lazy helper.
- Added `DoctorReport::for_tool`, `DoctorReport::with_tool_header`, and `DoctorReport::emit`.
- Switched shared doctor report construction to the new tool-scaffold path.

**Commits**
- `4a802cd` — `test(03-01): add failing tests for response and doctor helpers`
- `f1b2543` — `feat(03-01): add shared response and doctor builders`

## Verification

- `cargo test -p tftio-cli-common`
- `cargo clippy -p tftio-cli-common -- -D warnings`

Both commands exited successfully after the GREEN commits.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None.

## Next Phase Readiness

Plan `03-02-PLAN.md` can now migrate workspace tools onto the new metadata, runner, response, and doctor helper surface.

## Self-Check: PASSED

Verified the summary file exists on disk and all four task commits are present in git history.
