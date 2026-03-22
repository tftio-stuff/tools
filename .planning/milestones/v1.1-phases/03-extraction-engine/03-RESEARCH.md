# Phase 3: Extraction Engine - Research

**Researched:** 2026-03-22
**Domain:** AT Protocol client, Rust async HTTP, SQLite persistence
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Auth credential handling:**
- App password via `BSKY_APP_PASSWORD` env var only (no interactive prompt, no config file)
- Target user handle is a function/API argument (CLI positional arg wiring is Phase 4)
- Auth handle = target handle (same user model)
- Auth is optional: without `BSKY_APP_PASSWORD`, use public API (`public.api.bsky.app`) with a warning about lower rate limits
- Session created via `com.atproto.server.createSession`, tokens managed internally

**SQLite schema design:**
- Parsed key fields as columns + raw JSON blob column for full record
- Parsed columns: AT URI (primary key), author DID, post text, created_at timestamp
- Reply structure, embed info, language tags are NOT parsed into columns -- queryable via `json_extract()` on the raw blob if needed
- Separate `extractions` metadata table tracking: target DID, started_at, completed_at, record_count, last_cursor
- Single database file for all users (author_did column distinguishes), not per-user files
- Idempotent writes via UPSERT (INSERT OR REPLACE on AT URI)

**Pagination and resumability:**
- Resume from last cursor on re-run: store cursor in metadata table, continue where interrupted
- `--since` flag (optional date cutoff, default = everything) -- wired as a function parameter in this phase, CLI flag in Phase 4
- Smart incremental updates: on re-run for a user with completed extraction, only fetch records newer than last extraction; stop when hitting already-seen records
- Cursor saved on graceful exit and on interruption (where possible)

**Rate limit behavior:**
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

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUTH-01 | CLI accepts BlueSky handle and app password for authentication | `BSKY_APP_PASSWORD` env var; `createSession` endpoint documented below |
| AUTH-02 | Session created via `com.atproto.server.createSession` with token refresh | Endpoint shape verified; `refreshSession` uses Bearer refreshJwt |
| EXTR-01 | Retrieve all `app.bsky.feed.post` records via `com.atproto.repo.listRecords` | Endpoint signature, params, response shape fully documented |
| EXTR-02 | Paginate exhaustively through full post history | Cursor field in response; loop until no cursor returned |
| EXTR-03 | Resolve user handle to DID when handle is provided | `resolveHandle` endpoint signature confirmed |
| EXTR-04 | Respect rate limits with backoff on HTTP 429 | Headers: `ratelimit-limit`, `ratelimit-remaining`, `ratelimit-reset`; Retry-After header also present |
| STOR-01 | Store posts in SQLite with structured schema | Schema design documented; workspace rusqlite pattern confirmed |
| STOR-02 | Idempotent writes -- re-running updates existing records | INSERT OR REPLACE on AT URI primary key |
| STOR-03 | Configurable database file path (default: `./bsky-posts.db`) | `open_db(path)` pattern from todoer; path passed as fn arg |
</phase_requirements>

## Summary

Phase 3 adds a new `bsky-comment-extractor` library crate to the existing Cargo workspace. The crate provides a pure library surface (no CLI -- that is Phase 4) for authenticating to BlueSky, resolving handles to DIDs, exhaustively fetching all `app.bsky.feed.post` records via cursor-based pagination, and persisting them to SQLite with idempotent UPSERT semantics.

All foundational Rust dependencies are already present in the workspace (`reqwest` 0.13 rustls, `rusqlite` 0.38 bundled, `tokio` 1, `serde`, `serde_json`, `chrono`, `thiserror`). The new crate adds no net-new external dependencies. The workspace has two high-quality reference implementations to model: `asana-cli` (async HTTP client with retry/rate-limit/pagination patterns) and `silent-critic` (complex rusqlite schema with WAL mode, CRUD helpers, and in-memory test DB).

The AT Protocol lexicon for the three required endpoints has been verified from official GitHub sources: `com.atproto.server.createSession` (POST), `com.atproto.identity.resolveHandle` (GET), and `com.atproto.repo.listRecords` (GET). Cursor-based pagination is well-defined: the response includes an optional `cursor` field; when absent, pagination is complete. Rate limit headers follow the draft IETF pattern: `ratelimit-limit`, `ratelimit-remaining`, `ratelimit-reset` (Unix epoch), `ratelimit-policy`.

