# Milestone: cli-common-maximal-sharing

**Status:** ✅ SHIPPED 2026-03-22
**Milestone Type:** symbolic release-please planning milestone
**Phases:** 02-03
**Total Plans:** 8
**Total Tasks:** 8
**Git Range:** `250e68a` → `3ef48ef`
**Diff Summary:** 29 files changed, 2748 insertions(+), 650 deletions(-)

## Overview

Maximized the reusable CLI substrate in `tftio-cli-common` so workspace tools share one metadata-command mapping layer, one fatal runner surface, one lazy JSON/text response emitter, one doctor/report scaffold, and one repository-level CLI contract suite.

## Phases

### Phase 02: Maximize cli-common sharing

**Goal**: Audit the tools and the `cli-common` crate, then move as much shared or generally useful code as possible out of individual tools and into `cli-common`.
**Depends on**: Phase 01
**Plans**: 4 plans

Plans:

- [x] 02-01-PLAN.md -- Expand `cli-common` with shared adapters, completion rendering buffers, and doctor-report primitives
- [x] 02-02-PLAN.md -- Migrate the adapter-heavy tools off local metadata and response boilerplate
- [x] 02-03-PLAN.md -- Migrate `unvenv`, `asana-cli`, and `prompter` onto the richer shared surface
- [x] 02-04-PLAN.md -- Finish dependency cleanup, repository enforcement, and documentation of the final boundary

**Details:**

Success criteria achieved:
1. `cli-common` gained workspace tool presets, doctorless adapters, buffer-first completion rendering, structured doctor reports, and shared response helpers.
2. `gator`, `todoer`, `silent-critic`, and `bce` deleted large amounts of local metadata and response boilerplate while preserving command flows.
3. `prompter`, `unvenv`, and `asana-cli` adopted shared completion, doctor, and workspace metadata helpers without losing tool-specific behavior.
4. Repository shell enforcement and CLI boundary docs were added so extracted glue stays deleted.

### Phase 03: Extract remaining CLI glue into `cli-common`

**Goal**: Remove the remaining reusable metadata-mapping, fatal-runner, response-emitter, and doctor-provider scaffolding from individual tools so `cli-common` owns the full shared CLI substrate.
**Depends on**: Phase 02
**Plans**: 4 plans

Plans:

- [x] 03-01-PLAN.md -- Expand `cli-common` with the final shared metadata, runner, response, and doctor helper layer
- [x] 03-02-PLAN.md -- Migrate `gator`, `bce`, and `todoer` off the remaining thin-entrypoint glue
- [x] 03-03-PLAN.md -- Migrate `silent-critic`, `unvenv`, `asana-cli`, and `prompter` off the remaining richer glue
- [x] 03-04-PLAN.md -- Enforce the thinner boundary, document it, and run the full suite

**Details:**

Success criteria achieved:
1. `cli-common` now owns metadata-command mapping helpers, fatal runner helpers, lazy success-response rendering, and doctor-report scaffolding/builders.
2. Thin-entrypoint tools (`gator`, `bce`, `todoer`) now keep local command/domain behavior while shared entrypoint glue lives in `cli-common`.
3. Richer tools (`silent-critic`, `unvenv`, `asana-cli`, `prompter`) now use shared response, fatal-error, and doctor-report infrastructure.
4. The repository shell suite, `just cli-consistency`, and `just cli-metadata-consistency` enforce the final boundary automatically.

## Milestone Summary

**Key Accomplishments:**

- Expanded `tftio-cli-common` with the final shared metadata-command mapping, fatal runner, lazy response emission, and doctor-report builder surface.
- Migrated `gator`, `bce`, and `todoer` onto thinner entrypoints backed by shared `cli-common` infrastructure.
- Migrated `silent-critic`, `unvenv`, `asana-cli`, and `prompter` onto shared response/fatal/doctor helpers while preserving domain-specific behavior.
- Added a repository shell suite that exercises metadata commands, JSON/error contracts, doctor behavior, completion augmentation, and deleted-boilerplate assertions.

**Key Decisions:**

- Use symbolic milestone names because releases are managed by release-please instead of milestone tagging.
- Keep shared CLI infrastructure in `tftio-cli-common`; keep domain summaries, data collection, and dynamic augmentation local to the tool crates.
- Enforce the final shared/local split through shell contracts and explicit documentation instead of relying on convention alone.

**Issues Resolved:**

- Removed the remaining repeated `StandardCommand` mapping boilerplate from workspace binaries.
- Unified top-level fatal CLI handling without forcing every tool into one clap layout.
- Eliminated more repeated JSON-vs-text success branching while preserving tool-specific text formatting.
- Standardized doctor report scaffolding and repository-level CLI drift checks.

**Issues Deferred:**

- `bce` activity-type filtering remains future work.
- Release packaging and version tagging remain owned by release-please rather than milestone completion.
- A second tool has not yet required post-render completion augmentation helpers beyond `prompter`'s existing local logic.

**Technical Debt Incurred:**

- `gator` still needs a local wrapper around its library-owned clap enums because orphan rules prevent implementing the shared mapping trait directly on the foreign type.
- `bce` still keeps a minimal local doctor provider because only a small amount of doctor state remains tool-specific.

---

For current project status, see `.planning/ROADMAP.md`.
