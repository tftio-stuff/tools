---
phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
plan: 03
subsystem: cli
tags: [rust, clap, cli-common, agent-mode, bce, gator]
requires:
  - phase: 07-02
    provides: shared filtered parse/help/completion pipeline and agent renderers in cli-common
provides:
  - bce shared agent-surface rollout with query-posts-only visibility
  - gator shared agent-surface rollout with restricted root launch visibility
  - package-local smoke coverage for both migrated tools
affects: [phase-07-rollout, cli-common-agent-surface, bsky-comment-extractor, gator]
tech-stack:
  added: []
  patterns:
    - shared parse_with_agent_surface entrypoints in binary mains
    - declarative AgentSurfaceSpec capability declarations per tool
    - root-launch capability filtering for agent-mode positionals
key-files:
  created:
    - crates/bsky-comment-extractor/tests/agent_surface.rs
    - crates/gator/tests/agent_surface.rs
  modified:
    - crates/cli-common/src/agent.rs
    - crates/bsky-comment-extractor/src/cli.rs
    - crates/bsky-comment-extractor/src/main.rs
    - crates/gator/src/main.rs
key-decisions:
  - "bce exposes only query-posts in agent mode and keeps fetch entirely outside the restricted surface."
  - "gator models its shared agent surface as a root launch capability instead of exposing meta or privileged sandbox flags."
  - "cli-common keeps root positionals only when a capability explicitly targets the root command path."
patterns-established:
  - "Binary mains can hand shared inspection flags and filtered parsing to cli-common via parse_with_agent_surface."
  - "Root-path capabilities use CommandSelector::new(&[]) plus explicit FlagSelector entries for allowed long flags."
requirements-completed: [D-04, D-05, D-06, D-07, D-08, D-09, D-13, D-14]
duration: 5m 13s
completed: 2026-03-23
---

# Phase 07 Plan 03: bce and gator shared agent surface rollout Summary

**Shared agent-mode rollout for `bce query` and the `gator` root launch path with filtered help, skill contracts, and hidden-surface rejection**

## Performance

- **Duration:** 5m 13s
- **Started:** 2026-03-23T14:01:16Z
- **Completed:** 2026-03-23T14:06:29Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Replaced the local `bce --agent-help` Phase 6 stub with the shared `cli-common` parse/help pipeline.
- Restricted `gator` agent mode to a single `run-agent` launch capability and hid `meta` plus privileged flags.
- Added package-local integration tests proving `--agent-help`, `--agent-skill`, and hidden-surface rejection for both tools.

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace `bce`'s local Phase 6 agent-help stub with the shared Phase 7 substrate**
   - `0b9244d` test
   - `3cc9265` feat
2. **Task 2: Migrate `gator` to the shared restricted agent-launch surface**
   - `8b376fa` test
   - `927194e` feat
3. **Task 3: Run package-level smoke verification for the `bce` and `gator` rollout**
   - `ae2da36` test

## Files Created/Modified
- `crates/bsky-comment-extractor/src/cli.rs` - removed the tool-local hidden `agent_help` flag from the clap definition.
- `crates/bsky-comment-extractor/src/main.rs` - added shared `ToolSpec` agent metadata and routed startup through `parse_with_agent_surface`.
- `crates/bsky-comment-extractor/tests/agent_surface.rs` - verifies shared help/skill output and hidden `fetch` rejection.
- `crates/cli-common/src/agent.rs` - keeps root positionals only for explicitly declared root capabilities.
- `crates/gator/src/main.rs` - declared the `run-agent` capability and switched entrypoint parsing to the shared agent surface.
- `crates/gator/tests/agent_surface.rs` - verifies restricted help, skill output, hidden `meta`, and hidden privileged flags.

## Decisions Made
- Used explicit prose on `bce`'s `query-posts` capability because the plan required specific examples, output semantics, and constraints.
- Kept `gator` on shared default output/constraint rendering and declared only the root launch selectors needed for filtering.
- Fixed root-positionals handling in `cli-common` instead of adding tool-local parsing exceptions so root-launch tools can share the same substrate.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Shared agent filtering dropped root launch positionals**
- **Found during:** Task 2 (Migrate `gator` to the shared restricted agent-launch surface)
- **Issue:** `cli-common` kept positional args only for non-root command paths, which broke the `gator` root launch capability and let the parser mis-handle agent-mode input.
- **Fix:** Updated `cli-common` to retain positionals only when the current command path is explicitly declared as a visible capability, including the root path.
- **Files modified:** `crates/cli-common/src/agent.rs`
- **Verification:** `cargo test -p tftio-gator agent_surface`
- **Committed in:** `927194e`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The auto-fix was required for the planned `gator` rollout and kept the shared substrate reusable for root-launch tools.

## Issues Encountered
- `just cli-metadata-consistency` points at a different worktree path (`feature-add-agent-help-to-all-tools`) in this checkout, so the recipe itself is currently broken here.
- The workspace CLI consistency suite still expects pre-existing `bce` metadata and flat-invocation behavior that are outside this plan's scoped agent-surface rollout.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- `bce` and `gator` now use the shared agent substrate and provide rollout examples for the remaining workspace tools.
- Remaining Phase 07 plans can follow the same `ToolSpec` + `parse_with_agent_surface` pattern for additional binaries.
- Workspace-wide metadata consistency should be reconciled separately before relying on the `just cli-metadata-consistency` recipe in this worktree.

## Self-Check: PASSED
