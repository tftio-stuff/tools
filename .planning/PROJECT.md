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
- [x] Shared top-level `--agent-help` YAML and `--agent-skill` Claude skill output across all seven workspace binaries -- validated in Phase 7: Add shared `--agent-help` and `--agent-skill` support across CLI crates

### Active

- [ ] Phase 6 milestone cleanup in the active `.planning` tree (Phase 7 completed the cross-workspace rollout while the bce-specific roadmap still shows Phase 6 pending)

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

- 1,416 lines of Rust across 7 source files in `crates/bsky-comment-extractor/`
- Binary: `bce` (installed via `cargo install tftio-bsky-comment-extractor`)
- 32 tests (9 db, 14 client, 4 cli parse, 2 main, 3 ignored integration)
- Dependencies: reqwest (rustls), rusqlite (bundled), clap, tokio, serde, chrono, indicatif, dateparser, directories, anyhow, thiserror
- AT Protocol collections: `app.bsky.feed.post` (v1); `app.bsky.feed.like`, `app.bsky.feed.repost`, `app.bsky.graph.block` planned for v2
- Rate limit: ~3,000 requests per 5 minutes; `listRecords` paginated at 100 records per request

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

## Current Milestone: bce-query-mode

**Goal:** Make bce's stored data queryable by LLM agents via JSON output with pagination.

**Target features:**
- `bce query` subcommand: read-only against local SQLite, JSON output
- Offset/limit pagination for paging through results
- `--agent-help` flag: structured reference doc for LLM agent consumption

## Current State

**Phase 7 complete on 2026-03-23.** All seven workspace binaries now expose shared top-level `--agent-help` and `--agent-skill` output through `cli-common`, and `bce` still supports authenticated fetch plus read-only `query` output with envelope-first JSONL pagination from the local SQLite store. The active `.planning` roadmap still lists the earlier bce-specific Phase 6 placeholder.

---
*Last updated: 2026-03-23 after Phase 7 shared agent-doc rollout completion*
