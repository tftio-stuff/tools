---
gsd_state_version: 1.0
milestone: bce-query-mode
milestone_name: bce-query-mode
status: Ready to plan
stopped_at: Completed Phase 7 rollout and UAT
last_updated: "2026-03-23T16:30:00Z"
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 9
  completed_plans: 9
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23)

**Core value:** Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.
**Current focus:** No active milestone. The last completed milestone is `bce-query-mode`.

## Current Position

Phase: None
Plan: None

## Completed Milestones

- `v1.0` — shipped 2026-03-18
- `v1.1` — shipped 2026-03-22
- `bce-query-mode` — shipped 2026-03-23

## Next Steps

No active milestone is open.

Recommended:
- add the next milestone or phase when new work is ready
- keep Phase 5 and Phase 7 artifacts as the current inspectable record of shipped behavior

## Decisions

- Query pagination uses `ORDER BY created_at DESC, uri DESC` to keep page boundaries stable across repeated runs.
- Query mode opens existing SQLite databases without create semantics so missing paths fail fast.
- Workspace agent mode fails closed unless `TFTIO_AGENT_TOKEN` exactly matches `TFTIO_AGENT_TOKEN_EXPECTED`.
- Workspace tools declare agent-visible capabilities in shared `cli-common` metadata instead of tool-local policy callbacks.
- Shared `cli-common` filtering owns parser redaction, help redaction, completion redaction, `--agent-help`, and `--agent-skill` rendering.
- BCE exposes only its query capability in restricted agent mode while fetch remains outside that surface.

## Pending Todos

None.

## Blockers/Concerns

- "blocked-by" data is not in the user's own repo; requires a separate API call (`app.bsky.graph.getBlocks`) -- deferred to future work
- Rate limit: ~3,000 req/5min; keep backoff in any later network expansion

## Session Continuity

Last session: 2026-03-23T16:30:00Z
Stopped at: Completed Phase 7 rollout and UAT
Resume file: None
