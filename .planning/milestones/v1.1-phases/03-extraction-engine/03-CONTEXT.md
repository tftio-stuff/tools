# Phase 3: Extraction Engine - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

AT Protocol client library that authenticates to BlueSky, resolves handles to DIDs, exhaustively fetches all `app.bsky.feed.post` records for a user via `com.atproto.repo.listRecords`, and stores them in a local SQLite database. This is the library layer -- CLI surface is Phase 4.

</domain>

<decisions>
## Implementation Decisions

### Auth credential handling
- App password via `BSKY_APP_PASSWORD` env var only (no interactive prompt, no config file)
- Target user handle is a function/API argument (CLI positional arg wiring is Phase 4)
- Auth handle = target handle (same user model)
- Auth is optional: without `BSKY_APP_PASSWORD`, use public API (`public.api.bsky.app`) with a warning about lower rate limits
- Session created via `com.atproto.server.createSession`, tokens managed internally

### SQLite schema design
- Parsed key fields as columns + raw JSON blob column for full record
- Parsed columns: AT URI (primary key), author DID, post text, created_at timestamp
- Reply structure, embed info, language tags are NOT parsed into columns -- queryable via `json_extract()` on the raw blob if needed
- Separate `extractions` metadata table tracking: target DID, started_at, completed_at, record_count, last_cursor
- Single database file for all users (author_did column distinguishes), not per-user files
- Idempotent writes via UPSERT (INSERT OR REPLACE on AT URI)

### Pagination & resumability
- Resume from last cursor on re-run: store cursor in metadata table, continue where interrupted
- `--since` flag (optional date cutoff, default = everything) -- wired as a function parameter in this phase, CLI flag in Phase 4
- Smart incremental updates: on re-run for a user with completed extraction, only fetch records newer than last extraction; stop when hitting already-seen records
- Cursor saved on graceful exit and on interruption (where possible)

### Rate limit behavior
- Reactive pacing: fire requests as fast as API allows, back off on HTTP 429
- Read `RateLimit-*` response headers to calibrate wait times
- Exponential backoff on consecutive 429s (1s, 2s, 4s, 8s...)
- After 5 consecutive retries with no success, save cursor to metadata table and exit with clear error message
- Re-running after rate limit failure resumes from saved cursor

### Claude's Discretion
- Internal module structure (how to split client.rs, db.rs, models.rs, etc.)
- Exact exponential backoff parameters and jitter
- Whether to use `tokio` async or sync `reqwest::blocking` (workspace already has tokio)
- Error type hierarchy design
- How to detect "already-seen records" for incremental mode (by created_at comparison or by AT URI presence in DB)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### AT Protocol API
- No local spec files -- use AT Protocol documentation at docs.bsky.app
- Key endpoints: `com.atproto.server.createSession`, `com.atproto.repo.listRecords`, `com.atproto.identity.resolveHandle`
- Collection NSID for posts: `app.bsky.feed.post`
- Pagination: cursor-based, up to 100 records per page

### Workspace patterns (reference implementations)
- `crates/asana-cli/src/api/client.rs` -- Async HTTP client with retry/backoff, pagination, caching (reference for client design)
- `crates/todoer/src/db.rs` -- Simple rusqlite schema init pattern (reference for SQLite setup)
- `crates/silent-critic/src/db.rs` -- More complex rusqlite usage (reference for schema design)
- `crates/asana-cli/Cargo.toml` -- Workspace dependency override pattern (reqwest features, tokio features)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest` (workspace dep, rustls) -- HTTP client, already configured
- `rusqlite` (workspace dep, bundled) -- SQLite, already configured
- `tokio` (workspace dep) -- Async runtime
- `serde` + `serde_json` (workspace deps) -- JSON handling
- `chrono` (workspace dep) -- Timestamp parsing
- `indicatif` (workspace dep) -- Progress bars (Phase 4, but available)
- `thiserror` (workspace dep) -- Error types
- `tracing` + `tracing-subscriber` (workspace deps) -- Logging
- `tftio-cli-common` -- Shared CLI utilities (Phase 4)

### Established Patterns
- `asana-cli` API client: builder pattern (`ApiClientOptions`), retry with exponential backoff, `RateLimitInfo` struct, pagination via async streams
- `todoer` DB: `open_db(path)` + `init_db(conn)` pattern, `PRAGMA foreign_keys`, `execute_batch` for schema
- Workspace lints: `clippy::pedantic = deny`, `missing_docs = deny` -- new crate must comply or override locally

### Integration Points
- Root `Cargo.toml` workspace members array -- add `crates/bsky-comment-extractor`
- Workspace dependencies -- reuse existing, add new AT Protocol-specific deps if needed
- `release-please-config.json` -- register new crate for release management

</code_context>

<specifics>
## Specific Ideas

- The earlier research in this conversation established that `com.atproto.repo.listRecords` is the right endpoint for completeness -- it returns raw repo records, not rendered feed views
- `listRecords` requires a DID (not handle), so handle-to-DID resolution via `com.atproto.identity.resolveHandle` is a prerequisite step
- Public API base URL is `https://public.api.bsky.app`; authenticated requests go through PDS (typically `https://bsky.social`)
- Rate limit is ~3,000 requests per 5 minutes; `listRecords` paginates at up to 100 records per request

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 03-extraction-engine*
*Context gathered: 2026-03-22*
