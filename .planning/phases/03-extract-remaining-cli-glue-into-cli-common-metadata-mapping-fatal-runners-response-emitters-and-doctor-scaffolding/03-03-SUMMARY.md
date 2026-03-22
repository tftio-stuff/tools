---
phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding
plan: 03
subsystem: cli
tags: [rust, clap, cli-common, doctor, runner, json]
requires:
  - phase: 02-maximize-cli-common-sharing
    provides: richer shared command, response, and doctor primitives used by the remaining tools
provides:
  - lazy shared JSON/text response emission for richer command outputs
  - shared fatal runner handling for displayable top-level CLI errors
  - prompter doctor construction routed through shared doctor report helpers
affects:
  - 03-04-PLAN.md
  - cli-common boundary enforcement
  - workspace CLI consistency checks
tech-stack:
  added: []
  patterns:
    - lazy shared response builders via `render_response_parts`
    - shared top-level fatal handling via `run_with_display_error_handler`
    - shared doctor report construction via `DoctorReport::for_tool` and `DoctorReport::emit`
key-files:
  created: []
  modified:
    - crates/cli-common/src/json.rs
    - crates/cli-common/src/lib.rs
    - crates/cli-common/src/runner.rs
    - crates/silent-critic/src/main.rs
    - crates/unvenv/src/main.rs
    - crates/asana-cli/src/main.rs
    - crates/prompter/src/doctor.rs
key-decisions:
  - "Shared response plumbing stays infrastructure-only: `render_response_parts` owns JSON/text branching while `silent-critic` keeps command-specific summaries."
  - "Displayable top-level CLI failures use one shared fatal runner helper so tools can keep local parsing and tracing behavior without duplicating exit-path rendering."
  - "Prompter doctor state collection stays local, but report scaffolding and emission now come from shared `DoctorReport` helpers."
patterns-established:
  - "Lazy response emission: build JSON data and text summaries only on the selected output path."
  - "Thin main entrypoints: use shared fatal handlers instead of tool-local `match` blocks for terminal error exits."
requirements-completed: [CLI-SHARE-05, CLI-SHARE-06]
duration: 3m22s
completed: 2026-03-22
---

# Phase 03 Plan 03: Richer CLI glue migration Summary

**`silent-critic`, `unvenv`, `asana-cli`, and `prompter` now keep their domain behavior while shared response emission, fatal exit handling, and doctor report scaffolding live in `tftio-cli-common`.**

## Performance

- **Duration:** 3m22s
- **Started:** 2026-03-22T22:34:51Z
- **Completed:** 2026-03-22T22:38:13Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added a lazy shared response helper so richer `silent-critic` commands can drop repeated JSON-vs-text branching without moving domain formatting into `cli-common`.
- Added a shared fatal runner helper and moved the `unvenv` and `asana-cli` main entrypoints onto it.
- Rebuilt `prompter` doctor reporting from the shared `DoctorReport` builder and emitter while keeping local config/state checks.

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace reusable success-path branching in `silent-critic`** - `c0a51fe` (refactor)
2. **Task 2: Replace remaining fatal-runner and doctor-provider glue in `unvenv`, `asana-cli`, and `prompter`** - `0302c31` (refactor)

**Plan metadata:** Pending final docs commit for this summary and planning-state updates.

## Files Created/Modified
- `crates/cli-common/src/json.rs` - Added `render_response_parts` and tests for lazy data/text builders.
- `crates/cli-common/src/lib.rs` - Re-exported the new shared response and runner helpers.
- `crates/cli-common/src/runner.rs` - Added `run_with_display_error_handler` with unit coverage.
- `crates/silent-critic/src/main.rs` - Routed richer command outputs through shared lazy response emission.
- `crates/unvenv/src/main.rs` - Replaced the local fatal `match` block with shared fatal handling.
- `crates/asana-cli/src/main.rs` - Replaced duplicated top-level fatal printing with the shared fatal runner while preserving tracing.
- `crates/prompter/src/doctor.rs` - Built and emitted doctor reports through shared doctor helpers.

## Decisions Made
- Used a reusable lazy response helper instead of more `silent-critic`-specific wrappers so other tools can skip building unused JSON/text payloads.
- Kept `unvenv` scan behavior and `asana-cli` tracing local; only the fatal error conversion and exit handling moved into `cli-common`.
- Kept `prompter` doctor state collection local because its config/library checks are tool-specific, but removed local report scaffolding.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Re-exported the new shared response helper**
- **Found during:** Task 1 (Replace reusable success-path branching in `silent-critic`)
- **Issue:** `silent-critic` could not import `render_response_parts` until `tftio-cli-common` re-exported it from `src/lib.rs`.
- **Fix:** Added the missing public re-export alongside the existing JSON helpers.
- **Files modified:** `crates/cli-common/src/lib.rs`
- **Verification:** `cargo test -p tftio-silent-critic && cargo clippy -p tftio-silent-critic -- -D warnings`
- **Committed in:** `c0a51fe` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to complete the planned migration. No scope expansion.

## Issues Encountered
- The first `silent-critic` compile pass failed because the new helper existed only inside `cli-common::json`; exporting it from the crate root resolved the integration point.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The remaining Phase 03 work can focus on enforcement, documentation, and full-suite validation against the thinner boundary.
- No functional blockers remain from this plan’s migrations.

## Self-Check: PASSED

- Confirmed `.planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-03-SUMMARY.md` exists.
- Confirmed task commits `c0a51fe` and `0302c31` exist in `git log --oneline --all`.

---
*Phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding*
*Completed: 2026-03-22*
