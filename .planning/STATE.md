---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: bsky-comment-extractor
status: ready_to_plan
stopped_at: Roadmap created for v1.1
last_updated: "2026-03-22"
last_activity: 2026-03-22 -- Roadmap created, ready to plan Phase 3
progress:
  total_phases: 2
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-22)

**Core value:** Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.
**Current focus:** Phase 3 - Extraction Engine

## Current Position

Phase: 3 of 4 in v1.1 (Phase 3 not started)
Plan: No plans yet
Status: Ready to plan
Last activity: 2026-03-22 -- Roadmap created for v1.1 bsky-comment-extractor

Progress: [░░░░░░░░░░] 0% (v1.1)

## Performance Metrics

**Velocity (v1.1):**
- Total plans completed: 0
- Average duration: -
- Total execution time: -

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
- [v1.1 init]: Use `com.atproto.repo.listRecords` over `getAuthorFeed` for completeness
- [v1.1 init]: App password auth only (no OAuth/DPoP)
- [v1.1 init]: SQLite output (consistent with todoer/silent-critic workspace pattern)
- [v1.1 init]: New workspace crate `bsky-comment-extractor` in `crates/`

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
- [bce-query-mode roadmap]: QUERY-01 through QUERY-04 and AGENT-02 grouped into Phase 5 (coherent query unit); AGENT-01 isolated in Phase 6 (separate agent-doc concern)
- [Phase 05-query-subcommand]: 05-02: QueryArgs accepts only db, limit, and offset, with defaults 50 and 0.
- [Phase 05-query-subcommand]: 05-02: Cli reserves agent-help as a top-level global flag and exposes optional fetch/query subcommands.
- [Phase 05-query-subcommand]: Query pagination uses ORDER BY created_at DESC, uri DESC to keep page boundaries stable.
- [Phase 05-query-subcommand]: Query mode opens SQLite with SQLITE_OPEN_READ_WRITE and no create flag so missing database paths fail fast.
- [Phase 05-query-subcommand]: Query execution stays synchronous and never reads BSKY_APP_PASSWORD or starts tokio.
- [Phase 05-query-subcommand]: Query failures emit structured stderr JSON with db_not_found for missing databases and query_failed for other runtime errors.
- [Phase 05-query-subcommand]: The global --agent-help flag remains parseable but hidden from subcommand help so bce query --help exposes only query options.

### Pending Todos

None.

### Blockers/Concerns

- "blocked-by" data is not in the user's own repo; requires a separate API call (`app.bsky.graph.getBlocks`) -- deferred to v2
- Rate limit: ~3,000 req/5min; plan for backoff from the start

## Session Continuity

Last session: 2026-03-22
Stopped at: Roadmap created, no plans written yet
Resume file: None
