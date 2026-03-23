# Milestone: cli-common-unification

**Status:** ✅ SHIPPED 2026-03-22
**Milestone Type:** symbolic release-please planning milestone
**Phases:** 01
**Total Plans:** 4

## Overview

Consolidated the workspace CLI base UX into `tftio-cli-common` so metadata commands, JSON envelopes, top-level error rendering, doctor behavior, progress handling, and completion surfaces are standardized across the Rust CLI tools in this repository.

## Phases

### Phase 01: CLI Common Unification

**Goal**: Consolidate base CLI UX into `tftio-cli-common` so workspace binaries share one metadata-command surface, one error/JSON contract, one progress pattern, and centralized CLI dependencies.
**Depends on**: v1.1 workspace baseline
**Plans**: 4 plans

Plans:

- [x] 01-01-PLAN.md -- Build the shared `cli-common` metadata, command, JSON, error, and progress foundation
- [x] 01-02-PLAN.md -- Migrate machine-oriented CLIs (`gator`, `todoer`, `silent-critic`) to the shared base contract
- [x] 01-03-PLAN.md -- Migrate user-facing CLIs (`unvenv`, `bce`, `asana-cli`) to the shared base contract
- [x] 01-04-PLAN.md -- Finish `prompter`, add workspace CLI consistency checks, and document the boundary

**Details:**

Success criteria achieved:
1. `cli-common` owns the shared metadata-command dispatcher, tool metadata contract, JSON envelopes, top-level error rendering, and spinner helper
2. `gator`, `todoer`, and `silent-critic` now share one machine-facing JSON and error contract
3. `unvenv`, `bce`, `asana-cli`, and `prompter` now share the base metadata/doctor/error/progress UX without losing crate-specific behavior
4. Workspace-level shell coverage and `just cli-consistency` now detect CLI base UX drift

## Milestone Summary

**Key Decisions:**

- Use a symbolic milestone name because releases are managed by release-please rather than manual semantic milestone tags
- Base CLI UX belongs in `tftio-cli-common`, not duplicated in per-tool entrypoints
- Workspace CLI drift is enforced through repository-level shell tests plus `just cli-consistency`

**Issues Resolved:**

- Removed duplicated JSON envelope helpers from multiple CLIs
- Unified metadata-command behavior across machine-facing and user-facing binaries
- Preserved `prompter` dynamic completion augmentation while moving shared behavior into `cli-common`

**Issues Deferred:**

- Activity-type filtering for `bce` remains future work
- Release packaging remains owned by release-please rather than milestone tagging

**Technical Debt Incurred:**

- `cli-consistency` still combines shell checks and direct command assertions in one recipe; it can be split further if coverage grows materially

---

For current project status, see `.planning/ROADMAP.md`.
