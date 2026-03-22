---
phase: 01-cli-common-unification
plan: 04
subsystem: cli
tags: [rust, cli-common, prompter, justfile, workspace]
requires:
  - phase: 01-01
    provides: shared CLI base primitives
  - phase: 01-02
    provides: machine-oriented CLI migration pattern
  - phase: 01-03
    provides: user-facing CLI migration pattern
provides:
  - `prompter` uses shared metadata and doctor scaffolding
  - workspace-wide CLI consistency recipe
  - explicit documentation of the `cli-common` boundary
affects: [workspace, CLAUDE.md, justfile, PROJECT.md]
tech-stack:
  added: []
  patterns: [workspace CLI consistency recipe, documented cli-common boundary]
key-files:
  created: []
  modified: [crates/prompter/src/lib.rs, crates/prompter/src/main.rs, crates/prompter/src/doctor.rs, crates/prompter/tests/cli.rs, justfile, CLAUDE.md, .planning/PROJECT.md]
key-decisions:
  - "`prompter` keeps its custom dynamic completion augmentation while sharing the base metadata and doctor plumbing."
  - "Workspace drift is enforced through `just cli-consistency`, `just test`, and `just lint`."
patterns-established:
  - "Base CLI UX changes must be validated at the workspace level, not only per crate."
requirements-completed: [CLI-UNIFY-04]
duration: 8 min
completed: 2026-03-22
---

# Phase 1 Plan 04: Prompter Migration and Workspace Enforcement Summary

**`prompter` now shares the base CLI contract with the rest of the workspace, and the workspace now enforces that contract through tooling and documentation**

## Performance
- **Duration:** 8 min
- **Started:** 2026-03-22T20:16:03Z
- **Completed:** 2026-03-22T20:16:03Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Replaced `prompter`'s manual version, license, and non-JSON doctor wiring with the shared base contract.
- Added `just cli-consistency` to verify `--help` and JSON/text base UX across all binaries.
- Documented the invariant that base CLI UX belongs in `cli-common` and updated project context to reflect the active unification work.

## Task Commits
No commits created yet during this execution segment.

## Files Created/Modified
- `crates/prompter/src/lib.rs` - Fixed clap help/version parsing to preserve successful `--help` behavior.
- `crates/prompter/src/main.rs` - Shared metadata and doctor dispatch.
- `crates/prompter/src/doctor.rs` - `DoctorChecks` implementation plus shared-state-backed JSON doctor output.
- `crates/prompter/tests/cli.rs` - Added JSON version and JSON doctor coverage.
- `justfile` - Added `cli-consistency` recipe.
- `CLAUDE.md` - Documented the `cli-common` ownership boundary.
- `.planning/PROJECT.md` - Updated current-state context for the active CLI unification phase.

## Decisions Made
- `prompter` preserves JSON doctor output by reusing shared doctor state while still implementing `DoctorChecks` for the shared plain-text path.
- The final workspace consistency gate intentionally checks both `--help` behavior and JSON envelope behavior.

## Deviations from Plan
- `prompter` still owns its dynamic completion augmentation logic. The base shared completion path remains documented rather than fully abstracted because the current `cli-common` completion helper writes directly to stdout and does not expose a reusable script buffer.

## Issues Encountered
- `just cli-consistency` exposed a real regression: `prompter --help` exited with code 2. Fixed by treating clap display-help and display-version paths as successful parser outcomes.
- `just lint` surfaced three pre-existing `gator` test-only clippy violations. Updated those tests to avoid needless collection.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Phase 01 is complete. The workspace now has one shared base CLI contract plus workspace-level enforcement against future UX drift.