**Primary recommendation:** Model the client closely on `asana-cli`'s `ApiClient` (retry loop, rate-limit header extraction, `backoff_delay`) and the DB layer on `silent-critic`'s `db.rs` (WAL mode, `open_db` + `init_db`, typed CRUD functions, in-memory tests). Use async `reqwest` (tokio), not blocking, since the workspace runtime is already `rt-multi-thread`.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.13 (workspace) | Async HTTP client with rustls | Already in workspace; used by asana-cli |
| rusqlite | 0.38 bundled (workspace) | SQLite with bundled libsqlite3 | Already in workspace; used by todoer and silent-critic |
| tokio | 1 rt-multi-thread (workspace) | Async runtime | Already in workspace |
| serde + serde_json | 1 (workspace) | JSON de/serialization for AT Protocol responses | Already in workspace |
| chrono | 0.4 (workspace) | DateTime parsing for `createdAt` timestamps | Already in workspace |
| thiserror | 2 (workspace) | Derive error types | Already in workspace |
| tracing | 0.1 (workspace) | Structured logging | Already in workspace |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio time | feature flag | `sleep` for backoff delays | Add `time` feature in crate Cargo.toml alongside `rt-multi-thread`, `macros` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| async reqwest | reqwest::blocking | Blocking avoids async complexity but workspace is already async; blocking would require a separate thread pool; do not use |
| thiserror | anyhow | anyhow is good for application code; thiserror gives typed errors callers can match on -- prefer thiserror for a library crate |

**Installation:** No new dependencies. The new crate Cargo.toml reuses all workspace deps.

```toml
# crates/bsky-comment-extractor/Cargo.toml (dependencies section)
chrono = { workspace = true, features = ["std", "clock"] }
reqwest.workspace = true
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["time"] }
tracing.workspace = true
```

## Architecture Patterns

### Recommended Project Structure
```
crates/bsky-comment-extractor/
+-- Cargo.toml
+-- src/
|   +-- lib.rs          # public API re-exports; top-level doc comment
|   +-- client.rs       # BskyClient struct: auth, resolveHandle, listRecords, retry/backoff
|   +-- db.rs           # open_db, init_db, upsert_post, upsert_extraction, CRUD helpers
|   +-- models.rs       # serde structs for AT Protocol responses and DB row types
|   +-- error.rs        # ExtractorError enum via thiserror
+-- tests/
    +-- integration.rs  # DB roundtrip tests using Connection::open_in_memory()
```

### Pattern 1: Async Client with Retry and Rate-Limit Backoff
**What:** A `BskyClient` struct holds a `reqwest::Client`, optional `accessJwt`, and `base_url`. The `execute` private method runs the retry loop with exponential backoff on 429 and server errors.
**When to use:** All HTTP calls go through `execute`; callers call typed helpers (`get_json`, `post_json`).
**Example:**
```rust
// Modeled on crates/asana-cli/src/api/client.rs execute() method
async fn execute(&self, method: Method, path: &str, query: &[(&str, &str)])
    -> Result<Bytes, ExtractorError>
{
    let mut attempt = 0u32;
    loop {
        let mut req = self.http.request(method.clone(), self.build_url(path));
        if !query.is_empty() { req = req.query(query); }
        if let Some(token) = &self.access_jwt {
            req = req.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        match req.send().await {
            Err(e) if (e.is_timeout() || e.is_connect()) && attempt < MAX_RETRIES => {
                tokio::time::sleep(backoff_delay(attempt)).await;
                attempt += 1;
            }
            Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                let wait = parse_retry_after(resp.headers())
                    .unwrap_or_else(|| backoff_delay(attempt));
                if attempt >= MAX_RETRIES { return Err(ExtractorError::RateLimitExhausted); }
                tokio::time::sleep(wait).await;
                attempt += 1;
            }
            Ok(resp) if resp.status().is_success() => {
                return Ok(resp.bytes().await?);
            }
            Ok(resp) => return Err(ExtractorError::Http(resp.status())),
            Err(e) => return Err(ExtractorError::Network(e)),
        }
    }
}

fn backoff_delay(attempt: u32) -> Duration {
    // 1s, 2s, 4s, 8s, 16s -- capped at 60s
    Duration::from_secs(1u64.saturating_shl(attempt).min(60))
}
```

