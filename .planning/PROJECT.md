# bsky-comment-extractor

## What This Is

A Rust CLI tool (`bce`) that exhaustively retrieves a BlueSky user's post history via the AT Protocol and stores it in a local SQLite database. A workspace member of the `tools` Cargo monorepo.

## Core Value

Complete, reliable extraction of a single BlueSky user's entire post history into a queryable local store.

## Requirements

### Validated

- [x] Authenticate to BlueSky via app password -- v1.1
- [x] Retrieve all posts via `com.atproto.repo.listRecords` with exhaustive pagination -- v1.1
- [x] Resolve handle to DID, rate-limit backoff on HTTP 429 -- v1.1
- [x] Store posts in SQLite with structured schema, idempotent writes -- v1.1
- [x] Configurable database path (`--db` flag, XDG default) -- v1.1
- [x] CLI interface following workspace conventions (clap, cli-common, indicatif) -- v1.1
- [x] Query mode reads stored posts from local SQLite and outputs envelope-first JSONL pagination -- validated in Phase 5: Query Subcommand
- [x] Offset/limit pagination supports page traversal through stored results -- validated in Phase 5: Query Subcommand
- [x] Agent-facing help is available through the shared `cli-common` restricted agent surface (`--agent-help` / `--agent-skill`) -- validated in Phase 7: Workspace agent mode in cli-common

### Active

None.

### Future

- [ ] Support filtering by activity type: posts, likes, reposts, quote-posts, blocks, blocked-by
- [ ] Default filter: posts (all posts including replies) when no filter specified

### Out of Scope

- Firehose/streaming consumption -- batch retrieval only
- Multi-user extraction in a single invocation
- Real-time monitoring or polling
- OAuth authentication -- app passwords sufficient
- Search by keyword -- extracts activity, not search results

## Context

- Rust workspace member in `tools/`
- Binary: `bce` (installed via `cargo install tftio-bsky-comment-extractor`)
- Query output is envelope-first JSONL over the local SQLite store
- Shared agent-facing inspection now comes from `tftio-cli-common`, not a tool-local one-off implementation

## Constraints

- **Tech stack**: Rust, workspace member of `tools/`. Follows workspace lint config, dependency patterns, and crate structure
- **Dependencies**: Workspace-level deps via `dep.workspace = true`
- **Auth**: App password only (no OAuth complexity)
- **API approach**: `com.atproto.repo.listRecords` as primary data source

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SQLite output | Queryable, structured, consistent with workspace (todoer, silent-critic) | Good -- works well for single-user extraction |
| `listRecords` over `getAuthorFeed` | Completeness over richness; raw repo data captures everything | Good -- exhaustive, no gaps |
| App password auth only | Simple, well-supported, OAuth adds DPoP complexity for no gain here | Good -- sufficient for CLI tool |
| Workspace crate, not standalone | Shares deps, lint config, CI, release tooling | Good -- seamless workspace integration |
| Reply parent in raw_json, not dedicated column | Queryable via `json_extract()`, avoids schema rigidity | Good -- flexible for future query needs |
| Sync main + RuntimeBuilder (not `#[tokio::main]`) | Matches workspace pattern (asana-cli) | Good -- consistent conventions |
| XDG default path via `directories` crate | `~/.local/share/bce/bsky-posts.db` with auto-created parent dirs | Good -- follows platform conventions |
| Shared agent surface in `cli-common` | Keeps agent visibility, help, skill docs, completions, and redaction behavior uniform across workspace CLIs | Good -- BCE participates in the same restricted agent contract as the rest of the workspace |

## Latest Completed Milestone: bce-query-mode

**Goal:** Make BCE's stored data queryable by LLM agents via JSON output with pagination and a restricted inspectable CLI surface.

**Completed features:**
- `bce query` subcommand: read-only against local SQLite, JSON output
- Offset/limit pagination for paging through results
- Shared `cli-common` agent mode with BCE capability documentation via `--agent-help` and `--agent-skill`

## Current State

**bce-query-mode shipped on 2026-03-23.** There is no active milestone in this repo at the moment.

---
*Last updated: 2026-03-23 after Phase 7 completion*
