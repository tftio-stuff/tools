---
phase: 03-extraction-engine
verified: 2026-03-22T20:00:00Z
status: passed
score: 12/12 must-haves verified
gaps: []
human_verification:
  - test: "Run extraction against live BlueSky API with a real handle and app password"
    expected: "Posts appear in the SQLite database; tool exits cleanly; re-running does not duplicate rows"
    why_human: "HTTP auth and pagination against live AT Protocol API cannot be verified without network access"
  - test: "Trigger HTTP 429 during an extraction run (or throttle network)"
    expected: "Tool backs off, logs wait duration, retries, and eventually completes or saves cursor and exits with RateLimitExhausted"
    why_human: "Rate limit backoff logic is implemented and unit-tested for the retry_after parsing and backoff_delay math, but the full end-to-end 429 flow requires a live API or a mock HTTP server"
---

# Phase 3: Extraction Engine Verification Report

**Phase Goal:** A user's complete BlueSky post history can be fetched and stored in SQLite
**Verified:** 2026-03-22T20:00:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Given a handle and app password, the client authenticates via `com.atproto.server.createSession` and receives a valid session token | VERIFIED | `BskyClient::authenticate` POSTs to `/xrpc/com.atproto.server.createSession`, stores `access_jwt`/`refresh_jwt`, returns DID; `test_client_new_auth_mode` confirms PDS base URL is set |
| 2 | All posts are retrieved via `com.atproto.repo.listRecords` with cursor-based pagination until no cursor remains | VERIFIED | `fetch_all_posts` loops, passes `cursor` param, breaks on `page.cursor == None`; constant `COLLECTION_FEED_POST = "app.bsky.feed.post"` and `PAGE_LIMIT = "100"` used |
| 3 | A handle is resolved to a DID before fetching records | VERIFIED | `BskyClient::resolve_handle` calls `/xrpc/com.atproto.identity.resolveHandle`; `run_extraction` calls `resolve_handle` when no password is set, `authenticate` (which returns DID) when password is present |
| 4 | On HTTP 429, the client backs off and retries rather than crashing | VERIFIED | `execute()` retry loop handles `StatusCode::TOO_MANY_REQUESTS`; `parse_retry_after` reads `Retry-After`/`ratelimit-reset` headers; `backoff_delay` provides exponential fallback; `test_backoff_delay`, `test_parse_retry_after_seconds`, `test_parse_rate_limit_reset` all pass |
| 5 | Posts are written to SQLite with AT URI, author DID, text, created_at, and raw JSON; re-running does not duplicate rows | VERIFIED | `upsert_post` uses `INSERT OR REPLACE INTO posts (uri, author_did, text, created_at, raw_json)`; `test_upsert_post_idempotent` proves single-row constraint; schema has `uri TEXT PRIMARY KEY` |
| 6 | SQLite schema creates posts and extractions tables with correct columns | VERIFIED | `init_db` creates both tables; `test_init_db_creates_tables` verifies both table names present in `sqlite_master`; `test_upsert_post_insert` verifies all 5 columns round-trip |
| 7 | Inserting the same post twice by AT URI produces one row, not two | VERIFIED | `test_upsert_post_idempotent`: inserts same URI twice, `COUNT(*) = 1`, second insert updates `text` to "Updated" |
| 8 | Database can be opened at any path; parent directories are created | VERIFIED | `open_db` calls `create_dir_all(parent)` before `Connection::open`; `test_open_db_creates_parent_dirs` uses three-level nested path under tmpdir |
| 9 | On HTTP 401 mid-extraction, the client attempts one token refresh before failing | VERIFIED | `execute()` handles `StatusCode::UNAUTHORIZED`: if not yet refreshed, calls `refresh_auth()` and sets `refreshed = true`; second 401 returns `ExtractorError::AuthExpired` |
| 10 | After 5 consecutive 429 retries, cursor is saved and `RateLimitExhausted` error returned | VERIFIED | `execute()` returns `Err(ExtractorError::RateLimitExhausted)` when `attempt >= MAX_RETRIES`; `MAX_RETRIES = 5`; `fetch_all_posts` saves cursor after every page so cursor is persisted before the error surfaces |
| 11 | Without `BSKY_APP_PASSWORD`, the client uses the public API with a `tracing::warn` | VERIFIED | `BskyClient::new(None)` sets `base_url = PUBLIC_API_BASE` and calls `tracing::warn!("No BSKY_APP_PASSWORD set; using public API with lower rate limits")`; `test_client_new_public_mode` asserts `base_url == PUBLIC_API_BASE` |
| 12 | `run_extraction` is the single public entry point tying auth, resolve, fetch, and DB together | VERIFIED | `pub async fn run_extraction(handle, db_path, since)` in `lib.rs`: opens/inits DB, reads env var, creates client, authenticates or resolves, calls `fetch_all_posts`; `pub use client::BskyClient` and `pub use error::ExtractorError` re-exported |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` (root) | Lists crate in workspace members | VERIFIED | Line 11: `"crates/bsky-comment-extractor"` |
| `release-please-config.json` | Entry for tftio-bsky-comment-extractor | VERIFIED | `"package-name": "tftio-bsky-comment-extractor"` present |
| `crates/bsky-comment-extractor/Cargo.toml` | Crate manifest with workspace deps | VERIFIED | `name = "tftio-bsky-comment-extractor"`, all expected deps via `workspace = true` |
| `crates/bsky-comment-extractor/src/lib.rs` | Module declarations and `run_extraction` | VERIFIED | Declares `pub mod client/db/error/models`; `pub use client::BskyClient`; `pub use error::ExtractorError`; `pub async fn run_extraction` |
| `crates/bsky-comment-extractor/src/error.rs` | `ExtractorError` enum with all variants | VERIFIED | All 10 variants present: `Http`, `Network`, `Db`, `Json`, `Io`, `RateLimitExhausted`, `AuthExpired`, `AuthFailed`, `InvalidHandle`, `CursorExpired` |
| `crates/bsky-comment-extractor/src/models.rs` | AT Protocol serde structs | VERIFIED | All 7 structs present: `CreateSessionRequest`, `CreateSessionResponse`, `ResolveHandleResponse`, `ListRecordsResponse`, `RepoRecord`, `PostValue`, `FetchSummary` |
| `crates/bsky-comment-extractor/src/db.rs` | SQLite storage layer with 7 public functions | VERIFIED | All 7 functions present: `open_db`, `init_db`, `upsert_post`, `db_has_uri`, `save_cursor`, `load_resume_cursor`, `complete_extraction`; 9 unit tests |
| `crates/bsky-comment-extractor/src/client.rs` | `BskyClient` with auth, resolve, fetch, retry | VERIFIED | `BskyClient` struct with `authenticate`, `refresh_auth`, `resolve_handle`, `execute`, `fetch_all_posts`; helpers `backoff_delay`, `parse_retry_after`, `extract_post_fields`; 14 tests; 368 lines (well above min_lines: 150) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Cargo.toml` | `crates/bsky-comment-extractor` | workspace members array | WIRED | `"crates/bsky-comment-extractor"` on line 11 |
| `client.rs` | `db.rs` | `crate::db::` calls | WIRED | `crate::db::load_resume_cursor`, `db_has_uri`, `upsert_post`, `save_cursor`, `complete_extraction` all called in `fetch_all_posts` |
| `client.rs` | `models.rs` | `crate::models::` imports | WIRED | `use crate::models::{CreateSessionRequest, CreateSessionResponse, FetchSummary, ListRecordsResponse, ResolveHandleResponse}` |
| `client.rs` | `error.rs` | `crate::error::` return types | WIRED | `use crate::error::ExtractorError` at top of `client.rs` |
| `db.rs` | `models.rs` | `PostRecord` type (pattern in plan) | WIRED (via raw strings) | `db.rs` does not import `models.rs` directly -- `upsert_post` takes plain `&str` arguments; caller in `client.rs` does the model deserialization. Not a gap: this is the correct design (db layer is decoupled from AT Protocol types) |
| `db.rs` | `error.rs` | `ExtractorError` return type | WIRED | `use crate::error::ExtractorError` at top of `db.rs` |
| `lib.rs` | `client.rs` | `run_extraction` calling `BskyClient` | WIRED | `run_extraction` creates `BskyClient::new(...)`, calls `authenticate`/`resolve_handle`/`fetch_all_posts` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AUTH-01 | 03-02 | CLI accepts BlueSky handle and app password for authentication | SATISFIED | `run_extraction(handle, ...)` reads `BSKY_APP_PASSWORD` from env; `BskyClient::authenticate(handle, pw)` wires both into `createSession` |
| AUTH-02 | 03-02 | Session created via `com.atproto.server.createSession` with token refresh | SATISFIED | `authenticate` POSTs to `/xrpc/com.atproto.server.createSession`; `refresh_auth` POSTs to `/xrpc/com.atproto.server.refreshSession`; `execute` triggers refresh on 401 |
| EXTR-01 | 03-02 | Retrieve all `app.bsky.feed.post` records via `com.atproto.repo.listRecords` | SATISFIED | `fetch_all_posts` calls `execute(GET, "/xrpc/com.atproto.repo.listRecords", ...)` with `collection = COLLECTION_FEED_POST` |
| EXTR-02 | 03-02 | Paginate exhaustively through full post history | SATISFIED | `fetch_all_posts` loops until `page.cursor` is `None`; cursor passed as query param on each iteration |
| EXTR-03 | 03-02 | Resolve user handle to DID | SATISFIED | `resolve_handle` calls `/xrpc/com.atproto.identity.resolveHandle`; wired in `run_extraction` for the no-credentials path |
| EXTR-04 | 03-02 | Respect rate limits with backoff on HTTP 429 | SATISFIED | `execute` handles 429 with `parse_retry_after`/`backoff_delay`; after `MAX_RETRIES` returns `RateLimitExhausted`; all header-parsing and backoff math unit-tested |
| STOR-01 | 03-01 | Store posts in SQLite with structured schema (AT URI, author DID, text, created_at, reply parent, raw JSON) | SATISFIED (with design note) | Schema has `uri`, `author_did`, `text`, `created_at`, `raw_json`. No dedicated `reply_parent` column; per `03-CONTEXT.md` this is an explicit design decision: reply structure is queryable via `json_extract()` on `raw_json`. The full reply JSON is stored in `raw_json`. |
| STOR-02 | 03-01 | Idempotent writes -- re-running updates existing records, does not duplicate | SATISFIED | `INSERT OR REPLACE INTO posts` on `uri TEXT PRIMARY KEY`; proven by `test_upsert_post_idempotent` |
| STOR-03 | 03-01 | Configurable database file path (default: `./bsky-posts.db`) | SATISFIED (library layer) | `open_db(path: &Path)` and `run_extraction(handle, db_path, since)` accept any path; default path (`./bsky-posts.db`) is a CLI concern deferred to Phase 4 |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/lib.rs` | 39-43 | `if let Some(ref pw) = password { ... } else { ... }` triggers `clippy::option_if_let_else` warning | Info | `clippy::nursery` is `warn`-level in the workspace, not `deny`. Does not block builds or CI. Cosmetic only. |

No stubs, TODO comments, empty implementations, placeholder returns, or missing wiring detected in any of the five source files. All public functions have doc comments complying with the `missing_docs = "deny"` workspace lint.

### Human Verification Required

#### 1. Live extraction against real BlueSky account

**Test:** Set `BSKY_APP_PASSWORD` to a real app password and call `run_extraction("your.handle", Path::new("./test.db"), None)` (or wire a quick binary harness), targeting an account with a known post count.
**Expected:** The function returns `FetchSummary { count: N, done: true }` where N matches the account's actual post count; `./test.db` contains N rows in the `posts` table. Re-running yields the same N (incremental stop on first known URI).
**Why human:** Authentication tokens, live API responses, real pagination cursor sequences, and actual post counts cannot be verified without network access.

#### 2. Rate-limit backoff end-to-end

**Test:** Run extraction against a low-rate-limit endpoint or throttle the connection to force consecutive HTTP 429 responses, watching tracing output.
**Expected:** Each 429 logs the wait duration derived from `Retry-After` or `ratelimit-reset`; after 5 consecutive failures `RateLimitExhausted` is returned; the cursor saved before failure allows resumption on re-run.
**Why human:** Cannot reliably trigger 5 consecutive 429 responses in unit tests without a mock HTTP server (not present in this crate). The retry math and header parsing are unit-tested; the full retry loop requires live or mocked HTTP.

### Test Results

- `cargo test -p tftio-bsky-comment-extractor`: **23/23 passed** (9 db tests + 14 client tests)
- `cargo clippy -p tftio-bsky-comment-extractor`: **0 errors, 1 nursery warning** (`option_if_let_else` in `lib.rs:39`)
- `cargo build -p tftio-bsky-comment-extractor`: implicit in test run, clean

### Gaps Summary

No gaps. All 12 observable truths are verified, all 8 artifacts are substantive and wired, all 9 requirements are satisfied, and the test suite is fully green. The one clippy `nursery` warning in `lib.rs` is cosmetic and not a blocker under workspace lint policy.

The `reply parent` column absence noted under STOR-01 is a documented design decision (see `03-CONTEXT.md` line 26), not a missing implementation. Reply data is preserved in `raw_json` and remains queryable via `json_extract()`.

---

_Verified: 2026-03-22T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