### Pattern 2: Cursor-Based Exhaustive Pagination
**What:** Loop calling `listRecords` with incrementing cursor until response has no cursor. Stop early if `--since` date is exceeded or if an already-seen AT URI is encountered (incremental mode).
**When to use:** `fetch_all_posts` public function.
**Example:**
```rust
// Source: AT Protocol lexicon verification (docs.bsky.app)
// GET /xrpc/com.atproto.repo.listRecords
//   ?repo=did:plc:...
//   &collection=app.bsky.feed.post
//   &limit=100
//   &cursor=<opaque string>
pub async fn fetch_all_posts(
    &self,
    did: &str,
    since: Option<DateTime<Utc>>,
    db: &Connection,
) -> Result<FetchSummary, ExtractorError> {
    let mut cursor: Option<String> = self.load_resume_cursor(db, did)?;
    let mut count = 0u64;
    loop {
        let mut params = vec![
            ("repo", did),
            ("collection", "app.bsky.feed.post"),
            ("limit", "100"),
        ];
        let cursor_str = cursor.clone().unwrap_or_default();
        if cursor.is_some() { params.push(("cursor", &cursor_str)); }

        let page: ListRecordsResponse = self.get_json("/xrpc/com.atproto.repo.listRecords", &params).await?;

        for record in &page.records {
            // incremental: stop at already-seen or before since date
            if db_has_uri(db, &record.uri)? { return Ok(FetchSummary { count, done: true }); }
            if let Some(cutoff) = since {
                if record.value.created_at < cutoff { return Ok(FetchSummary { count, done: true }); }
            }
            upsert_post(db, record, did)?;
            count += 1;
        }
        save_cursor(db, did, page.cursor.as_deref())?;

        match page.cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    Ok(FetchSummary { count, done: true })
}
```

### Pattern 3: SQLite Schema with WAL Mode
**What:** `open_db` opens the connection and sets PRAGMAs; `init_db` creates tables. Both follow the `silent-critic` pattern exactly.
**When to use:** Called once at startup by the public `Extractor` entry point.
**Example:**
```rust
// Modeled on crates/silent-critic/src/db.rs open_db + init_db
pub fn open_db(path: &Path) -> Result<Connection, ExtractorError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;"
    )?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<(), ExtractorError> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS posts (
            uri         TEXT PRIMARY KEY,
            author_did  TEXT NOT NULL,
            text        TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            raw_json    TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS extractions (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            target_did    TEXT NOT NULL,
            started_at    TEXT NOT NULL,
            completed_at  TEXT,
            record_count  INTEGER NOT NULL DEFAULT 0,
            last_cursor   TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_posts_author_did ON posts(author_did);
        CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at);
        CREATE INDEX IF NOT EXISTS idx_extractions_target_did ON extractions(target_did);
    ")?;
    Ok(())
}
```

### Pattern 4: UPSERT for Idempotent Writes
**What:** `INSERT OR REPLACE INTO posts` on AT URI primary key. No duplicate check needed.
**When to use:** Every record from `listRecords`.
**Example:**
```rust
// rusqlite INSERT OR REPLACE pattern (idempotent)
pub fn upsert_post(conn: &Connection, record: &PostRecord, author_did: &str)
    -> Result<(), ExtractorError>
{
    conn.execute(
        "INSERT OR REPLACE INTO posts (uri, author_did, text, created_at, raw_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            record.uri,
            author_did,
            record.value.text,
            record.value.created_at,
            serde_json::to_string(&record.value)?,
        ],
    )?;
    Ok(())
}
```

### Pattern 5: Auth Token Management
**What:** `createSession` on startup (if `BSKY_APP_PASSWORD` set), store `accessJwt` and `refreshJwt`. On 401 response, call `refreshSession` once using `refreshJwt` as Bearer token, then retry original request. If refresh fails, return `ExtractorError::AuthExpired`.
**When to use:** All authenticated requests go through the retry loop; auth refresh is transparent.
**Example:**
```rust
// POST /xrpc/com.atproto.server.createSession
// Body: {"identifier": "user.bsky.social", "password": "app-password-here"}
// Response: {"accessJwt": "...", "refreshJwt": "...", "did": "did:plc:...", "handle": "..."}

// POST /xrpc/com.atproto.server.refreshSession
// Header: Authorization: Bearer <refreshJwt>   (NOT accessJwt)
// Response: same shape as createSession
```

