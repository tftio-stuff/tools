---
phase: 03-extraction-engine
plan: 02
subsystem: http-client
tags: [rust, reqwest, at-protocol, bluesky, rate-limit, pagination, sqlite]

# Dependency graph
requires:
  - tftio-bsky-comment-extractor crate scaffold (03-01)
  - ExtractorError, models, db layer (03-01)
provides:
  - BskyClient with authenticate, refresh_auth, resolve_handle, execute, fetch_all_posts
  - run_extraction public entry point tying auth + resolve + fetch + DB together
  - backoff_delay, parse_retry_after, extract_post_fields public helpers
  - 14 unit tests in client.rs covering all pure functions and DB integration
affects: [04-cli]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - execute() retry loop with attempt counter and refreshed flag for 401 handling
    - parse_retry_after reads Retry-After header first, falls back to ratelimit-reset Unix epoch
    - backoff_delay: 1u64 << attempt.min(63) capped at 60s (avoids saturating_shl unavailability)
    - fetch_all_posts uses let...else for extract_post_fields (clippy manual_let_else compliance)
    - future_not_send allow on async fns taking &rusqlite::Connection (expected; single-threaded)
    - body: Option<Vec<u8>> for cloneable retry bodies (not Option<&[u8]> which causes Sized issues)

key-files:
  created: []
  modified:
    - crates/bsky-comment-extractor/src/client.rs
    - crates/bsky-comment-extractor/src/lib.rs
    - .gitignore

key-decisions:
  - "execute() uses Option<Vec<u8>> for body (not Option<&[u8]>) to allow body cloning across retry iterations"
  - "backoff_delay uses 1u64 << attempt.min(63) not saturating_shl (unavailable on stable u64)"
  - "reqwest::Bytes not exposed in workspace reqwest 0.13 without stream feature; use Vec<u8> as return type from execute()"
  - "future_not_send allowed on fetch_all_posts and run_extraction since rusqlite::Connection is inherently not Send"
  - "AppView requires backticks in doc comments per workspace doc_markdown lint"

patterns-established:
  - "Pattern: retry loop with attempt + refreshed flag -- increment attempt for 429/network, not for 401"
  - "Pattern: parse_retry_after checks Retry-After (seconds) then ratelimit-reset (Unix epoch)"
  - "Pattern: fetch_all_posts saves cursor after every page, not just on completion"

requirements-completed: [AUTH-01, AUTH-02, EXTR-01, EXTR-02, EXTR-03, EXTR-04]

# Metrics
duration: 5min
completed: 2026-03-22
---

# Phase 03 Plan 02: AT Protocol HTTP Client Summary

**`BskyClient` with `BlueSky` authentication, handle resolution, exhaustive cursor-based pagination, rate-limit backoff, token refresh, and `run_extraction` public entry point wiring all layers together**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-22T16:10:16Z
- **Completed:** 2026-03-22T16:15:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Full `BskyClient` implementation: `authenticate` (createSession), `refresh_auth` (refreshSession), `resolve_handle` (resolveHandle), `execute` (retry loop with 429/401/network handling), `fetch_all_posts` (pagination with incremental stop and since cutoff)
- Public helpers: `backoff_delay` (1s * 2^n capped at 60s), `parse_retry_after` (Retry-After header + ratelimit-reset fallback), `extract_post_fields` (text/createdAt from raw JSON)
- `run_extraction` top-level entry point: reads `BSKY_APP_PASSWORD`, opens/inits DB, authenticates or resolves handle, calls `fetch_all_posts`
- Re-exports: `pub use client::BskyClient` and `pub use error::ExtractorError` in lib.rs
- 23 tests green: 14 client tests (unit + DB integration) + 9 existing db tests

## Task Commits

Each task was committed atomically:

1. **Task 1: BskyClient with auth, resolve, rate-limit backoff** - `aed7a0f` (feat)
2. **Task 2: run_extraction entry point and public re-exports** - `ff6986b` (feat)

