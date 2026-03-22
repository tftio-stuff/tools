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
- [x] Query mode reads stored posts from local SQLite and outputs envelope-first JSONL pagination -- bce-query-mode milestone
- [x] Offset/limit pagination supports page traversal through stored results -- bce-query-mode milestone
- [x] Shared metadata commands (version, license, completions, doctor) via cli-common integration -- cli-common-maximal-sharing milestone

### Active

- [ ] Support filtering by activity type: posts, likes, reposts, quote-posts, blocks, blocked-by
- [ ] Default filter: posts (all posts including replies) when no filter specified

### Out of Scope

- Firehose/streaming consumption -- batch retrieval only
- Multi-user extraction in a single invocation
- Real-time monitoring or polling
- OAuth authentication -- app passwords sufficient
- Search by keyword -- extracts activity, not search results
- Output formats other than SQLite (JSONL, CSV) -- may revisit later

## Context

- 1,416 lines of Rust across 7 source files in `crates/bsky-comment-extractor/`
- Binary: `bce` (installed via `cargo install tftio-bsky-comment-extractor`)
- 32 tests (9 db, 14 client, 4 cli parse, 2 main, 3 ignored integration)
- Dependencies: reqwest (rustls), rusqlite (bundled), clap, tokio, serde, chrono, indicatif, dateparser, directories, anyhow, thiserror, tftio-cli-common
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
| Symbolic CLI refactor milestones | Release-please owns releases; planning milestones archive shared refactors without manual version tags | Good -- consistent with repository release workflow |
| Shared CLI substrate in `cli-common` | Workspace CLI UX stays consistent when metadata, runner, response, doctor, and shell enforcement live in one crate | Good -- final shared/local split is now explicit and enforced |
| Query subcommand instead of flat CLI | Explicit `bce fetch` and `bce query` separation clarifies networked vs local-only operations | Good -- clearer UX, room for future subcommands |

## Completed Milestones

### bce-query-mode (Completed 2026-03-22)
**Goal:** Make bce's stored data queryable by LLM agents via JSON output with pagination.

**Features delivered:**
- `bce query` subcommand: read-only against local SQLite, JSON output
- Offset/limit pagination for paging through results
- QueryEnvelope and QueryPost models for structured JSONL output
- Integration with existing fetch path via unified CLI

### cli-common-maximal-sharing (Completed 2026-03-22)
**Goal:** Centralize remaining reusable CLI infrastructure in `tftio-cli-common` and establish explicit shared/local boundaries.

**Features delivered:**
- Shared metadata commands (version, license, completions, doctor) for all workspace tools
- Workspace tool presets and `StandardCommandMap` pattern
- Fatal runner handling and lazy JSON/text response emission
- Doctor report builders and repository shell-enforcement helpers
- Migration of `bce` to use cli-common infrastructure

**Shared/local boundary:** Tool crates keep domain command trees, domain summaries, and data collection required before shared rendering. Intentional local exceptions are limited to tool-specific clap enum bridges, tool-specific doctor state collection, domain formatting, and `prompter`'s dynamic completion augmentation.

## Current State

**Product status:** `bce` v0.1.0 shipped on 2026-03-22. The binary supports:
- Authenticated fetch via app password
- Exhaustive pagination with resume capability
- SQLite storage with idempotent writes
- Read-only query mode with JSONL pagination
- Shared metadata commands (version, license, completions, doctor)
- Progress reporting and completion summaries

**Architecture:** Both symbolic planning milestones (`bce-query-mode` and `cli-common-maximal-sharing`) completed on 2026-03-22. The tool now uses shared `tftio-cli-common` infrastructure for consistent workspace UX while maintaining domain-specific query functionality.

## Next Steps

No new symbolic milestone is defined yet.

Recommended next steps:
- Continue product work on BlueSky activity-type filtering when ready
- Start the next symbolic milestone with GSD workflow if architectural work is needed
- Keep release tagging/versioning under release-please rather than milestone completion

---
*Last updated: 2026-03-22 after reconciling `bce-query-mode` and `cli-common-maximal-sharing` milestones*