### Anti-Patterns to Avoid
- **Using `getAuthorFeed` instead of `listRecords`:** `getAuthorFeed` returns rendered feed views and may skip records; `listRecords` returns the raw repo and is complete. The decision to use `listRecords` is locked.
- **Passing handle to `listRecords`:** The `repo` parameter accepts both handle and DID, but the `extractions` metadata table keys on DID. Always resolve handle to DID first so the metadata table rows are stable.
- **Using `reqwest::blocking`:** The workspace runtime is `rt-multi-thread`; blocking reqwest inside tokio requires `spawn_blocking` overhead. Use async reqwest throughout.
- **Missing WAL mode:** Without `PRAGMA journal_mode = WAL`, concurrent read/write from a second process (e.g., a query tool) will block. Always enable WAL.
- **Not saving cursor before exit:** If cursor is only saved at the end of a successful fetch, interruption loses progress. Save cursor after each page.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SQLite connection + schema | Custom migration system | `execute_batch` with `CREATE TABLE IF NOT EXISTS` | Workspace pattern; no migration tool needed for v1 |
| HTTP retry loop | Custom retry wrapper | Model on asana-cli `execute()` | Already proven, handles edge cases (timeout vs 5xx vs 429) |
| Rate limit header parsing | Custom header parser | Read `ratelimit-reset` as Unix epoch, compute `Duration::from_secs(reset - now)` | Simple arithmetic; see asana-cli `parse_retry_after` |
| JSON deserialization | Manual string splitting | `serde_json` + typed structs | AT Protocol responses are well-structured; let serde handle it |
| Timestamp comparison for incremental | String comparison | Parse `createdAt` with `chrono::DateTime::parse_from_rfc3339` | ISO 8601 strings sort correctly as strings only if zero-padded, which AT Protocol guarantees -- but typed comparison is safer |

**Key insight:** The workspace already contains a complete, tested implementation of the retry + rate-limit + pagination pattern in `asana-cli`. The bsky client should be a simplified adaptation, not a reimplementation from first principles.

## Common Pitfalls

### Pitfall 1: listRecords `repo` Parameter Accepts Handle but Metadata Needs DID
**What goes wrong:** You call `listRecords` with `?repo=user.bsky.social` and it works, but then you key the `extractions` row on the handle. If the user changes their handle, resume logic breaks.
**Why it happens:** The API accepts both; it's tempting to skip the resolution step.
**How to avoid:** Always call `resolveHandle` first. Store and key everything on DID. The handle is only used in the initial API call.
**Warning signs:** `extractions` rows with `target_did` values that look like `user.bsky.social` instead of `did:plc:...`.

### Pitfall 2: Token Expiry During Long Extractions
**What goes wrong:** `accessJwt` expires mid-extraction; all subsequent requests return 401; extraction aborts.
**Why it happens:** `accessJwt` has a short TTL (typically ~2 hours for BlueSky); a user with many posts may take longer.
**How to avoid:** On any 401 response, attempt one `refreshSession` call using `refreshJwt` as the Bearer token. If refresh succeeds, update stored `accessJwt` and retry the original request. If refresh fails, return `ExtractorError::AuthExpired` with a clear message.
**Warning signs:** 401 errors appearing 1-2 hours into a long extraction.

### Pitfall 3: Cursor Expiry / Pagination Staleness
**What goes wrong:** A stored cursor is too old; the API returns an error instead of a page.
**Why it happens:** AT Protocol cursors may be time-bounded at the PDS level. A cursor saved from a previous run weeks ago may be invalid.
**How to avoid:** When resuming from a saved cursor, handle cursor-invalid errors (likely a 400 with a specific error code) by restarting from the beginning or from a `created_at` date filter. The `--since` parameter provides a fallback.
**Warning signs:** 400 response with cursor-related error message on first request after resume.