## Files Created/Modified

- `crates/bsky-comment-extractor/src/client.rs` - Full BskyClient implementation with all methods and 14 tests
- `crates/bsky-comment-extractor/src/lib.rs` - Added run_extraction function and BskyClient/ExtractorError re-exports
- `.gitignore` - Added .cargo/ entry to suppress Nix linker config from being tracked

## Decisions Made

- `execute()` takes `body: Option<Vec<u8>>` (not `Option<&[u8]>`) so the body can be cloned on each retry iteration without lifetime complications
- `backoff_delay` uses bit-shift `1u64 << attempt.min(63)` because `saturating_shl` is not available on stable `u64` in the workspace MSRV
- `reqwest::Bytes` is not re-exported by reqwest 0.13 without the `stream` feature; `execute()` returns `Vec<u8>` instead
- `#[allow(clippy::future_not_send)]` added to `fetch_all_posts` and `run_extraction` because `rusqlite::Connection` is intentionally not `Send` (single-threaded DB access)
- `AppView` requires backtick wrapping in doc comments under the workspace `doc_markdown` pedantic lint

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] bytes::Bytes not available in reqwest 0.13 without stream feature**
- **Found during:** Task 1 compilation
- **Issue:** Plan specified `bytes::Bytes` return type for `execute()` but neither `bytes` crate nor `reqwest::Bytes` re-export is available in workspace
- **Fix:** Changed return type to `Vec<u8>` and converted with `.to_vec()` on success path
- **Files modified:** crates/bsky-comment-extractor/src/client.rs
- **Commit:** aed7a0f

**2. [Rule 1 - Bug] saturating_shl not available on u64 in stable Rust**
- **Found during:** Task 1 compilation
- **Issue:** `1u64.saturating_shl(attempt)` does not exist; clippy suggested `saturating_mul` which would be wrong semantically
- **Fix:** Used `1u64 << attempt.min(63)` which is equivalent and correct
- **Files modified:** crates/bsky-comment-extractor/src/client.rs
- **Commit:** aed7a0f

**3. [Rule 1 - Bug] Option<&[u8]> body parameter causes Sized errors with ? operator**
- **Found during:** Task 1 compilation
- **Issue:** Rust requires all local variables to be Sized; `[u8]` is unsized and caused compile errors when awaiting a Result containing it
- **Fix:** Changed to `Option<Vec<u8>>` and cloned with `.clone()` in the retry loop
- **Files modified:** crates/bsky-comment-extractor/src/client.rs
- **Commit:** aed7a0f

**4. [Rule 2 - Missing critical functionality] .cargo/config.toml (Nix linker) was untracked**
- **Found during:** Task 1 git commit check
- **Issue:** Nix-generated .cargo/config.toml appeared in git status as untracked; would pollute commits
- **Fix:** Added `.cargo/` to .gitignore
- **Files modified:** .gitignore
- **Commit:** aed7a0f

**5. [Rule 1 - Bug] 5 clippy pedantic errors on initial client.rs**
- **Found during:** Task 1 clippy run
- **Issue:** `doc_markdown` lint for `AppView` (2 occurrences), `manual_let_else` for match on `extract_post_fields`, `single_match_else` covered by same fix
- **Fix:** Added backticks around `AppView`, converted match to `let...else` pattern
- **Files modified:** crates/bsky-comment-extractor/src/client.rs
- **Commit:** aed7a0f (fixes applied before commit)

---

**Total deviations:** 5 auto-fixed (3 compilation bugs, 1 missing gitignore, 1 lint fixes)
**Impact on plan:** All fixes were correctness requirements. No scope creep. Public API shape preserved.

## Self-Check: PASSED

- FOUND: crates/bsky-comment-extractor/src/client.rs
- FOUND: crates/bsky-comment-extractor/src/lib.rs
- FOUND: commit aed7a0f (Task 1)
- FOUND: commit ff6986b (Task 2)
