---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Ready to execute
stopped_at: Completed 03-02-PLAN.md
last_updated: "2026-03-22T22:31:12.565Z"
progress:
  total_phases: 2
  completed_phases: 1
  total_plans: 8
  completed_plans: 6
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-22)

**Core value:** Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.
**Current focus:** Phase 03 — extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding

## Current Position

Phase: 03 (extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding) — EXECUTING
Plan: 3 of 4

## Performance Metrics

**Velocity (v1.1):**

- Total plans completed: 2
- Average duration: ~4.5min
- Total execution time: ~9min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 03-extraction-engine | 2 | 9min | 4.5min |
| Phase 04-cli-surface P01 | 4 | 2 tasks | 4 files |
| Phase 04 P02 | 3.5min | 3 tasks | 6 files |
| Phase 01 P01 | 4 min | 2 tasks | 9 files |
| Phase 03 P01 | 3 min | 2 tasks | 6 files |
| Phase 03 P02 | 5 min | 2 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.

- [v1.1 init]: Use `com.atproto.repo.listRecords` over `getAuthorFeed` for completeness
- [v1.1 init]: App password auth only (no OAuth/DPoP)
- [v1.1 init]: SQLite output (consistent with todoer/silent-critic workspace pattern)
- [v1.1 init]: New workspace crate `bsky-comment-extractor` in `crates/`
- [03-01]: save_cursor uses SELECT + UPDATE/INSERT (not INSERT OR REPLACE) to preserve AUTOINCREMENT id on extractions table
- [03-01]: u64 record_count stored via .cast_signed() for clippy pedantic cast_possible_wrap compliance
- [03-01]: Technical nouns (SQLite, BlueSky) require backticks in doc comments under workspace pedantic clippy
- [Phase 03-02]: execute() uses Option<Vec<u8>> for body to allow cloning across retry iterations
- [Phase 03-02]: backoff_delay uses bit-shift not saturating_shl (unavailable on stable u64 MSRV)
- [Phase 03-02]: future_not_send allowed on async fns taking &rusqlite::Connection (expected single-threaded)
- [Phase 04-01]: on_progress uses Option<&dyn Fn(u64)> ref pattern -- non-Send compatible, no heap allocation, matches future_not_send established pattern
- [Phase 04-01]: upsert_post checks db_has_uri before INSERT OR REPLACE rather than conn.changes() -- semantically unambiguous, avoids surprising delete+insert change count
- [Phase 04-02]: make_spinner returns None when quiet=true OR stdout is not TTY, matching workspace UX pattern
- [Phase 04-02]: test_db_path_default checks path contains bce and ends with bsky-posts.db for cross-platform correctness
- [Phase 01-01]: `ToolSpec` and `StandardCommand` centralize shared CLI metadata, doctor, completions, version, and update wiring in `cli-common`
- [Phase 01-02]: `gator`, `todoer`, and `silent-critic` share one JSON envelope and one top-level error renderer from `cli-common`
- [Phase 01-03]: `unvenv`, `bce`, and `asana-cli` keep their invocation ergonomics while sharing base metadata, progress, and error primitives
- [Phase 01-04]: `prompter` joins the shared base contract and workspace drift is checked through `just cli-consistency`
- [Milestone]: release-please milestones use symbolic names in planning artifacts instead of manual semantic version tags
- [Milestone]: cli-common-maximal-sharing started to continue extracting shared CLI behavior into `tftio-cli-common`
- [Phase 02 planning]: remaining high-value extractions are adapter helpers, completion buffering, structured doctor reporting, and repository-level drift enforcement
- [Phase 02-01]: `cli-common` now provides `workspace_tool`, the shared doctorless adapter, buffer-first completion rendering, structured doctor reports, and shared response rendering helpers
- [Phase 02-02]: `gator`, `todoer`, `silent-critic`, and `bce` now consume the richer shared metadata surface and the shell suite enforces deleted boilerplate patterns
- [Phase 02-03]: `prompter` now uses shared completion rendering and JSON doctor reporting helpers; `unvenv` and `asana-cli` now use shared workspace tool presets
- [Phase 02-04]: workspace docs and CLI consistency automation now describe and enforce the maximal-sharing boundary
- [Phase 03 planning]: the remaining shared CLI glue is metadata mapping, fatal-runner plumbing, richer response emission, and doctor-provider scaffolding
- [Phase 03]: Metadata helpers support global JSON flags, version-local JSON flags, and doctor/update variants through one shared mapping trait and macro.
- [Phase 03]: Fatal CLI handling stays closure-based so tools can keep their clap layouts while centralizing error printing and exit-code behavior.
- [Phase 03]: Response and doctor helpers stay infrastructure-only: text formatting remains caller-owned while cli-common owns the JSON/text branch and doctor scaffolding.
- [Phase 03]: gator uses a local StandardCommandMap wrapper for library-owned clap enums so shared metadata dispatch can replace repeated StandardCommand matches.
- [Phase 03]: bce keeps a minimal DoctorChecks provider while maybe_run_standard_command handles its shared metadata dispatch without changing doctor behavior.
- [Phase 03]: todoer task output now keeps text formatting local while cli-common owns JSON-text branching and direct shared error rendering.

### Pending Todos

None.

### Blockers/Concerns

- "blocked-by" data is not in the user's own repo; requires a separate API call (`app.bsky.graph.getBlocks`) -- deferred to v2
- Rate limit: ~3,000 req/5min; plan for backoff from the start

### Roadmap Evolution

- Phase 1 added: I want you to examine all the code here and extract into the cli-common create all common functionality. Be extremely thorough; I want these tools to have a completely uniform base UI/UX, and I want all dependencies to be centralized.
- Phase 2 added: Audit the tools and `cli-common`, then move as much shared or generally useful functionality as possible into `cli-common`.
- Phase 3 added: Extract remaining CLI glue into `cli-common`: metadata mapping, fatal runners, response emitters, and doctor scaffolding.
- Phase 2 planned: 4 plans across 4 waves for maximal extraction into `tftio-cli-common`.
- Phase 2 executed: shared CLI adapters, completion rendering, doctor reporting, response helpers, and drift tests expanded across the workspace.
- Phase 3 planned: 4 plans across 4 waves to remove the remaining metadata, runner, response, and doctor glue from tool crates.

## Session Continuity

Last session: 2026-03-22T22:31:12.563Z
Stopped At: Completed 03-02-PLAN.md
Resume file: None