### Pitfall 4: Incomplete Records on Public API Without Auth
**What goes wrong:** The public API at `public.api.bsky.app` has lower rate limits and may not expose all records (e.g., records from deactivated accounts or accounts with content warnings).
**Why it happens:** Public AppView applies filters; repo endpoint via PDS is more complete.
**How to avoid:** Document the limitation in the `ExtractorConfig` when `BSKY_APP_PASSWORD` is not set. Emit a `tracing::warn!` on startup.
**Warning signs:** Record count from public API does not match known post count.

### Pitfall 5: Missing `missing_docs` Compliance
**What goes wrong:** Workspace lints include `missing_docs = "deny"`. Every public item needs a doc comment.
**Why it happens:** Easy to forget when iterating quickly; compile error only appears at `cargo build`.
**How to avoid:** Write doc comments as items are created. Check `todoer`'s `#[lints] missing_docs = "allow"` -- if the doc burden is too high, add the same override locally in the new crate's `[lints]` section. The planner should decide.
**Warning signs:** `error: missing documentation for ...` at build time.

### Pitfall 6: Incremental Detection Logic for Already-Seen Records
**What goes wrong:** The "stop when hitting already-seen records" logic is order-dependent. `listRecords` returns records in reverse chronological order by default (newest first). If the user has posted since the last extraction, newer posts appear first, then we hit the watermark.
**Why it happens:** The correct stopping condition is "AT URI already in DB" (exact match), not created_at comparison -- posts can be backdated.
**How to avoid:** Check AT URI presence in the DB for each record during the page scan. This is an O(1) indexed lookup on the primary key. Stop the entire fetch (not just skip) when the first known URI is encountered.
**Warning signs:** Re-fetching all records on incremental runs, or stopping too early.

## Code Examples

Verified patterns from official sources:

### createSession Request/Response
```rust
// Source: github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/server/createSession.json
// POST /xrpc/com.atproto.server.createSession
// Content-Type: application/json

#[derive(Serialize)]
struct CreateSessionRequest<'a> {
    identifier: &'a str,  // handle or DID
    password: &'a str,    // app password
}

#[derive(Deserialize)]
struct CreateSessionResponse {
    #[serde(rename = "accessJwt")]
    access_jwt: String,
    #[serde(rename = "refreshJwt")]
    refresh_jwt: String,
    handle: String,
    did: String,
    // active, status, email etc. are optional -- use Option<T>
    active: Option<bool>,
}
```

### resolveHandle Request/Response
```rust
// Source: github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/identity/resolveHandle.json
// GET /xrpc/com.atproto.identity.resolveHandle?handle=user.bsky.social
// Does NOT require auth

#[derive(Deserialize)]
struct ResolveHandleResponse {
    did: String,
}
```

### listRecords Request/Response
```rust
// Source: github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/repo/listRecords.json
// GET /xrpc/com.atproto.repo.listRecords
//   ?repo=did:plc:...          (required; accepts handle or DID)
//   &collection=app.bsky.feed.post  (required NSID)
//   &limit=100                 (optional; 1-100, default 50)
//   &cursor=<opaque>           (optional; omit for first page)
//   &reverse=false             (optional; true = oldest first)
// Does NOT require auth

#[derive(Deserialize)]
struct ListRecordsResponse {
    cursor: Option<String>,  // absent = last page
    records: Vec<RepoRecord>,
}

#[derive(Deserialize)]
struct RepoRecord {
    uri: String,  // at://did:plc:.../app.bsky.feed.post/tid
    cid: String,
    value: PostValue,  // the actual post record
}

#[derive(Deserialize, Serialize)]
struct PostValue {
    #[serde(rename = "$type")]
    record_type: Option<String>,  // "app.bsky.feed.post"
    text: String,
    #[serde(rename = "createdAt")]
    created_at: String,           // ISO 8601 datetime
    reply: Option<serde_json::Value>,
    embed: Option<serde_json::Value>,
    langs: Option<Vec<String>>,
    // ...other optional fields stored in raw_json
}
```

