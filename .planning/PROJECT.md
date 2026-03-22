# bsky-comment-extractor

## What This Is

A Rust CLI tool that exhaustively retrieves a BlueSky user's network activity via the AT Protocol, stores it in a local SQLite database, and supports filtering by activity type. A new crate in the existing `tools` Cargo workspace.

## Core Value

Complete, reliable extraction of a single BlueSky user's entire post and interaction history into a queryable local store.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Authenticate to BlueSky via app password
- [ ] Retrieve all posts (top-level and replies) for a given user handle or DID
- [ ] Use `com.atproto.repo.listRecords` for completeness over `getAuthorFeed`
- [ ] Store results in a local SQLite database
- [ ] Support filtering by activity type: posts, likes, reposts, quote-posts, blocks, blocked-by
- [ ] Default filter: posts (all posts including replies) when no filter specified
- [ ] Paginate exhaustively through full history
- [ ] CLI interface following workspace conventions (clap, cli-common)

### Out of Scope

- Firehose/streaming consumption — batch retrieval only
- Multi-user extraction in a single invocation
- Real-time monitoring or polling
- OAuth authentication — app passwords are sufficient
- Search by keyword — this extracts a user's activity, not search results
- Output formats other than SQLite (JSONL, CSV, etc.) — may revisit later

## Context

- BlueSky's `com.atproto.repo.listRecords` provides raw repository records by collection, giving complete historical data without the gaps that `getAuthorFeed` may have
- Relevant AT Protocol collections: `app.bsky.feed.post`, `app.bsky.feed.like`, `app.bsky.feed.repost`, `app.bsky.graph.block`
- Quote-posts are regular posts with an embed of type `app.bsky.embed.record` — filtered client-side
- "Blocked-by" is not stored in the user's own repo; requires `app.bsky.graph.getBlocks` or similar API call
- The workspace already uses `rusqlite` (bundled) and `reqwest` with rustls — reuse these
- Rate limit: ~3,000 requests per 5 minutes; `listRecords` paginated at up to 100 records per request
- No auth needed for public read endpoints, but authenticated requests may have higher limits

## Constraints

- **Tech stack**: Rust, workspace member of `tools/`. Must follow workspace lint config, dependency patterns, and crate structure
- **Dependencies**: Prefer workspace-level deps (`reqwest`, `rusqlite`, `serde`, `serde_json`, `clap`, `tokio`, `thiserror`, `chrono`). Add AT Protocol-specific deps as needed
- **Auth**: App password only (no OAuth complexity)
- **API approach**: `com.atproto.repo.listRecords` as primary data source for owned-repo collections; API endpoints for data not in user's repo (blocked-by)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SQLite output | Queryable, structured, consistent with workspace (todoer, silent-critic) | — Pending |
| `listRecords` over `getAuthorFeed` | Completeness over richness; raw repo data captures everything | — Pending |
| App password auth only | Simple, well-supported, OAuth adds DPoP complexity for no gain here | — Pending |
| Workspace crate, not standalone | Shares deps, lint config, CI, release tooling | — Pending |

---
*Last updated: 2026-03-22 after initialization*
