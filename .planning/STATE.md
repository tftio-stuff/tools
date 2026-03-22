---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: bsky-comment-extractor
status: complete
stopped_at: Completed 03-02-PLAN.md
last_updated: "2026-03-22T16:16:16.677Z"
progress:
  total_phases: 2
  completed_phases: 1
  total_plans: 2
  completed_plans: 2
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-22)

**Core value:** Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.
**Current focus:** Phase 03 — extraction-engine

## Current Position

Phase: 03 (extraction-engine) — COMPLETE
Plan: 2 of 2 (all plans complete)

## Completed Milestones

**Velocity (v1.1):**

- Total plans completed: 2
- Average duration: ~4.5min
- Total execution time: ~9min

## Integration Status

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 03-extraction-engine | 2 | 9min | 4.5min |

## Next Steps

No active milestone. Recommended:
- Start new symbolic milestone with GSD workflow if architectural work needed
- Continue product features (activity-type filtering) when ready
- Release versioning managed by release-please

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

Last session: 2026-03-22T16:16:16.674Z
Stopped at: Completed 03-02-PLAN.md
Resume file: None