### Rate Limit Header Parsing
```rust
// Source: github.com/bluesky-social/atproto/discussions/697 + IETF draft headers
// Headers: ratelimit-limit, ratelimit-remaining, ratelimit-reset (Unix epoch), ratelimit-policy
// Also: Retry-After (seconds or HTTP date) on 429 responses

fn parse_rate_limit_reset(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let reset_str = headers.get("ratelimit-reset")?.to_str().ok()?;
    let reset_epoch: u64 = reset_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    Some(Duration::from_secs(reset_epoch.saturating_sub(now).max(1)))
}

fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    // Retry-After takes priority; fall back to ratelimit-reset
    if let Some(val) = headers.get(reqwest::header::RETRY_AFTER) {
        if let Ok(secs) = val.to_str().ok()?.parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
    }
    parse_rate_limit_reset(headers)
}
```

### New Crate Cargo.toml Pattern
```toml
# crates/bsky-comment-extractor/Cargo.toml
[package]
name = "tftio-bsky-comment-extractor"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Extract BlueSky post history to SQLite"

[lints]
workspace = true

[lib]
name = "bsky_comment_extractor"
path = "src/lib.rs"

[dependencies]
chrono = { workspace = true, features = ["std", "clock"] }
reqwest.workspace = true
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["time"] }
tracing.workspace = true
```

### Adding to Workspace
```toml
# Root Cargo.toml -- add to [workspace] members array
members = [
    "crates/cli-common",
    "crates/prompter",
    "crates/unvenv",
    "crates/asana-cli",
    "crates/todoer",
    "crates/silent-critic",
    "crates/gator",
    "crates/bsky-comment-extractor",  # add this line
]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `getAuthorFeed` for post retrieval | `com.atproto.repo.listRecords` with collection `app.bsky.feed.post` | AT Protocol design; feed API is a view | `listRecords` is complete; feed may omit records |
| OAuth / DPoP auth | App password (`createSession`) | App passwords introduced mid-2023 | Simpler auth; no DPoP complexity; sufficient for personal tools |
| Per-shard PDS resolution | `public.api.bsky.app` for unauthenticated, PDS for authenticated | PDS distribution model | Public AppView aggregates; use it without auth; use PDS base URL from `createSession` response for auth |

**Deprecated/outdated:**
- `entities` field in `app.bsky.feed.post`: Replaced by `facets`. Schema still accepts it but new posts use facets. For extraction, store both in raw_json; no need to parse either.
- `getAuthorFeed`: Returns rendered feed views. Not suitable for complete extraction. Use `listRecords`.

## Open Questions

1. **PDS base URL for authenticated requests**
   - What we know: `createSession` response includes `didDoc` (optional) which may contain PDS service endpoint. The `bsky.social` PDS is `https://bsky.social`.
   - What's unclear: For users on third-party PDS instances, the correct auth endpoint is not `https://bsky.social`. The DID document should be consulted.
   - Recommendation: For v1, hardcode `https://bsky.social` as the PDS URL for auth. Document the limitation. The `didDoc` field in `createSession` response can be used in v2 for correct PDS routing. The user base for a personal extraction tool is likely on `bsky.social`.

2. **Cursor validity window**
   - What we know: AT Protocol spec does not document cursor TTL. Community reports suggest cursors may expire.
   - What's unclear: Exact expiry window; whether a 400 is returned with a specific error code.
   - Recommendation: On cursor-resume failure (any non-success response on first page with cursor), retry from scratch. Log a warning. This is a graceful degradation, not an error.

3. **`missing_docs` compliance burden**
   - What we know: The workspace lint `missing_docs = "deny"` applies. `todoer` overrides it to `"allow"`. `asana-cli` fully complies with doc comments.
   - What's unclear: Whether the library crate warrants full doc coverage or a local lint override.
   - Recommendation: Comply with `missing_docs`. This is a library crate with a public API; doc comments are appropriate. Follow `asana-cli`'s example (every pub item has a doc comment). The planner should schedule this as part of each implementation task.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (no additional test framework needed) |
