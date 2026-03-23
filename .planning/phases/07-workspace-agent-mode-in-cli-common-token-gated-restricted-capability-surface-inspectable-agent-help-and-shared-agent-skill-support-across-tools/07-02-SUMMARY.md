---
phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
plan: 02
subsystem: cli
tags: [rust, clap, cli-common, agent-mode, completions]
requires:
  - phase: 07-01
    provides: agent token detection, declarative capability metadata, command-tree pruning
provides:
  - shared agent-aware clap parsing with hidden-surface rejection
  - structured `--agent-help` and `--agent-skill` rendering with fallback metadata
  - completion generation from the same filtered clap command tree
affects: [07-03, 07-04, 07-05, 07-06, workspace-cli-rollout]
tech-stack:
  added: []
  patterns: [filtered-clap-command-tree, agent-dispatch-short-circuit, command-based-completion-rendering]
key-files:
  created: [.planning/phases/07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools/07-02-SUMMARY.md]
  modified:
    - crates/cli-common/src/agent.rs
    - crates/cli-common/src/command.rs
    - crates/cli-common/src/completions.rs
    - crates/cli-common/src/lib.rs
key-decisions:
  - "Agent parsing now reconstructs typed CLI values from `ArgMatches` after validating argv against the filtered command tree."
  - "Agent help and skill text derive from shared capability metadata with centralized fallback prose instead of tool-local strings."
  - "Shared completion dispatch now builds the clap command, applies agent pruning when active, and renders completions from that exact command."
patterns-established:
  - "Pattern: agent-mode parse/help/completion flows all consume the same pruned clap tree."
  - "Pattern: shared inspection flags short-circuit through `AgentDispatch::Printed` before tool-local dispatch."
  - "Pattern: completion APIs accept either a `CommandFactory` type or a pre-built `clap::Command`."
requirements-completed: [D-04, D-05, D-06, D-07, D-08, D-09]
duration: 3m
completed: 2026-03-23
---

# Phase 07 Plan 02: Shared agent-mode parse/help/completion pipeline Summary

**Restricted agent parsing, structured capability help, and filtered completion output now share one pruned clap command tree in `cli-common`**

## Performance

- **Duration:** 3m
- **Started:** 2026-03-23T13:55:23Z
- **Completed:** 2026-03-23T13:58:28Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Added shared agent-aware parse entrypoints that reject hidden commands and flags as nonexistent while preserving human-mode parsing.
- Added structured `--agent-help` and `--agent-skill` rendering with centralized fallback prose for capabilities that omit optional metadata.
- Routed completion rendering through command-based helpers so the shared metadata completion path respects the restricted agent surface.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a shared agent-aware parse path for clap-based tools** - `5caac2c` (feat)
2. **Task 2: Render structured `--agent-help` and `--agent-skill` output from capability metadata** - `c547876` (feat)
3. **Task 3: Make completion generation consume the same filtered command tree** - `fc417c4` (feat)

## Files Created/Modified
- `crates/cli-common/src/agent.rs` - agent-aware parse dispatch, inspection rendering, fallback capability metadata, and redaction tests
- `crates/cli-common/src/command.rs` - shared parse wrappers plus filtered completion dispatch for standard metadata commands
- `crates/cli-common/src/completions.rs` - command-based completion rendering/generation helpers
- `crates/cli-common/src/lib.rs` - re-exports for the new parse/help/completion APIs

## Decisions Made
- Reconstructed typed clap values from validated `ArgMatches` so downstream tool dispatch stays unchanged after agent-surface filtering.
- Kept fallback help prose centralized in `cli-common` by extending `AgentCapability` with optional metadata and deriving defaults when omitted.
- Added command-based completion helpers instead of duplicating rendering logic so typed and pre-pruned completion paths stay aligned.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `cli-common` now exposes shared parse/help/completion primitives for the workspace rollout plans in 07-03 through 07-06.
- Remaining rollout work can call the new shared wrappers instead of open-coding agent surface filtering in each tool.

## Self-Check: PASSED
- Verified summary file exists at `.planning/phases/07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools/07-02-SUMMARY.md`.
- Verified task commits `5caac2c`, `c547876`, and `fc417c4` exist in git history.

---
*Phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools*
*Completed: 2026-03-23*
