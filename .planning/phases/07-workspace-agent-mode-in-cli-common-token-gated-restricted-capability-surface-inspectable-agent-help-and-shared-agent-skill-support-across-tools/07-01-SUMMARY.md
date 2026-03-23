---
phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
plan: 01
subsystem: cli
tags: [rust, clap, cli-common, agent-mode, capability-policy]
requires:
  - phase: 06-agent-help
    provides: shared `--agent-help` groundwork that Phase 7 extends into workspace-wide agent mode
provides:
  - shared env-var contract for agent-mode activation in `cli-common`
  - declarative `ToolSpec` agent surfaces and named capability metadata
  - shared `clap` pruning helpers that filter commands and flags by capability
affects: [07-02, 07-03, 07-04, 07-05, 07-06, workspace-cli-agent-surface]
tech-stack:
  added: []
  patterns:
    - exact token equality activation via shared env vars
    - static `AgentSurfaceSpec` declarations attached to `ToolSpec`
    - capability-first `clap` tree pruning in `cli-common`
key-files:
  created: []
  modified:
    - crates/cli-common/src/agent.rs
    - crates/cli-common/src/app.rs
    - crates/cli-common/src/lib.rs
key-decisions:
  - "Agent mode fails closed unless `TFTIO_AGENT_TOKEN` exactly matches `TFTIO_AGENT_TOKEN_EXPECTED`."
  - "Workspace tools declare static `AgentSurfaceSpec` metadata on `ToolSpec` instead of using tool-local policy callbacks."
  - "`apply_agent_surface` prunes the actual `clap::Command` tree and preserves only declared capability paths plus shared inspection flags."
patterns-established:
  - "Agent activation: read both shared env vars at process start and require exact string equality."
  - "Capability declaration: store command-path and flag selectors in static `AgentCapability` metadata."
  - "Surface filtering: build restricted agent-mode command trees from shared capability declarations in `cli-common`."
requirements-completed: [D-01, D-02, D-03, D-10, D-11, D-12]
duration: 13m
completed: 2026-03-23
---

# Phase 07 Plan 01: Shared agent-mode contract summary

**Shared `cli-common` agent contract with exact token gating, declarative capabilities, and `clap` surface pruning helpers**

## Performance

- **Duration:** 13m
- **Started:** 2026-03-23T13:33:27Z
- **Completed:** 2026-03-23T13:46:40Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added shared `cli-common` agent-mode env-var constants and `AgentModeContext::detect()` fail-closed activation logic.
- Extended `ToolSpec` to carry an optional static `AgentSurfaceSpec` so workspace tools can declare named capabilities centrally.
- Added shared filtering helpers that prune a `clap::Command` tree down to declared subcommands and long flags while preserving shared agent inspection flags.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define shared agent-mode types and the workspace token contract** - `f5bb499` (feat)
2. **Task 2: Implement capability-first filtering helpers in `cli-common`** - `3687f32` (feat)

## Files Created/Modified

- `crates/cli-common/src/agent.rs` - Shared agent-mode constants, capability metadata, activation detection, filtering helpers, and unit tests.
- `crates/cli-common/src/app.rs` - `ToolSpec` support for optional declarative agent surfaces via `agent_surface` and `with_agent_surface`.
- `crates/cli-common/src/lib.rs` - Public re-exports for the shared agent-mode API.

## Decisions Made

- Used exact env-var string equality for activation so missing or mismatched tokens always leave agent mode inactive.
- Kept policy input purely declarative on `ToolSpec` to avoid tool-local callback hooks in the shared contract.
- Pruned the actual `clap` command tree in agent mode so help, parsing, and completion surfaces can share the same restricted structure in later plans.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None within plan scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `cli-common` now exports the shared agent-mode primitives required by Phase 07 plans 02-06.
- Later plans can reuse `AgentModeContext`, `AgentSurfaceSpec`, `visible_capabilities`, and `apply_agent_surface` without renaming or tool-local policy code.

## Self-Check: PASSED

- Verified `.planning/phases/07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools/07-01-SUMMARY.md` exists.
- Verified task commits `f5bb499` and `3687f32` exist in git history.

---
*Phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools*
*Completed: 2026-03-23*
