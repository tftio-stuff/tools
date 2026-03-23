---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Ready to execute
stopped_at: Completed 07-03-PLAN.md
last_updated: "2026-03-23T14:07:23.079Z"
progress:
  total_phases: 3
  completed_phases: 0
  total_plans: 6
  completed_plans: 5
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-22)

**Core value:** Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.
**Current focus:** Phase 07 — workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools

## Current Position

Phase: 07 (workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools) — EXECUTING
Plan: 6 of 6

## Completed Milestones

**Velocity (v1.1):**

- Total plans completed: 2
- Average duration: ~4.5min
- Total execution time: ~9min

## Integration Status

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 03-extraction-engine | 2 | 9min | 4.5min |
| Phase 04-cli-surface P01 | 4 | 2 tasks | 4 files |
| Phase 04 P02 | 3.5min | 3 tasks | 6 files |
| Phase 05-query-subcommand P02 | 2m | 2 tasks | 2 files |
| Phase 05-query-subcommand P01 | 0 min | 2 tasks | 3 files |
| Phase 05-query-subcommand P03 | 3m 23s | 3 tasks | 4 files |

## Next Steps

Continue Phase 07 execution:

- Execute 07-02 to wire the shared filtered parse/help/completion pipeline onto `cli-common`
- Roll the shared restricted agent surface through the remaining workspace tools in 07-03 through 07-06
- Keep phase summaries current so Phase 07 rollout decisions remain inspectable

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
- [bce-query-mode roadmap]: QUERY-01 through QUERY-04 and AGENT-02 grouped into Phase 5 (coherent query unit); AGENT-01 isolated in Phase 6 (separate agent-doc concern)
- [Phase 05-query-subcommand]: 05-02: QueryArgs accepts only db, limit, and offset, with defaults 50 and 0.
- [Phase 05-query-subcommand]: 05-02: Cli reserves agent-help as a top-level global flag and exposes optional fetch/query subcommands.
- [Phase 05-query-subcommand]: Query pagination uses ORDER BY created_at DESC, uri DESC to keep page boundaries stable.
- [Phase 05-query-subcommand]: Query mode opens SQLite with SQLITE_OPEN_READ_WRITE and no create flag so missing database paths fail fast.
- [Phase 05-query-subcommand]: Query execution stays synchronous and never reads BSKY_APP_PASSWORD or starts tokio.
- [Phase 05-query-subcommand]: Query failures emit structured stderr JSON with db_not_found for missing databases and query_failed for other runtime errors.
- [Phase 05-query-subcommand]: The global --agent-help flag remains parseable but hidden from subcommand help so bce query --help exposes only query options.

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

## Decisions

- [Phase 07-01]: Agent mode fails closed unless `TFTIO_AGENT_TOKEN` exactly matches `TFTIO_AGENT_TOKEN_EXPECTED`.
- [Phase 07-01]: Workspace tools declare static `AgentSurfaceSpec` metadata on `ToolSpec` instead of tool-local policy callbacks.
- [Phase 07-01]: `apply_agent_surface` prunes the `clap` command tree and preserves only declared capability paths plus shared inspection flags.
- [Phase 07]: Agent parsing now reconstructs typed CLI values from ArgMatches after validating argv against the filtered command tree.
- [Phase 07]: Agent help and skill text derive from shared capability metadata with centralized fallback prose instead of tool-local strings.
- [Phase 07]: Shared completion dispatch now builds the clap command, applies agent pruning when active, and renders completions from that exact command.
- [Phase 07]: Todoer declares its agent-visible workflows on TOOL_SPEC and uses parse_with_agent_surface for shared help/skill rendering.
- [Phase 07]: Unvenv agent mode exposes only scan-venvs; version, license, completions, doctor, and update remain human-only.
- [Phase 07]: cli-common filtering now owns recursive subcommand path segments before descending into nested filtered commands.
- [Phase 07]: 07-05: asana-cli exposes only declared operational capability groups through parse_with_agent_surface.
- [Phase 07]: 07-05: silent-critic agent mode exposes only session-status, session-manifest, and session-submit while SILENT_CRITIC_TOKEN remains the runtime worker auth gate.
- [Phase 07]: 07-05: cli-common now preserves full descendant subtrees when a capability explicitly selects a grouped command path.
- [Phase 07]: bce exposes only query-posts in agent mode while fetch stays outside the restricted surface.
- [Phase 07]: gator models agent mode as a root launch capability and hides meta plus privileged flags.
- [Phase 07]: cli-common retains root positionals only when a capability explicitly targets the root command path.

## Performance Metrics

| Phase | Plan | Duration | Tasks | Files | Completed |
|-------|------|----------|-------|-------|-----------|
| 07 | 01 | 13m | 2 | 3 | 2026-03-23 |
| Phase 07 P02 | 3m | 3 tasks | 4 files |
| Phase 07 P04 | 2m 57s | 3 tasks | 5 files |
| Phase 07 P05 | 4m17s | 3 tasks | 5 files |
| Phase 07 P03 | 5m 13s | 3 tasks | 6 files |

## Accumulated Context

### Roadmap Evolution

- Phase 7 added: Workspace agent mode in cli-common: token-gated restricted capability surface, inspectable agent help, and shared --agent-skill support across tools
  - Supersets the original Phase 6 `bce --agent-help` scope
  - Extends the agent-facing contract into `cli-common` for workspace-wide reuse
  - Introduces environment-token-gated restricted capability exposure for autonomous agents

### Pending Todos

None.

### Blockers/Concerns

- "blocked-by" data is not in the user's own repo; requires a separate API call (`app.bsky.graph.getBlocks`) -- deferred to v2
- Rate limit: ~3,000 req/5min; plan for backoff from the start

## Session Continuity

Last session: 2026-03-23T14:07:23.076Z
Stopped at: Completed 07-03-PLAN.md
Resume file: None