| Config file | none -- `cargo test -p tftio-bsky-comment-extractor` |
| Quick run command | `cargo test -p tftio-bsky-comment-extractor` |
| Full suite command | `cargo test -p tftio-bsky-comment-extractor` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | `BSKY_APP_PASSWORD` env var read; missing = public API mode | unit | `cargo test -p tftio-bsky-comment-extractor -- auth` | Wave 0 |
| AUTH-02 | `createSession` request body serializes correctly | unit (mock) | `cargo test -p tftio-bsky-comment-extractor -- create_session` | Wave 0 |
| EXTR-01 | `listRecords` query params built correctly | unit | `cargo test -p tftio-bsky-comment-extractor -- list_records` | Wave 0 |
| EXTR-02 | Pagination loop stops when cursor absent | unit | `cargo test -p tftio-bsky-comment-extractor -- pagination` | Wave 0 |
| EXTR-03 | `resolveHandle` returns DID from mock response | unit (mock) | `cargo test -p tftio-bsky-comment-extractor -- resolve_handle` | Wave 0 |
| EXTR-04 | Backoff delay doubles on consecutive 429 | unit | `cargo test -p tftio-bsky-comment-extractor -- backoff` | Wave 0 |
| STOR-01 | `init_db` creates posts + extractions tables | unit | `cargo test -p tftio-bsky-comment-extractor -- schema` | Wave 0 |
| STOR-02 | `upsert_post` called twice with same URI does not duplicate | unit | `cargo test -p tftio-bsky-comment-extractor -- upsert_idempotent` | Wave 0 |
| STOR-03 | `open_db` accepts arbitrary path; default path is `./bsky-posts.db` | unit | `cargo test -p tftio-bsky-comment-extractor -- db_path` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p tftio-bsky-comment-extractor`
- **Per wave merge:** `cargo test -p tftio-bsky-comment-extractor`
- **Phase gate:** Full suite green + `cargo clippy -p tftio-bsky-comment-extractor` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/bsky-comment-extractor/src/lib.rs` -- crate entry point
- [ ] `crates/bsky-comment-extractor/src/error.rs` -- `ExtractorError` type
- [ ] `crates/bsky-comment-extractor/src/models.rs` -- AT Protocol serde structs
- [ ] `crates/bsky-comment-extractor/src/client.rs` -- `BskyClient` with HTTP methods
- [ ] `crates/bsky-comment-extractor/src/db.rs` -- `open_db`, `init_db`, CRUD
- [ ] `crates/bsky-comment-extractor/Cargo.toml` -- crate manifest
- [ ] Root `Cargo.toml` -- add member `crates/bsky-comment-extractor`
- [ ] Tests embedded in each module under `#[cfg(test)]` using `Connection::open_in_memory()` for DB tests

## Sources

### Primary (HIGH confidence)
- github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/repo/listRecords.json -- endpoint params, response shape, cursor field
- github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/server/createSession.json -- auth request/response shape
- github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/server/refreshSession.json -- refresh auth; uses refreshJwt as Bearer
- github.com/bluesky-social/atproto/blob/main/lexicons/com/atproto/identity/resolveHandle.json -- handle-to-DID resolution
- github.com/bluesky-social/atproto/blob/main/lexicons/app/bsky/feed/post.json -- post record fields
- crates/asana-cli/src/api/client.rs -- retry loop, rate-limit header extraction, backoff, pagination stream
- crates/asana-cli/src/api/error.rs -- RateLimitInfo struct, ApiError enum
- crates/todoer/src/db.rs -- open_db + init_db minimal pattern
- crates/silent-critic/src/db.rs -- open_db with WAL + create_dir_all, complex CRUD, in-memory tests
- crates/asana-cli/Cargo.toml -- workspace dep override pattern (features)
- main/Cargo.toml -- workspace deps list, lint configuration

### Secondary (MEDIUM confidence)
- github.com/bluesky-social/atproto/discussions/697 -- rate limit header names confirmed (ratelimit-limit, ratelimit-remaining, ratelimit-reset, ratelimit-policy); 3000 req/5min read limit
- docs.bsky.app/docs/advanced-guides/rate-limits -- general rate limit categories; 3000 req/5min PDS read limit
- atproto.com/specs/xrpc -- 429 status + optional Retry-After header

### Tertiary (LOW confidence)
- getskyscraper.com/blog/bluesky-rate-limits-api-guide -- secondary source corroborating header names and values; not official

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all dependencies already in workspace; versions confirmed
- Architecture: HIGH -- modeled directly on verified workspace reference implementations
- AT Protocol endpoint signatures: HIGH -- verified from official lexicon JSON files
- Rate limit headers: MEDIUM -- confirmed from community discussions + IETF draft; exact header casing (lowercase) confirmed from issue #3217
- Pitfalls: HIGH -- derived from code analysis and AT Protocol design characteristics

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (AT Protocol endpoints stable; rate limits may change)
